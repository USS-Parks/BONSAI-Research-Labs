use crate::ProcessCommand;
use bonsai_contracts::track::{Track, TrackDeclaration, derive_track};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Write as _};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const MAX_AUTHORIZED_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_INPUT_NAME_BYTES: usize = 64;
const AGENT_ROOT_ENV: &str = "BONSAI_AGENT_ROOT";
const INPUT_ROOT_ENV: &str = "BONSAI_INPUT_ROOT";
const WORK_ROOT_ENV: &str = "BONSAI_WORK_ROOT";

#[derive(Debug)]
#[non_exhaustive]
pub enum IsolationError {
    Io(io::Error),
    InvalidInputName,
    InputNotRegularFile,
    InputTooLarge { actual: u64, maximum: u64 },
    InputAlreadyGranted,
    InputHashInvalid,
    InputHashMismatch,
    InputOutsideGrantedRoot,
    ObserverPathExposure,
    NonUnicodeLaunchValue,
}

impl IsolationError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Io(_) => "ISOLATION_IO_FAILED",
            Self::InvalidInputName => "ISOLATION_INPUT_NAME_INVALID",
            Self::InputNotRegularFile => "ISOLATION_INPUT_NOT_REGULAR_FILE",
            Self::InputTooLarge { .. } => "ISOLATION_INPUT_TOO_LARGE",
            Self::InputAlreadyGranted => "ISOLATION_INPUT_ALREADY_GRANTED",
            Self::InputHashInvalid => "ISOLATION_INPUT_HASH_INVALID",
            Self::InputHashMismatch => "ISOLATION_INPUT_HASH_MISMATCH",
            Self::InputOutsideGrantedRoot => "ISOLATION_INPUT_OUTSIDE_GRANTED_ROOT",
            Self::ObserverPathExposure => "ISOLATION_OBSERVER_PATH_EXPOSURE",
            Self::NonUnicodeLaunchValue => "ISOLATION_LAUNCH_VALUE_NON_UNICODE",
        }
    }
}

impl fmt::Display for IsolationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        match self {
            Self::Io(error) => write!(formatter, ": {error}"),
            Self::InputTooLarge { actual, maximum } => {
                write!(formatter, ": actual={actual}, maximum={maximum}")
            }
            _ => Ok(()),
        }
    }
}

impl Error for IsolationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for IsolationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug)]
pub struct IsolatedRunLayout {
    root: PathBuf,
    agent_root: PathBuf,
    input_root: PathBuf,
    writable_root: PathBuf,
    observer_root: PathBuf,
    telemetry_root: PathBuf,
    index_root: PathBuf,
    report_root: PathBuf,
}

