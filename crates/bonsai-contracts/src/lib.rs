//! Versioned BONSAI wire contracts and validation.

#![forbid(unsafe_code)]

use prost::Message;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;

#[allow(
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/bonsai.rs"));
}

pub use generated::bonsai;
pub mod adapter;
pub mod inventory;
pub mod lineage;
pub mod resource;
pub mod track;

pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/bonsai-descriptor.bin"));

use bonsai::event::v1::{Availability, EventEnvelope};

#[derive(Debug, Eq, PartialEq)]
pub enum EventValidationError {
    Decode(String),
    InvalidId(&'static str),
    InvalidTime(&'static str),
    InvalidEventType,
    InvalidSchemaVersion,
    InvalidAvailability,
    InvalidPrecision,
    PayloadHashMismatch,
}

impl fmt::Display for EventValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(detail) => write!(formatter, "EVENT_DECODE_FAILED: {detail}"),
            Self::InvalidId(field) => write!(formatter, "EVENT_ID_INVALID: {field}"),
            Self::InvalidTime(field) => write!(formatter, "EVENT_TIME_INVALID: {field}"),
            Self::InvalidEventType => formatter.write_str("EVENT_TYPE_INVALID"),
            Self::InvalidSchemaVersion => formatter.write_str("EVENT_SCHEMA_VERSION_INVALID"),
            Self::InvalidAvailability => formatter.write_str("EVENT_AVAILABILITY_INVALID"),
            Self::InvalidPrecision => formatter.write_str("EVENT_PRECISION_INVALID"),
            Self::PayloadHashMismatch => formatter.write_str("EVENT_PAYLOAD_HASH_MISMATCH"),
        }
    }
}

impl EventValidationError {
    /// Stable machine-oriented validation code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Decode(_) => "EVENT_DECODE_FAILED",
            Self::InvalidId(_) => "EVENT_ID_INVALID",
            Self::InvalidTime(_) => "EVENT_TIME_INVALID",
            Self::InvalidEventType => "EVENT_TYPE_INVALID",
            Self::InvalidSchemaVersion => "EVENT_SCHEMA_VERSION_INVALID",
            Self::InvalidAvailability => "EVENT_AVAILABILITY_INVALID",
            Self::InvalidPrecision => "EVENT_PRECISION_INVALID",
            Self::PayloadHashMismatch => "EVENT_PAYLOAD_HASH_MISMATCH",
        }
    }
}

impl Error for EventValidationError {}

/// Decode and validate the known fields of one event envelope.
///
/// # Errors
///
/// Returns a stable validation error when the bytes are not Protobuf or any
/// required epoch-1 invariant is violated.
pub fn decode_and_validate_event(bytes: &[u8]) -> Result<EventEnvelope, EventValidationError> {
    let event = EventEnvelope::decode(bytes)
        .map_err(|error| EventValidationError::Decode(error.to_string()))?;
    validate_event(&event)?;
    Ok(event)
}

/// Validate known epoch-1 fields and return the original wire bytes unchanged.
///
/// Prost decoding is not the supported relay path because it does not retain
/// unknown fields. Relays use this function and never rebuild the envelope.
///
/// # Errors
///
/// Returns the same errors as [`decode_and_validate_event`].
pub fn relay_validated_event(bytes: &[u8]) -> Result<Vec<u8>, EventValidationError> {
    let _event = decode_and_validate_event(bytes)?;
    Ok(bytes.to_vec())
}

