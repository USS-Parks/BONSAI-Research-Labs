//! Read-only, path-contained access to local report bundles.

use crate::{ReportData, StaticReport, generate_static_report};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewerError {
    Root,
    UnsafePath,
    NotFile,
    Io,
    ReportJson,
    ReportMismatch,
}

impl fmt::Display for ViewerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Root => "VIEWER_ROOT_INVALID",
            Self::UnsafePath => "VIEWER_PATH_UNSAFE",
            Self::NotFile => "VIEWER_PATH_NOT_FILE",
            Self::Io => "VIEWER_READ_FAILED",
            Self::ReportJson => "VIEWER_REPORT_JSON_INVALID",
            Self::ReportMismatch => "VIEWER_REPORT_MISMATCH",
        })
    }
}

impl Error for ViewerError {}

/// Canonical bundle root used exclusively for read-only file access.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadOnlyBundleViewer {
    root: PathBuf,
}

impl ReadOnlyBundleViewer {
    /// Open and canonicalize an existing bundle directory.
    ///
    /// # Errors
    ///
    /// Rejects missing roots, files, and roots that cannot be canonicalized.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ViewerError> {
        let root = fs::canonicalize(root).map_err(|_| ViewerError::Root)?;
        if !fs::metadata(&root).map_err(|_| ViewerError::Root)?.is_dir() {
            return Err(ViewerError::Root);
        }
        Ok(Self { root })
    }

    /// Read one existing regular file contained by the canonical bundle root.
    ///
    /// # Errors
    ///
    /// Rejects absolute paths, parent/current components, symlink escapes,
    /// directories, missing files, and I/O failures.
    pub fn read(&self, relative: impl AsRef<Path>) -> Result<Vec<u8>, ViewerError> {
        let relative = relative.as_ref();
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(ViewerError::UnsafePath);
        }
        let resolved = fs::canonicalize(self.root.join(relative)).map_err(|_| ViewerError::Io)?;
        if !resolved.starts_with(&self.root) {
            return Err(ViewerError::UnsafePath);
        }
        if !fs::metadata(&resolved)
            .map_err(|_| ViewerError::Io)?
            .is_file()
        {
            return Err(ViewerError::NotFile);
        }
        fs::read(resolved).map_err(|_| ViewerError::Io)
    }

    /// Load `report.json` and require `report.html` to equal its regeneration.
    ///
    /// # Errors
    ///
    /// Returns a stable error for unreadable JSON, invalid report data, or an
    /// HTML file that differs from the canonical static generator output.
    pub fn load_static_report(&self) -> Result<StaticReport, ViewerError> {
        let machine_bytes = self.read("report.json")?;
        let data: ReportData =
            serde_json::from_slice(&machine_bytes).map_err(|_| ViewerError::ReportJson)?;
        let generated = generate_static_report(&data).map_err(|_| ViewerError::ReportJson)?;
        let stored_html = self.read("report.html")?;
        if generated.machine_json.as_bytes() != machine_bytes
            || generated.html.as_bytes() != stored_html
        {
            return Err(ViewerError::ReportMismatch);
        }
        Ok(generated)
    }
}

#[cfg(test)]
mod tests {
    use super::{ReadOnlyBundleViewer, ViewerError};
    use crate::{ReportData, generate_static_report};
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    const HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn fixture() -> ReportData {
        ReportData {
            schema: "bonsai.static-report/v1".to_owned(),
            title: "Viewer fixture".to_owned(),
            manifest: json!({"bundle_id": "viewer-fixture"}),
            platform: json!({"os": "fixture-os"}),
            track: json!({"derived": "A"}),
            resources: json!({"cpu_ns": 125}),
            overhead: json!({"throughput_ppm": 50}),
            behavior: json!({"reward": 9}),
            failures: json!([]),
            claims: json!({"C0": "indeterminate"}),
            limitations: vec!["fixture only".to_owned()],
            hashes: BTreeMap::from([("manifest.json".to_owned(), HASH.to_owned())]),
        }
    }

    fn bundle() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("bonsai-viewer-{}-{unique}", std::process::id()));
        fs::create_dir(&root).expect("create bundle");
        let report = generate_static_report(&fixture()).expect("report");
        fs::write(root.join("report.json"), report.machine_json).expect("machine report");
        fs::write(root.join("report.html"), report.html).expect("HTML report");
        for (name, value) in [
            ("lineage.json", json!({"parents": ["root"]})),
            ("metrics.json", json!({"reward": 9})),
            ("decisions.json", json!({"budget": "continue"})),
            ("comparisons.json", json!({"baseline": "primitive"})),
        ] {
            fs::write(root.join(name), serde_json::to_vec(&value).expect("JSON"))
                .expect("viewer fixture");
        }
        root
    }

    fn modified(path: &Path) -> SystemTime {
        fs::metadata(path)
            .expect("metadata")
            .modified()
            .expect("modified")
    }

    #[test]
    fn browsing_is_read_only_and_report_values_are_identical() {
        let root = bundle();
        let watched = [
            "report.json",
            "report.html",
            "lineage.json",
            "metrics.json",
            "decisions.json",
            "comparisons.json",
        ];
        let before = watched
            .iter()
            .map(|path| {
                (
                    path.to_string(),
                    fs::read(root.join(path)).expect("before bytes"),
                    modified(&root.join(path)),
                )
            })
            .collect::<Vec<_>>();
        let viewer = ReadOnlyBundleViewer::open(&root).expect("viewer");
        let report = viewer.load_static_report().expect("canonical report");
        assert_eq!(
            serde_json::from_str::<ReportData>(&report.machine_json).expect("report data"),
            fixture()
        );
        for path in &watched[2..] {
            assert_eq!(
                viewer.read(path).expect("browse"),
                fs::read(root.join(path)).expect("source")
            );
        }
        for (path, bytes, timestamp) in before {
            assert_eq!(fs::read(root.join(&path)).expect("after bytes"), bytes);
            assert_eq!(modified(&root.join(path)), timestamp);
        }
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn traversal_absolute_paths_and_report_drift_fail_closed() {
        let root = bundle();
        let viewer = ReadOnlyBundleViewer::open(&root).expect("viewer");
        assert_eq!(viewer.read("../outside.json"), Err(ViewerError::UnsafePath));
        assert_eq!(
            viewer.read(root.join("report.json")),
            Err(ViewerError::UnsafePath)
        );
        fs::write(root.join("report.html"), "drift").expect("introduce drift");
        assert_eq!(
            viewer.load_static_report(),
            Err(ViewerError::ReportMismatch)
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = bundle();
        let outside = root.with_extension("outside");
        fs::write(&outside, "outside").expect("outside file");
        symlink(&outside, root.join("escape.json")).expect("symlink");
        let viewer = ReadOnlyBundleViewer::open(&root).expect("viewer");
        assert_eq!(viewer.read("escape.json"), Err(ViewerError::UnsafePath));
        fs::remove_dir_all(root).expect("remove fixture");
        fs::remove_file(outside).expect("remove outside fixture");
    }
}