impl IsolatedRunLayout {
    /// Create sibling agent and observer trees beneath one supervisor-owned run root.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if any directory cannot be created or canonicalized.
    pub fn create(root: impl AsRef<Path>) -> Result<Self, IsolationError> {
        let root = root.as_ref();
        let agent_root = root.join("agent");
        let input_root = agent_root.join("inputs");
        let writable_root = agent_root.join("work");
        let observer_root = root.join("observer");
        let telemetry_root = observer_root.join("telemetry");
        let index_root = observer_root.join("index");
        let report_root = observer_root.join("reports");
        for directory in [
            &input_root,
            &writable_root,
            &telemetry_root,
            &index_root,
            &report_root,
        ] {
            fs::create_dir_all(directory)?;
        }
        Ok(Self {
            root: fs::canonicalize(root)?,
            agent_root: fs::canonicalize(agent_root)?,
            input_root: fs::canonicalize(input_root)?,
            writable_root: fs::canonicalize(writable_root)?,
            observer_root: fs::canonicalize(observer_root)?,
            telemetry_root: fs::canonicalize(telemetry_root)?,
            index_root: fs::canonicalize(index_root)?,
            report_root: fs::canonicalize(report_root)?,
        })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn agent_root(&self) -> &Path {
        &self.agent_root
    }

    #[must_use]
    pub fn input_root(&self) -> &Path {
        &self.input_root
    }

    #[must_use]
    pub fn writable_root(&self) -> &Path {
        &self.writable_root
    }

    #[must_use]
    pub fn observer_root(&self) -> &Path {
        &self.observer_root
    }

    #[must_use]
    pub fn telemetry_root(&self) -> &Path {
        &self.telemetry_root
    }

    #[must_use]
    pub fn index_root(&self) -> &Path {
        &self.index_root
    }

    #[must_use]
    pub fn report_root(&self) -> &Path {
        &self.report_root
    }

    /// Copy one manifest-authorized regular file into the immutable agent input tree.
    /// The source path itself is never included in the resulting grant.
    ///
    /// # Errors
    ///
    /// Rejects unsafe names, hashes, symlinks/non-files, excessive size, and duplicate grants.
    pub fn grant_input(
        &self,
        name: &str,
        source: impl AsRef<Path>,
        expected_sha256: &str,
    ) -> Result<GrantedInput, IsolationError> {
        validate_input_name(name)?;
        validate_sha256(expected_sha256)?;
        let source = source.as_ref();
        let metadata = fs::symlink_metadata(source)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(IsolationError::InputNotRegularFile);
        }
        if metadata.len() > MAX_AUTHORIZED_INPUT_BYTES {
            return Err(IsolationError::InputTooLarge {
                actual: metadata.len(),
                maximum: MAX_AUTHORIZED_INPUT_BYTES,
            });
        }
        let path = self.input_root.join(name);
        let mut source_file = File::open(source)?;
        let mut destination = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                if error.kind() == io::ErrorKind::AlreadyExists {
                    IsolationError::InputAlreadyGranted
                } else {
                    IsolationError::Io(error)
                }
            })?;
        let copy_result = (|| -> Result<u64, io::Error> {
            let copied = io::copy(&mut source_file, &mut destination)?;
            destination.flush()?;
            destination.sync_all()?;
            Ok(copied)
        })();
        drop(destination);
        let copied = match copy_result {
            Ok(copied) => copied,
            Err(error) => {
                let _ = fs::remove_file(&path);
                return Err(IsolationError::Io(error));
            }
        };
        if copied != metadata.len() {
            fs::remove_file(&path)?;
            return Err(IsolationError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "authorized input changed during copy",
            )));
        }
        let path = fs::canonicalize(path)?;
        if !path.starts_with(&self.input_root) {
            return Err(IsolationError::InputOutsideGrantedRoot);
        }
        let sha256 = sha256_file(&path)?;
        if sha256 != expected_sha256 {
            fs::remove_file(&path)?;
            return Err(IsolationError::InputHashMismatch);
        }
        let mut permissions = fs::metadata(&path)?.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions)?;
        Ok(GrantedInput {
            name: name.to_owned(),
            sha256,
            bytes: copied,
            path,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantedInput {
    pub name: String,
    pub sha256: String,
    pub bytes: u64,
    path: PathBuf,
}

impl GrantedInput {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentCapabilityAudit {
    pub current_directory: String,
    pub arguments: Vec<String>,
    pub environment_keys: Vec<String>,
    pub inherited_handles: Vec<String>,
    pub observer_path_exposed: bool,
}

#[derive(Clone, Debug)]
pub struct IsolatedLaunch {
    pub command: ProcessCommand,
    pub audit: AgentCapabilityAudit,
}

#[derive(Clone, Debug)]
pub struct AgentLaunchPolicy {
    layout: IsolatedRunLayout,
}

impl AgentLaunchPolicy {
    #[must_use]
    pub const fn new(layout: IsolatedRunLayout) -> Self {
        Self { layout }
    }

    /// Construct an environment-cleared launch specification containing only
    /// agent-tree paths and the inherited protocol/diagnostic standard streams.
    ///
    /// # Errors
    ///
    /// Rejects observer-path references, non-Unicode values, duplicate or stale grants.
    pub fn build_command(
        &self,
        program: impl Into<OsString>,
        adapter_arguments: impl IntoIterator<Item = OsString>,
        inputs: &[GrantedInput],
    ) -> Result<IsolatedLaunch, IsolationError> {
        let program = program.into();
        self.reject_observer_reference(&program)?;
        let mut arguments = adapter_arguments.into_iter().collect::<Vec<_>>();
        for argument in &arguments {
            self.reject_observer_reference(argument)?;
        }
        let mut names = BTreeSet::new();
        for input in inputs {
            validate_input_name(&input.name)?;
            let canonical = fs::canonicalize(&input.path)?;
            if canonical != input.path
                || !canonical.starts_with(&self.layout.input_root)
                || !names.insert(input.name.clone())
            {
                return Err(IsolationError::InputOutsideGrantedRoot);
            }
            arguments.push(OsString::from("--bonsai-input"));
            arguments.push(OsString::from(format!(
                "{}={}",
                input.name,
                unicode_path(&input.path)?
            )));
        }
        arguments.push(OsString::from("--bonsai-work-dir"));
        arguments.push(self.layout.writable_root.clone().into_os_string());

        let mut command = ProcessCommand::new(program)
            .clear_environment()
            .current_directory(&self.layout.agent_root)
            .environment(AGENT_ROOT_ENV, self.layout.agent_root.as_os_str())
            .environment(INPUT_ROOT_ENV, self.layout.input_root.as_os_str())
            .environment(WORK_ROOT_ENV, self.layout.writable_root.as_os_str());
        for argument in arguments {
            command = command.argument(argument);
        }
        let audit = self.audit(&command)?;
        if audit.observer_path_exposed {
            return Err(IsolationError::ObserverPathExposure);
        }
        Ok(IsolatedLaunch { command, audit })
    }

    /// Reject an outbound protocol frame containing any observer-tree path.
    ///
    /// # Errors
    ///
    /// Returns `ISOLATION_OBSERVER_PATH_EXPOSURE` before transport mutation.
    pub fn validate_protocol_payload(&self, payload: &[u8]) -> Result<(), IsolationError> {
        let payload = String::from_utf8_lossy(payload);
        if self.contains_observer_reference(&payload) {
            Err(IsolationError::ObserverPathExposure)
        } else {
            Ok(())
        }
    }

    /// Record a denied observer-artifact capability request and derive the
    /// resulting non-Track-A classification from BC-05 runtime facts.
    #[must_use]
    pub fn deny_observer_access(
        &self,
        declaration: &TrackDeclaration,
        requested: ObserverArtifactClass,
    ) -> ObserverAccessDenial {
        let mut observed_declaration = declaration.clone();
        observed_declaration.observer_data_access = true;
        let verdict = derive_track(&observed_declaration);
        ObserverAccessDenial {
            requested,
            allowed: false,
            code: "OBSERVER_ACCESS_DENIED",
            derived_track: verdict.derived,
            reason_code: verdict.reason_code,
            observed_declaration,
        }
    }

    fn audit(&self, command: &ProcessCommand) -> Result<AgentCapabilityAudit, IsolationError> {
        let current_directory = command
            .current_directory
            .as_deref()
            .ok_or(IsolationError::InputOutsideGrantedRoot)
            .and_then(unicode_path)?;
        let arguments = command
            .arguments
            .iter()
            .map(|value| unicode_os(value).map(str::to_owned))
            .collect::<Result<Vec<_>, _>>()?;
        let environment_keys = command
            .environment
            .iter()
            .map(|(key, _)| unicode_os(key).map(str::to_owned))
            .collect::<Result<Vec<_>, _>>()?;
        let observer_path_exposed = self.contains_observer_reference(&current_directory)
            || arguments
                .iter()
                .any(|value| self.contains_observer_reference(value))
            || command.environment.iter().any(|(key, value)| {
                unicode_os(key)
                    .ok()
                    .is_some_and(|value| self.contains_observer_reference(value))
                    || unicode_os(value)
                        .ok()
                        .is_some_and(|value| self.contains_observer_reference(value))
            });
        Ok(AgentCapabilityAudit {
            current_directory,
            arguments,
            environment_keys,
            inherited_handles: vec![
                "stdin:protocol".to_owned(),
                "stdout:protocol".to_owned(),
                "stderr:bounded-diagnostic".to_owned(),
            ],
            observer_path_exposed,
        })
    }

    fn reject_observer_reference(&self, value: &OsStr) -> Result<(), IsolationError> {
        let value = unicode_os(value)?;
        if self.contains_observer_reference(value) {
            Err(IsolationError::ObserverPathExposure)
        } else {
            Ok(())
        }
    }

    fn contains_observer_reference(&self, value: &str) -> bool {
        let value = normalize(value);
        [
            &self.layout.observer_root,
            &self.layout.telemetry_root,
            &self.layout.index_root,
            &self.layout.report_root,
        ]
        .iter()
        .filter_map(|path| path.to_str())
        .map(normalize)
        .any(|path| value.contains(&path))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObserverArtifactClass {
    Telemetry,
    Index,
    Report,
}

#[derive(Clone, Debug)]
pub struct ObserverAccessDenial {
    pub requested: ObserverArtifactClass,
    pub allowed: bool,
    pub code: &'static str,
    pub derived_track: Track,
    pub reason_code: &'static str,
    pub observed_declaration: TrackDeclaration,
}

fn validate_input_name(value: &str) -> Result<(), IsolationError> {
    if value.is_empty()
        || value.len() > MAX_INPUT_NAME_BYTES
        || value == "."
        || value == ".."
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        Err(IsolationError::InvalidInputName)
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), IsolationError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        Ok(())
    } else {
        Err(IsolationError::InputHashInvalid)
    }
}

fn unicode_path(path: &Path) -> Result<String, IsolationError> {
    unicode_os(path.as_os_str()).map(str::to_owned)
}

fn unicode_os(value: &OsStr) -> Result<&str, IsolationError> {
    value.to_str().ok_or(IsolationError::NonUnicodeLaunchValue)
}

fn normalize(value: &str) -> String {
    value.replace('\\', "/").to_lowercase()
}

fn sha256_file(path: &Path) -> Result<String, IsolationError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let mut encoded = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(&mut encoded, "{byte:02x}").expect("writing to String is infallible");
    }
    Ok(encoded)
}
