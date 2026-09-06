//! Explicit recovery derivative for the governed runner's current single segment.
use bonsai_bundle::{put_blob_stream, rebuild_index, salvage_segment, visit_segment_file};
use bonsai_contracts::decode_and_validate_event;
use serde_json::json;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

pub(super) fn run(args: &[OsString]) -> Result<i32, String> {
    let [root_flag, root, output_flag, output, quota_flag, quota] = args else {
        return Err("recover-run requires --root --output --maximum-output-bytes".into());
    };
    if root_flag != "--root" || output_flag != "--output" || quota_flag != "--maximum-output-bytes"
    {
        return Err("recover-run argument order invalid".into());
    }
    let maximum = quota
        .to_str()
        .ok_or("quota encoding")?
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    recover(&PathBuf::from(root), &PathBuf::from(output), maximum)
}

fn recover(root: &Path, output: &Path, maximum: u64) -> Result<i32, String> {
    reject_link(root)?;
    let telemetry = root.join("telemetry");
    reject_link(&telemetry)?;
    let open = telemetry.join("segment-00000000000000000000.open");
    let finalized = telemetry.join("segment-00000000000000000000.bseg");
    let source = if open.exists() { open } else { finalized };
    for entry in fs::read_dir(&telemetry).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        if name != "segment-00000000000000000000.open"
            && name != "segment-00000000000000000000.bseg"
        {
            return Err("RUN_RECOVERY_LAYOUT_UNSUPPORTED".into());
        }
        reject_link(&entry.path())?;
    }
    reject_link(&source)?;
    let metadata = fs::metadata(&source).map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err("RUN_RECOVERY_SOURCE_INVALID".into());
    }
    let reserve = 4 * 1024 * 1024;
    if metadata
        .len()
        .checked_mul(3)
        .and_then(|v| v.checked_add(reserve))
        .is_none_or(|v| v > maximum)
    {
        return Err("RUN_RECOVERY_OUTPUT_QUOTA".into());
    }
    fs::create_dir(output).map_err(|e| e.to_string())?;
    let status = output.join("recovery.json");
    fs::write(
        &status,
        b"{\"status\":\"INCOMPLETE\",\"uninterrupted_track_a_eligible\":false}\n",
    )
    .map_err(|e| e.to_string())?;
    let captured = put_blob_stream(
        output,
        &mut File::open(&source).map_err(|e| e.to_string())?,
        metadata.len(),
    )
    .map_err(|e| e.to_string())?;
    if captured.byte_length != metadata.len() {
        return Err("RUN_RECOVERY_SOURCE_CHANGED".into());
    }
    let saved = output.join(&captured.relative_path);
    let mut header = [0_u8; 24];
    File::open(&saved)
        .map_err(|e| e.to_string())?
        .read_exact(&mut header)
        .map_err(|e| e.to_string())?;
    let maximum_frame = u32::from_le_bytes(header[20..24].try_into().map_err(|_| "header")?);
    if maximum_frame > 256 * 1024 {
        return Err("RUN_RECOVERY_FRAME_LIMIT".into());
    }
    let salvage = salvage_segment(&saved, output, maximum_frame).map_err(|e| e.to_string())?;
    let target = output.join("segment-00000000000000000000.bseg");
    let validated = validate_events(&target)?;
    if validated != salvage.summary.frame_count {
        return Err("RUN_RECOVERY_EVENT_COUNT".into());
    }
    let index = rebuild_index(output).map_err(|e| e.to_string())?;
    let report = json!({
        "format":"bonsai.run-recovery/v1","status":"INCOMPLETE","recovery_status":"RECOVERED_PREFIX",
        "continuation":true,"uninterrupted_track_a_eligible":false,
        "agent_state_restored":false,"source_capture_sha256":captured.id.to_hex(),
        "source_bytes":captured.byte_length,"recovered_events":validated,
        "truncated_tail":salvage.truncated_tail,"source_preserved":true,
        "segment_count":index.segment_count,"output_limit_bytes":maximum,
        "scope":"checksummed observer prefix only; no resumed agent or C0/C1 acceptance"
    });
    let encoded = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    fs::write(status, &encoded).map_err(|e| e.to_string())?;
    println!("{report}");
    Ok(0)
}

fn validate_events(path: &Path) -> Result<u64, String> {
    let mut sequence = 0_u64;
    let mut previous = None;
    let mut run = None;
    let mut failure = None;
    visit_segment_file(path, |bytes| {
        if failure.is_some() {
            return;
        }
        let result = (|| {
            let event = decode_and_validate_event(bytes).map_err(|e| e.to_string())?;
            if event.source_id != [42; 16]
                || event.source_sequence != sequence
                || run.as_ref().is_some_and(|id| id != &event.run_id)
                || event.causal_parent_event_ids != previous.iter().cloned().collect::<Vec<_>>()
            {
                return Err("RUN_RECOVERY_EVENT_ORDER".to_owned());
            }
            run = Some(event.run_id);
            previous = Some(event.event_id);
            sequence += 1;
            Ok(())
        })();
        if let Err(error) = result {
            failure = Some(error);
        }
    })
    .map_err(|e| e.to_string())?;
    if let Some(error) = failure {
        return Err(error);
    }
    Ok(sequence)
}

fn reject_link(path: &Path) -> Result<(), String> {
    if fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("RUN_RECOVERY_SYMLINK_REJECTED".into());
    }
    Ok(())
}