/// Validate all required epoch-1 envelope invariants.
///
/// # Errors
///
/// Returns a stable validation error for an invalid identifier, timestamp,
/// event type, schema version, availability, precision, or payload hash.
pub fn validate_event(event: &EventEnvelope) -> Result<(), EventValidationError> {
    validate_uuid(&event.run_id, "run_id")?;
    validate_uuid(&event.source_id, "source_id")?;
    validate_uuid(&event.event_id, "event_id")?;
    for parent in &event.causal_parent_event_ids {
        validate_uuid(parent, "causal_parent_event_ids")?;
        if parent == &event.event_id {
            return Err(EventValidationError::InvalidId(
                "causal_parent_event_ids contains event_id",
            ));
        }
    }
    if event.monotonic_time_ns == 0 {
        return Err(EventValidationError::InvalidTime("monotonic_time_ns"));
    }
    if event
        .wall_time_unix_ns
        .is_some_and(|timestamp| timestamp <= 0)
    {
        return Err(EventValidationError::InvalidTime("wall_time_unix_ns"));
    }
    if event.event_type.is_empty()
        || !event
            .event_type
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-._/".contains(&byte))
    {
        return Err(EventValidationError::InvalidEventType);
    }
    if event.payload_schema_epoch == 0 {
        return Err(EventValidationError::InvalidSchemaVersion);
    }
    let availability = Availability::try_from(event.availability)
        .map_err(|_| EventValidationError::InvalidAvailability)?;
    if availability == Availability::Unspecified {
        return Err(EventValidationError::InvalidAvailability);
    }
    let precision = event
        .precision
        .as_ref()
        .ok_or(EventValidationError::InvalidPrecision)?;
    if precision.representation.is_empty() || precision.significant_bits == Some(0) {
        return Err(EventValidationError::InvalidPrecision);
    }
    if event.payload_sha256.as_slice() != Sha256::digest(&event.payload).as_slice() {
        return Err(EventValidationError::PayloadHashMismatch);
    }
    Ok(())
}

fn validate_uuid(bytes: &[u8], field: &'static str) -> Result<(), EventValidationError> {
    if bytes.len() != 16 || bytes.iter().all(|byte| *byte == 0) {
        return Err(EventValidationError::InvalidId(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::bonsai::event::v1::{Availability, EventEnvelope, Precision};
    use super::{EventValidationError, relay_validated_event, validate_event};
    use prost::Message;
    use sha2::{Digest, Sha256};

    fn valid_event() -> EventEnvelope {
        let payload = b"fixture-payload".to_vec();
        EventEnvelope {
            run_id: vec![1; 16],
            source_id: vec![2; 16],
            event_id: vec![3; 16],
            source_sequence: 7,
            causal_parent_event_ids: vec![vec![4; 16]],
            monotonic_time_ns: 10,
            wall_time_unix_ns: Some(1_700_000_000_000_000_000),
            event_type: "fixture.event/v1".to_owned(),
            payload_schema_epoch: 1,
            payload_schema_minor: 0,
            payload_sha256: Sha256::digest(&payload).to_vec(),
            availability: Availability::Measured as i32,
            precision: Some(Precision {
                representation: "bytes".to_owned(),
                significant_bits: None,
            }),
            payload,
        }
    }

    #[test]
    fn valid_envelope_round_trips_through_supported_relay() {
        let encoded = valid_event().encode_to_vec();
        assert_eq!(relay_validated_event(&encoded), Ok(encoded));
    }

    #[test]
    fn supported_relay_preserves_unknown_fields_byte_for_byte() {
        let mut encoded = valid_event().encode_to_vec();
        encoded.extend_from_slice(&[0x98, 0x06, 0x07]); // field 99, varint 7
        assert_eq!(relay_validated_event(&encoded), Ok(encoded));
    }

    #[test]
    fn invalid_ids_and_times_fail_closed() {
        let mut event = valid_event();
        event.run_id.clear();
        assert_eq!(
            validate_event(&event),
            Err(EventValidationError::InvalidId("run_id"))
        );

        let mut event = valid_event();
        event.monotonic_time_ns = 0;
        assert_eq!(
            validate_event(&event),
            Err(EventValidationError::InvalidTime("monotonic_time_ns"))
        );

        let mut event = valid_event();
        event.wall_time_unix_ns = Some(-1);
        assert_eq!(
            validate_event(&event),
            Err(EventValidationError::InvalidTime("wall_time_unix_ns"))
        );
    }

    #[test]
    fn payload_hash_is_verified() {
        let mut event = valid_event();
        event.payload.push(0);
        assert_eq!(
            validate_event(&event),
            Err(EventValidationError::PayloadHashMismatch)
        );
    }
}
