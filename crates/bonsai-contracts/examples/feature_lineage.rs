//! Validate actual online proposal events using the existing lineage contract.
use bonsai_contracts::bonsai::artifact::v1::ArtifactLifecycleEvent;
use bonsai_contracts::lineage::validate_artifact_lineage_trace;
use prost::Message;
use serde_json::json;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 || fs::metadata(&args[0])?.len() > 2 * 1024 * 1024 {
        return Err("FEATURE_LINEAGE_INPUT_INVALID".into());
    }
    let encoded: Vec<String> = serde_json::from_slice(&fs::read(&args[0])?)?;
    if encoded.is_empty() || encoded.len() > 4096 {
        return Err("FEATURE_LINEAGE_COUNT_INVALID".into());
    }
    let events = encoded
        .iter()
        .map(|text| {
            if !text.len().is_multiple_of(2) || text.len() > 16384 {
                return Err("FEATURE_LINEAGE_ENCODING_INVALID".into());
            }
            let bytes = text
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| Ok(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?))
                .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
            Ok(ArtifactLifecycleEvent::decode(bytes.as_slice())?)
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    validate_artifact_lineage_trace(&events)?;
    println!("{}", json!({"lineage_events":events.len(),"result":"pass"}));
    Ok(())
}
