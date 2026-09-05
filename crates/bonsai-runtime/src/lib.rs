//! Bounded, cross-platform child-process transport for BONSAI adapters.

#![forbid(unsafe_code)]

mod conformance;
mod interruptible_io;
mod isolation;
mod lifecycle;
#[cfg(target_os = "linux")]
mod linux_transport;
mod replay;
mod storage;

pub use conformance::{
    AdapterCertificationInput, AdapterCertificationReport, AdapterConformanceSuite,
    CertificationCheck, CertificationError, CertificationVerdict, CheckResult, CheckVerdict,
    ProtocolExchange, TimeoutProbe,
};
pub use isolation::{
    AgentCapabilityAudit, AgentLaunchPolicy, GrantedInput, IsolatedLaunch, IsolatedRunLayout,
    IsolationError, ObserverAccessDenial, ObserverArtifactClass,
};
pub use lifecycle::{
    LifecycleError, LifecycleRecord, LifecycleState, RecoveredTransition, RecoveryReport,
    RunSupervisor, TransitionOutcome,
};
pub use replay::{
    ObserverReplayAnalyzer, ObserverReplayArtifact, ObserverReplayArtifactKind,
    ObserverReplayOutput, ReplayDestination, ReplayError, ReplayRouteDecision,
};
pub use storage::{
    AgentStorageBroker, InspectedObject, PersistDecision, PersistOutcome, PersistRequest,
    PersistenceClass, StorageError, StoragePolicy,
};

use interruptible_io::{InterruptibleReader, POLL_INTERVAL};
use prost::Message;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const HARD_MAX_TRANSPORT_FRAME_BYTES: u32 = 16 * 1024 * 1024;
const FAILURE_DETAIL_LIMIT: usize = 256;

/// Additional maximum cleanup allowance after an operation deadline on Linux.
pub const TRANSPORT_CLEANUP_ALLOWANCE: Duration = Duration::from_secs(1);
/// Default Linux write deadline; callers can supply a smaller one.
pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(5);

/// Thread-safe cancellation signal for an owned Linux adapter connection.
#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
pub struct TransportCancellation(Arc<AtomicBool>);

#[cfg(target_os = "linux")]
impl TransportCancellation {
    /// Wake pending pipe operations; the owner performs process cleanup.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportLimits {
    pub maximum_frame_bytes: u32,
    pub pending_frame_capacity: usize,
    pub retained_stderr_bytes: usize,
}

impl TransportLimits {
    /// Validate nonzero bounds below the hard implementation ceiling.
    ///
    /// # Errors
    ///
    /// Returns `TRANSPORT_LIMIT_INVALID` for a zero or excessive bound.
    pub fn validate(self) -> Result<Self, TransportError> {
        if self.maximum_frame_bytes == 0
            || self.maximum_frame_bytes > HARD_MAX_TRANSPORT_FRAME_BYTES
            || self.pending_frame_capacity == 0
            || self.retained_stderr_bytes == 0
        {
            Err(TransportError::LimitInvalid)
        } else {
            Ok(self)
        }
    }
}

impl Default for TransportLimits {
    fn default() -> Self {
        Self {
            maximum_frame_bytes: 1024 * 1024,
            pending_frame_capacity: 8,
            retained_stderr_bytes: 64 * 1024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportError {
    LimitInvalid,
    EmptyFrame,
    FrameTooLarge { declared: u32, maximum: u32 },
    HeaderPartial,
    PayloadPartial,
    ProtocolDecode,
    ReadTimeout,
    WriteTimeout,
    Cancelled,
    CleanupTimeout,
    BackpressureExceeded,
    ProtocolStreamClosed,
    ProcessSpawn(io::ErrorKind),
    Io(io::ErrorKind),
    ShutdownTimeout,
    ThreadFailed,
}

impl TransportError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::LimitInvalid => "TRANSPORT_LIMIT_INVALID",
            Self::EmptyFrame => "TRANSPORT_EMPTY_FRAME",
            Self::FrameTooLarge { .. } => "TRANSPORT_FRAME_TOO_LARGE",
            Self::HeaderPartial => "TRANSPORT_HEADER_PARTIAL",
            Self::PayloadPartial => "TRANSPORT_PAYLOAD_PARTIAL",
            Self::ProtocolDecode => "TRANSPORT_PROTOCOL_DECODE_FAILED",
            Self::ReadTimeout => "TRANSPORT_READ_TIMEOUT",
            Self::WriteTimeout => "TRANSPORT_WRITE_TIMEOUT",
            Self::Cancelled => "TRANSPORT_CANCELLED",
            Self::CleanupTimeout => "TRANSPORT_CLEANUP_TIMEOUT",
            Self::BackpressureExceeded => "TRANSPORT_BACKPRESSURE_EXCEEDED",
            Self::ProtocolStreamClosed => "TRANSPORT_STREAM_CLOSED",
            Self::ProcessSpawn(_) => "TRANSPORT_PROCESS_SPAWN_FAILED",
            Self::Io(_) => "TRANSPORT_IO_FAILED",
            Self::ShutdownTimeout => "TRANSPORT_SHUTDOWN_TIMEOUT",
            Self::ThreadFailed => "TRANSPORT_THREAD_FAILED",
        }
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        match self {
            Self::FrameTooLarge { declared, maximum } => {
                write!(formatter, ": declared={declared}, maximum={maximum}")
            }
            Self::ProcessSpawn(kind) | Self::Io(kind) => write!(formatter, ": {kind:?}"),
            _ => Ok(()),
        }
    }
}

