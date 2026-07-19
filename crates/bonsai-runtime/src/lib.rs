//! Bounded, cross-platform child-process transport for BONSAI adapters.

#![forbid(unsafe_code)]

use prost::Message;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const HARD_MAX_TRANSPORT_FRAME_BYTES: u32 = 16 * 1024 * 1024;
const FAILURE_DETAIL_LIMIT: usize = 256;

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
}

impl ProcessCommand {
    #[must_use]
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            arguments: Vec::new(),
            current_directory: None,
            environment: Vec::new(),
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
        command
            .args(&specification.arguments)
            .envs(specification.environment.iter().cloned())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(directory) = &specification.current_directory {
            command.current_dir(directory);
        }
        let mut child = command
            .spawn()
            .map_err(|error| TransportError::ProcessSpawn(error.kind()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or(TransportError::ProcessSpawn(io::ErrorKind::BrokenPipe))?;
        let mut stdout = child
            .stdout
            .take()
            .ok_or(TransportError::ProcessSpawn(io::ErrorKind::BrokenPipe))?;
        let mut stderr = child
            .stderr
            .take()
            .ok_or(TransportError::ProcessSpawn(io::ErrorKind::BrokenPipe))?;

        let (sender, receiver) = mpsc::sync_channel(limits.pending_frame_capacity);
        let reader_error = Arc::new(Mutex::new(None));
        let reader_error_thread = Arc::clone(&reader_error);
        let maximum_frame_bytes = limits.maximum_frame_bytes;
        let reader_thread = thread::spawn(move || {
            loop {
                match read_frame(&mut stdout, maximum_frame_bytes) {
                    Ok(Some(frame)) => {
                        if let Err(error) = try_send_frame(&sender, frame) {
                            set_reader_error(&reader_error_thread, error);
                            break;
                        }
                    }
                    Ok(None) => {
                        let _ = sender.send(ReaderMessage::EndOfStream);
                        break;
                    }
                    Err(error) => {
                        set_reader_error(&reader_error_thread, error);
                        break;
                    }
                }
            }
        });

        let stderr_state = Arc::new(Mutex::new(StderrState {
            retained: Vec::new(),
            total_bytes: 0,
            truncated: false,
        }));
        let stderr_state_thread = Arc::clone(&stderr_state);
        let retained_stderr_bytes = limits.retained_stderr_bytes;
        let stderr_thread = thread::spawn(move || {
            let mut buffer = [0_u8; 4096];
            loop {
                let read = match stderr.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => read,
                };
                let Ok(mut state) = stderr_state_thread.lock() else {
                    break;
                };
                state.total_bytes = state
                    .total_bytes
                    .saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
                let remaining = retained_stderr_bytes.saturating_sub(state.retained.len());
                state
                    .retained
                    .extend_from_slice(&buffer[..read.min(remaining)]);
                state.truncated |= read > remaining;
            }
        });

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
        })
    }

    /// Send one already encoded protocol message.
    ///
    /// # Errors
    ///
    /// Contains the child and records a bounded failure when framing or I/O fails.
    pub fn send(&mut self, frame: &[u8]) -> Result<(), TransportError> {
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
        let result = match self.receiver.as_ref() {
            Some(receiver) => match receiver.recv_timeout(timeout) {
                Ok(ReaderMessage::Frame(frame)) => Ok(Some(frame)),
                Ok(ReaderMessage::EndOfStream) => Ok(None),
                Err(mpsc::RecvTimeoutError::Timeout) => Err(TransportError::ReadTimeout),
                Err(mpsc::RecvTimeoutError::Disconnected) => self
                    .take_reader_error()
                    .map_or(Err(TransportError::ProtocolStreamClosed), Err),
            },
            None => Err(TransportError::ProtocolStreamClosed),
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
        let status = match wait_for_exit(&mut self.child, timeout) {
            Ok(status) => status,
            Err(error) => {
                self.failures.push(TransportFailure::from(&error));
                let _ = self.child.kill();
                let _ = self.child.wait();
                return Err(error);
            }
        };
        self.receiver.take();
        join_thread(self.reader_thread.take())?;
        join_thread(self.stderr_thread.take())?;
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
        self.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ChildTransport {
    fn drop(&mut self) {
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

fn wait_for_exit(child: &mut Child, timeout: Duration) -> Result<ExitStatus, TransportError> {
    let started = Instant::now();
    loop {
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

fn join_thread(thread: Option<JoinHandle<()>>) -> Result<(), TransportError> {
    thread.map_or(Ok(()), |thread| {
        thread.join().map_err(|_| TransportError::ThreadFailed)
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