impl Error for TransportError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportFailure {
    pub code: &'static str,
    pub bounded_detail: String,
}

impl From<&TransportError> for TransportFailure {
    fn from(error: &TransportError) -> Self {
        let detail = error.to_string();
        Self {
            code: error.code(),
            bounded_detail: detail.chars().take(FAILURE_DETAIL_LIMIT).collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StderrCapture {
    pub retained: Vec<u8>,
    pub total_bytes: u64,
    pub truncated: bool,
}

#[derive(Debug)]
struct StderrState {
    retained: Vec<u8>,
    total_bytes: u64,
    truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessCommand {
    pub program: OsString,
    pub arguments: Vec<OsString>,
    pub current_directory: Option<PathBuf>,
    pub environment: Vec<(OsString, OsString)>,
    pub clear_environment: bool,
}

impl ProcessCommand {
    #[must_use]
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            arguments: Vec::new(),
            current_directory: None,
            environment: Vec::new(),
            clear_environment: false,
        }
    }

    #[must_use]
    pub fn argument(mut self, argument: impl Into<OsString>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    #[must_use]
    pub fn current_directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.current_directory = Some(directory.into());
        self
    }

    #[must_use]
    pub fn environment(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.environment.push((key.into(), value.into()));
        self
    }

    #[must_use]
    pub const fn clear_environment(mut self) -> Self {
        self.clear_environment = true;
        self
    }
}

#[derive(Debug)]
enum ReaderMessage {
    Frame(Vec<u8>),
    EndOfStream,
}

/// A bounded protocol connection to one child adapter.
pub struct ChildTransport {
    child: Child,
    stdin: Option<BufWriter<ChildStdin>>,
    receiver: Option<Receiver<ReaderMessage>>,
    reader_error: Arc<Mutex<Option<TransportError>>>,
    reader_thread: Option<JoinHandle<()>>,
    stderr_state: Arc<Mutex<StderrState>>,
    stderr_thread: Option<JoinHandle<()>>,
    limits: TransportLimits,
    failures: Vec<TransportFailure>,
    stopped: Arc<AtomicBool>,
    #[cfg(target_os = "linux")]
    cleanup_result: Option<Result<(), TransportError>>,
}

impl ChildTransport {
    /// Spawn a child with only piped protocol stdin/stdout and an independently drained stderr.
    ///
    /// # Errors
    ///
    /// Returns a stable limit, spawn, or missing-pipe error.
    pub fn spawn(
        specification: &ProcessCommand,
        limits: TransportLimits,
    ) -> Result<Self, TransportError> {
        let limits = limits.validate()?;
        let mut command = Command::new(&specification.program);
        if specification.clear_environment {
            command.env_clear();
        }
        command
            .args(&specification.arguments)
            .envs(specification.environment.iter().cloned())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(directory) = &specification.current_directory {
            command.current_dir(directory);
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = command
            .spawn()
            .map_err(|error| TransportError::ProcessSpawn(error.kind()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or(TransportError::ProcessSpawn(io::ErrorKind::BrokenPipe))?;
        let stdout = child
            .stdout
            .take()
            .ok_or(TransportError::ProcessSpawn(io::ErrorKind::BrokenPipe))?;
        let stderr = child
            .stderr
            .take()
            .ok_or(TransportError::ProcessSpawn(io::ErrorKind::BrokenPipe))?;

        #[cfg(target_os = "linux")]
        if let Err(error) = linux_transport::nonblocking(&stdin)
            .and_then(|()| linux_transport::nonblocking(&stdout))
            .and_then(|()| linux_transport::nonblocking(&stderr))
        {
            let _ = linux_transport::kill_group(&child);
            let _ = wait_for_exit(&mut child, TRANSPORT_CLEANUP_ALLOWANCE);
            return Err(error);
        }
        let stopped = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::sync_channel(limits.pending_frame_capacity);
        let reader_error = Arc::new(Mutex::new(None));
        let reader_thread = spawn_reader(
            InterruptibleReader::new(stdout, Arc::clone(&stopped)),
            sender,
            Arc::clone(&reader_error),
            Arc::clone(&stopped),
            limits.maximum_frame_bytes,
        );
        let stderr_state = Arc::new(Mutex::new(StderrState {
            retained: Vec::new(),
            total_bytes: 0,
            truncated: false,
        }));
        let stderr_thread = spawn_stderr_reader(
            InterruptibleReader::new(stderr, Arc::clone(&stopped)),
            Arc::clone(&stderr_state),
            limits.retained_stderr_bytes,
        );

        Ok(Self {
            child,
            stdin: Some(BufWriter::new(stdin)),
            receiver: Some(receiver),
            reader_error,
            reader_thread: Some(reader_thread),
            stderr_state,
            stderr_thread: Some(stderr_thread),
            limits,
            failures: Vec::new(),
            stopped,
            #[cfg(target_os = "linux")]
            cleanup_result: None,
        })
    }

    /// Send one already encoded protocol message.
    ///
    /// # Errors
    ///
    /// Contains the child and records a bounded failure when framing or I/O fails.
    pub fn send(&mut self, frame: &[u8]) -> Result<(), TransportError> {
        #[cfg(target_os = "linux")]
        {
            self.send_with_timeout(frame, DEFAULT_WRITE_TIMEOUT)
        }
        #[cfg(not(target_os = "linux"))]
        self.send_legacy(frame)
    }

    #[cfg(not(target_os = "linux"))]
    fn send_legacy(&mut self, frame: &[u8]) -> Result<(), TransportError> {
        let result = self
            .stdin
            .as_mut()
            .ok_or(TransportError::ProtocolStreamClosed)
            .and_then(|stdin| write_frame(stdin, frame, self.limits.maximum_frame_bytes));
        if let Err(error) = &result {
            self.contain(error);
        }
        result
    }

    /// Send a frame under one wall deadline on Linux, including partial writes.
    ///
    /// # Errors
    ///
    /// Timeout or cancellation contains the process group and retains the failure.
    #[cfg(target_os = "linux")]
    pub fn send_with_timeout(
        &mut self,
        frame: &[u8],
        timeout: Duration,
    ) -> Result<(), TransportError> {
        let result = self
            .stdin
            .as_mut()
            .ok_or(TransportError::ProtocolStreamClosed)
            .and_then(|stdin| {
                let mut writer = interruptible_io::DeadlineWriter {
                    inner: stdin,
                    stopped: &self.stopped,
                    started: Instant::now(),
                    timeout,
                };
                write_frame(&mut writer, frame, self.limits.maximum_frame_bytes)
            })
            .map_err(|error| match error {
                TransportError::Io(io::ErrorKind::TimedOut) => TransportError::WriteTimeout,
                TransportError::Io(io::ErrorKind::ConnectionAborted) => TransportError::Cancelled,
                other => other,
            });
        if let Err(error) = &result {
            self.contain(error);
        }
        result
    }

    /// Return the owned process identifier for diagnostics and cleanup evidence.
    #[must_use]
    pub fn process_id(&self) -> u32 {
        self.child.id()
    }

    /// Obtain an external cancellation signal for pending Linux I/O.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn cancellation_handle(&self) -> TransportCancellation {
        TransportCancellation(Arc::clone(&self.stopped))
    }

    /// Cancel and clean up the Linux adapter group within the cleanup allowance.
    ///
    /// # Errors
    ///
    /// Returns the actual cleanup error rather than claiming successful termination.
    #[cfg(target_os = "linux")]
    pub fn cancel(&mut self) -> Result<(), TransportError> {
        self.failures
            .push(TransportFailure::from(&TransportError::Cancelled));
        let result = self.cleanup_linux(true);
        if let Err(error) = &result {
            self.failures.push(TransportFailure::from(error));
        }
        result
    }

    /// Encode and send one Protobuf message.
    ///
    /// # Errors
    ///
    /// Returns the same bounded transport failures as [`Self::send`].
    pub fn send_message<M: Message>(&mut self, message: &M) -> Result<(), TransportError> {
        self.send(&message.encode_to_vec())
    }

    /// Receive one frame before the caller-owned deadline duration expires.
    ///
    /// # Errors
    ///
    /// Timeout, malformed/partial/flood traffic, or stream failure terminates the child and emits one bounded failure.
    pub fn receive(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>, TransportError> {
        let started = Instant::now();
        let result = loop {
            if self.stopped.load(Ordering::Acquire) {
                break Err(TransportError::Cancelled);
            }
            let Some(receiver) = self.receiver.as_ref() else {
                break Err(TransportError::ProtocolStreamClosed);
            };
            let remaining = timeout.saturating_sub(started.elapsed());
            match receiver.recv_timeout(remaining.min(POLL_INTERVAL)) {
                Ok(ReaderMessage::Frame(frame)) => break Ok(Some(frame)),
                Ok(ReaderMessage::EndOfStream) => break Ok(None),
                Err(mpsc::RecvTimeoutError::Timeout) if started.elapsed() >= timeout => {
                    break Err(TransportError::ReadTimeout);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break if self.stopped.load(Ordering::Acquire) {
                        Err(TransportError::Cancelled)
                    } else {
                        self.take_reader_error()
                            .map_or(Err(TransportError::ProtocolStreamClosed), Err)
                    };
                }
            }
        };
        if let Err(error) = &result {
            self.contain(error);
        }
        result
    }

    /// Receive and decode one Protobuf message.
    ///
    /// # Errors
    ///
    /// Returns a bounded framing, deadline, stream, or decode error.
    pub fn receive_message<M: Message + Default>(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<M>, TransportError> {
        let result = self
            .receive(timeout)?
            .map(|bytes| M::decode(bytes.as_slice()).map_err(|_| TransportError::ProtocolDecode))
            .transpose();
        if let Err(error) = &result {
            self.contain(error);
        }
        result
    }

    #[must_use]
    pub fn failures(&self) -> &[TransportFailure] {
        &self.failures
    }

    #[must_use]
    pub fn stderr_snapshot(&self) -> StderrCapture {
        self.stderr_state.lock().map_or(
            StderrCapture {
                retained: Vec::new(),
                total_bytes: 0,
                truncated: true,
            },
            |state| StderrCapture {
                retained: state.retained.clone(),
                total_bytes: state.total_bytes,
                truncated: state.truncated,
            },
        )
    }

    /// Close protocol input and wait for a clean child exit, killing on timeout.
    ///
    /// # Errors
    ///
    /// Returns a bounded I/O, timeout, or thread failure after containing the process.
    pub fn shutdown(mut self, timeout: Duration) -> Result<ProcessOutcome, TransportError> {
        self.stdin.take();
        let status = match wait_for_exit_or_cancel(&mut self.child, timeout, Some(&self.stopped)) {
            Ok(status) => status,
            Err(error) => {
                self.contain(&error);
                return Err(error);
            }
        };
        #[cfg(target_os = "linux")]
        self.cleanup_linux(false)?;
        #[cfg(not(target_os = "linux"))]
        {
            self.receiver.take();
            join_thread(self.reader_thread.take())?;
            join_thread(self.stderr_thread.take())?;
        }
        let stderr = self.stderr_snapshot();
        Ok(ProcessOutcome {
            exit_code: status.code(),
            stderr,
            failures: std::mem::take(&mut self.failures),
        })
    }

    fn take_reader_error(&self) -> Option<TransportError> {
        self.reader_error.lock().ok()?.take()
    }

    fn contain(&mut self, error: &TransportError) {
        self.failures.push(TransportFailure::from(error));
        #[cfg(target_os = "linux")]
        if let Err(cleanup) = self.cleanup_linux(true) {
            self.failures.push(TransportFailure::from(&cleanup));
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.stdin.take();
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
    #[cfg(target_os = "linux")]
    fn cleanup_linux(&mut self, cancel: bool) -> Result<(), TransportError> {
        if let Some(result) = &self.cleanup_result {
            return result.clone();
        }
        let started = Instant::now();
        if cancel {
            self.stopped.store(true, Ordering::Release);
        }
        self.stdin.take();
        self.receiver.take();
        let killed = linux_transport::kill_group(&self.child);
        let waited = wait_for_exit(
            &mut self.child,
            TRANSPORT_CLEANUP_ALLOWANCE.saturating_sub(started.elapsed()),
        );
        let group = linux_transport::wait_group_exit(
            &self.child,
            TRANSPORT_CLEANUP_ALLOWANCE.saturating_sub(started.elapsed()),
        );
        let readers = join_before(&mut self.reader_thread, started)
            .and_then(|()| join_before(&mut self.stderr_thread, started));
        // On a failed cleanup, wake polling workers and report failure; never join forever.
        self.stopped.store(true, Ordering::Release);
        let result = killed.and(waited.map(|_| ())).and(group).and(readers);
        self.cleanup_result = Some(result.clone());
        result
    }
}

impl Drop for ChildTransport {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        {
            let _ = self.cleanup_linux(true);
        }
        #[cfg(not(target_os = "linux"))]
        self.drop_legacy();
    }
}

#[cfg(not(target_os = "linux"))]
impl ChildTransport {
    fn drop_legacy(&mut self) {
        self.stopped.store(true, Ordering::Release);
        self.stdin.take();
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        self.receiver.take();
        let _ = self.reader_thread.take().map(JoinHandle::join);
        let _ = self.stderr_thread.take().map(JoinHandle::join);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessOutcome {
    pub exit_code: Option<i32>,
    pub stderr: StderrCapture,
    pub failures: Vec<TransportFailure>,
}

/// Write one little-endian length-prefixed frame and flush it.
///
/// # Errors
///
/// Rejects empty/oversized frames before writing or returns a bounded I/O failure.
pub fn write_frame(
    writer: &mut impl Write,
    frame: &[u8],
    maximum: u32,
) -> Result<(), TransportError> {
    if frame.is_empty() {
        return Err(TransportError::EmptyFrame);
    }
    let declared = u32::try_from(frame.len()).unwrap_or(u32::MAX);
    if maximum == 0 || maximum > HARD_MAX_TRANSPORT_FRAME_BYTES {
        return Err(TransportError::LimitInvalid);
    }
    if declared > maximum {
        return Err(TransportError::FrameTooLarge { declared, maximum });
    }
    writer
        .write_all(&declared.to_le_bytes())
        .and_then(|()| writer.write_all(frame))
        .and_then(|()| writer.flush())
        .map_err(|error| TransportError::Io(error.kind()))
}

/// Read one little-endian length-prefixed frame without allocating above the declared bound.
///
/// # Errors
///
/// Distinguishes partial header, partial payload, empty, excessive, invalid-limit, and I/O outcomes.
pub fn read_frame(reader: &mut impl Read, maximum: u32) -> Result<Option<Vec<u8>>, TransportError> {
    if maximum == 0 || maximum > HARD_MAX_TRANSPORT_FRAME_BYTES {
        return Err(TransportError::LimitInvalid);
    }
    let mut header = [0_u8; 4];
    let first = reader
        .read(&mut header[..1])
        .map_err(|error| TransportError::Io(error.kind()))?;
    if first == 0 {
        return Ok(None);
    }
    read_exact_bounded(reader, &mut header[1..], TransportError::HeaderPartial)?;
    let declared = u32::from_le_bytes(header);
    if declared == 0 {
        return Err(TransportError::EmptyFrame);
    }
    if declared > maximum {
        return Err(TransportError::FrameTooLarge { declared, maximum });
    }
    let mut frame = vec![0_u8; declared as usize];
    read_exact_bounded(reader, &mut frame, TransportError::PayloadPartial)?;
    Ok(Some(frame))
}

fn read_exact_bounded(
    reader: &mut impl Read,
    mut target: &mut [u8],
    partial: TransportError,
) -> Result<(), TransportError> {
    while !target.is_empty() {
        match reader.read(target) {
            Ok(0) => return Err(partial),
            Ok(read) => target = &mut target[read..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(TransportError::Io(error.kind())),
        }
    }
    Ok(())
}

fn try_send_frame(
    sender: &SyncSender<ReaderMessage>,
    frame: Vec<u8>,
) -> Result<(), TransportError> {
    match sender.try_send(ReaderMessage::Frame(frame)) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(_)) => Err(TransportError::BackpressureExceeded),
        Err(TrySendError::Disconnected(_)) => Err(TransportError::ProtocolStreamClosed),
    }
}

fn set_reader_error(target: &Mutex<Option<TransportError>>, error: TransportError) {
    if let Ok(mut stored) = target.lock() {
        *stored = Some(error);
    }
}

#[cfg(target_os = "linux")]
fn wait_for_exit(child: &mut Child, timeout: Duration) -> Result<ExitStatus, TransportError> {
    wait_for_exit_or_cancel(child, timeout, None)
}

fn wait_for_exit_or_cancel(
    child: &mut Child,
    timeout: Duration,
    stopped: Option<&AtomicBool>,
) -> Result<ExitStatus, TransportError> {
    let started = Instant::now();
    loop {
        if stopped.is_some_and(|flag| flag.load(Ordering::Acquire)) {
            return Err(TransportError::Cancelled);
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| TransportError::Io(error.kind()))?
        {
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            return Err(TransportError::ShutdownTimeout);
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[cfg(not(target_os = "linux"))]
fn join_thread(thread: Option<JoinHandle<()>>) -> Result<(), TransportError> {
    thread.map_or(Ok(()), |thread| {
        thread.join().map_err(|_| TransportError::ThreadFailed)
    })
}

fn spawn_reader(
    mut stdout: impl Read + Send + 'static,
    sender: SyncSender<ReaderMessage>,
    error: Arc<Mutex<Option<TransportError>>>,
    stopped: Arc<AtomicBool>,
    maximum: u32,
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            match read_frame(&mut stdout, maximum) {
                Ok(Some(frame)) => {
                    if let Err(failure) = try_send_frame(&sender, frame) {
                        set_reader_error(&error, failure);
                        break;
                    }
                }
                Ok(None) => {
                    while !stopped.load(Ordering::Acquire) {
                        match sender.try_send(ReaderMessage::EndOfStream) {
                            Ok(()) | Err(TrySendError::Disconnected(_)) => break,
                            Err(TrySendError::Full(_)) => thread::sleep(POLL_INTERVAL),
                        }
                    }
                    break;
                }
                Err(failure) => {
                    set_reader_error(&error, failure);
                    break;
                }
            }
        }
    })
}

fn spawn_stderr_reader(
    mut stderr: impl Read + Send + 'static,
    state: Arc<Mutex<StderrState>>,
    retained_limit: usize,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        loop {
            let read = match stderr.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => read,
            };
            let Ok(mut state) = state.lock() else { break };
            state.total_bytes = state
                .total_bytes
                .saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
            let remaining = retained_limit.saturating_sub(state.retained.len());
            state
                .retained
                .extend_from_slice(&buffer[..read.min(remaining)]);
            state.truncated |= read > remaining;
        }
    })
}

#[cfg(target_os = "linux")]
fn join_before(
    handle: &mut Option<JoinHandle<()>>,
    started: Instant,
) -> Result<(), TransportError> {
    while handle.as_ref().is_some_and(|worker| !worker.is_finished()) {
        if started.elapsed() >= TRANSPORT_CLEANUP_ALLOWANCE {
            return Err(TransportError::CleanupTimeout);
        }
        thread::sleep(POLL_INTERVAL);
    }
    handle.take().map_or(Ok(()), |worker| {
        worker.join().map_err(|_| TransportError::ThreadFailed)
    })
}

#[cfg(test)]
mod tests {
    use super::{TransportError, read_frame, write_frame};
    use std::io::Cursor;

    #[test]
    fn bounded_frame_round_trip_is_exact() {
        let mut encoded = Vec::new();
        write_frame(&mut encoded, b"payload", 32).expect("write frame");
        assert_eq!(
            read_frame(&mut Cursor::new(encoded), 32),
            Ok(Some(b"payload".to_vec()))
        );
    }

    #[test]
    fn malformed_partial_and_oversized_frames_are_distinct() {
        assert_eq!(
            read_frame(&mut Cursor::new(vec![1, 2]), 32),
            Err(TransportError::HeaderPartial)
        );
        assert_eq!(
            read_frame(&mut Cursor::new([4, 0, 0, 0, 1, 2]), 32),
            Err(TransportError::PayloadPartial)
        );
        assert_eq!(
            read_frame(&mut Cursor::new([33, 0, 0, 0]), 32),
            Err(TransportError::FrameTooLarge {
                declared: 33,
                maximum: 32,
            })
        );
    }
}
