//! Frozen schema-compatibility conformance suite.

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const CATALOG_SCHEMA: &str = "bonsai.schema-catalog/v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema: String,
    epoch: u64,
    minor: u64,
    protobuf_messages: Vec<ProtoMessage>,
    json_schemas: Vec<JsonSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtoMessage {
    name: String,
    fields: Vec<ProtoField>,
    #[serde(default)]
    reserved_numbers: Vec<u32>,
    #[serde(default)]
    reserved_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtoField {
    number: u32,
    name: String,
    #[serde(rename = "type")]
    field_type: String,
    presence: String,
    unit: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonSchema {
    name: String,
    uri: String,
    version: Option<String>,
    properties: Vec<JsonProperty>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonProperty {
    name: String,
    #[serde(rename = "type")]
    property_type: String,
    required: bool,
    unit: Option<String>,
}

#[derive(Debug)]
struct CompatibilityError {
    code: &'static str,
    path: String,
    detail: String,
}

struct FixtureCase {
    file: &'static str,
    compatible: bool,
    expected_error: Option<&'static str>,
}

pub(crate) fn run() -> Result<(), String> {
    let root = workspace_root();
    let fixture_dir = root.join("fixtures/schema-compatibility/v1");
    let baseline = load_catalog(&fixture_dir.join("baseline.json"))?;
    let baseline_errors = validate_catalog(&baseline);
    if !baseline_errors.is_empty() {
        return Err(format_errors(
            "baseline fixture is invalid",
            &baseline_errors,
        ));
    }

    let cases = [
        FixtureCase {
            file: "additive.json",
            compatible: true,
            expected_error: None,
        },
        FixtureCase {
            file: "field-renumbering.json",
            compatible: false,
            expected_error: Some("FIELD_RENUMBERED"),
        },
        FixtureCase {
            file: "field-reuse.json",
            compatible: false,
            expected_error: Some("FIELD_REUSE"),
        },
        FixtureCase {
            file: "silent-unit-change.json",
            compatible: false,
            expected_error: Some("UNIT_CHANGED"),
        },
        FixtureCase {
            file: "unversioned-json.json",
            compatible: false,
            expected_error: Some("JSON_VERSION_MISSING"),
        },
    ];

    for case in cases {
        let path = fixture_dir.join(case.file);
        let raw = fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let candidate = parse_catalog(&path, &raw)?;
        let mut errors = validate_catalog(&candidate);
        errors.extend(compare_catalogs(&baseline, &candidate));
        let observed_compatible = errors.is_empty();
        if observed_compatible != case.compatible {
            return Err(format_errors(
                &format!(
                    "{} expected compatible={} but observed compatible={observed_compatible}",
                    case.file, case.compatible
                ),
                &errors,
            ));
        }
        if let Some(expected) = case.expected_error
            && !errors.iter().any(|error| error.code == expected)
        {
            return Err(format_errors(
                &format!("{} did not produce expected error {expected}", case.file),
                &errors,
            ));
        }
        let digest = Sha256::digest(canonical_json(&raw)?);
        let mut digest_hex = String::with_capacity(64);
        for byte in digest {
            write!(digest_hex, "{byte:02x}")
                .map_err(|error| format!("encode canonical digest: {error}"))?;
        }
        println!(
            "schema fixture {}: {} canonical_sha256={digest_hex}",
            case.file,
            if observed_compatible {
                "compatible"
            } else {
                case.expected_error.unwrap_or("incompatible")
            }
        );
    }

    println!("schema compatibility check passed: 1 additive and 4 rejection fixtures");
    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}

fn load_catalog(path: &Path) -> Result<Catalog, String> {
    let raw = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    parse_catalog(path, &raw)
}

fn parse_catalog(path: &Path, raw: &[u8]) -> Result<Catalog, String> {
    serde_json::from_slice(raw).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn validate_catalog(catalog: &Catalog) -> Vec<CompatibilityError> {
    let mut errors = Vec::new();
    if catalog.schema != CATALOG_SCHEMA {
        push_error(
            &mut errors,
            "CATALOG_VERSION_INVALID",
            "schema",
            format!("expected {CATALOG_SCHEMA}"),
        );
    }
    if catalog.epoch == 0 {
        push_error(
            &mut errors,
            "EPOCH_INVALID",
            "epoch",
            "epoch must be at least 1",
        );
    }

    let mut message_names = HashSet::new();
    for message in &catalog.protobuf_messages {
        if !message_names.insert(message.name.as_str()) {
            push_error(
                &mut errors,
                "DUPLICATE_MESSAGE",
                format!("protobuf_messages.{}", message.name),
                "message names must be unique",
            );
        }
        validate_message(message, &mut errors);
    }

    let mut schema_names = HashSet::new();
    for schema in &catalog.json_schemas {
        if !schema_names.insert(schema.name.as_str()) {
            push_error(
                &mut errors,
                "DUPLICATE_JSON_SCHEMA",
                format!("json_schemas.{}", schema.name),
                "JSON schema names must be unique",
            );
        }
        validate_json_schema(catalog, schema, &mut errors);
    }
    errors
}

fn validate_message(message: &ProtoMessage, errors: &mut Vec<CompatibilityError>) {
    let reserved_numbers = message
        .reserved_numbers
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let reserved_names = message
        .reserved_names
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut numbers = HashSet::new();
    let mut names = HashSet::new();
    for field in &message.fields {
        let path = format!("protobuf_messages.{}.{}", message.name, field.name);
        if field.number == 0 || (19_000..=19_999).contains(&field.number) {
            push_error(
                errors,
                "FIELD_NUMBER_INVALID",
                &path,
                "field number is zero or in the Protobuf implementation-reserved range",
            );
        }
        if !numbers.insert(field.number) || !names.insert(field.name.as_str()) {
            push_error(
                errors,
                "DUPLICATE_FIELD",
                &path,
                "field names and numbers must be unique within a message",
            );
        }
        if reserved_numbers.contains(&field.number) || reserved_names.contains(field.name.as_str())
        {
            push_error(
                errors,
                "FIELD_REUSE",
                &path,
                "field uses a reserved number or name",
            );
        }
        if is_numeric_type(&field.field_type) && field.unit.is_none() {
            push_error(
                errors,
                "UNIT_MISSING",
                &path,
                "numeric fields require an explicit unit",
            );
        }
    }
}

fn validate_json_schema(
    catalog: &Catalog,
    schema: &JsonSchema,
    errors: &mut Vec<CompatibilityError>,
) {
    let path = format!("json_schemas.{}", schema.name);
    let expected_version = format!("{}.{}", catalog.epoch, catalog.minor);
    match &schema.version {
        Some(version) if version == &expected_version => {}
        Some(version) => push_error(
            errors,
            "JSON_VERSION_MISMATCH",
            &path,
            format!("version {version} does not match catalog {expected_version}"),
        ),
        None => push_error(
            errors,
            "JSON_VERSION_MISSING",
            &path,
            "JSON schemas require an explicit epoch.minor version",
        ),
    }
    let epoch_segment = format!("/v{}", catalog.epoch);
    if !schema.uri.ends_with(&epoch_segment) && !schema.uri.contains(&format!("{epoch_segment}/")) {
        push_error(
            errors,
            "JSON_URI_UNVERSIONED",
            &path,
            format!("URI must contain the epoch segment {epoch_segment}"),
        );
    }

    let mut names = HashSet::new();
    for property in &schema.properties {
        let property_path = format!("{path}.{}", property.name);
        if !names.insert(property.name.as_str()) {
            push_error(
                errors,
                "DUPLICATE_JSON_PROPERTY",
                &property_path,
                "property names must be unique",
            );
        }
        if matches!(property.property_type.as_str(), "integer" | "number")
            && property.unit.is_none()
        {
            push_error(
                errors,
                "UNIT_MISSING",
                &property_path,
                "numeric properties require an explicit unit",
            );
        }
    }
}

fn compare_catalogs(baseline: &Catalog, candidate: &Catalog) -> Vec<CompatibilityError> {
    let mut errors = Vec::new();
    if candidate.epoch != baseline.epoch {
        push_error(
            &mut errors,
            "EPOCH_CHANGE_REQUIRES_MIGRATION",
            "epoch",
            "minor compatibility checks require an unchanged epoch",
        );
        return errors;
    }
    if candidate.minor < baseline.minor {
        push_error(
            &mut errors,
            "MINOR_VERSION_REGRESSED",
            "minor",
            "minor version cannot decrease within an epoch",
        );
    }

    let candidate_messages = candidate
        .protobuf_messages
        .iter()
        .map(|message| (message.name.as_str(), message))
        .collect::<HashMap<_, _>>();
    for old_message in &baseline.protobuf_messages {
        let Some(new_message) = candidate_messages.get(old_message.name.as_str()) else {
            push_error(
                &mut errors,
                "MESSAGE_REMOVED",
                format!("protobuf_messages.{}", old_message.name),
                "messages cannot be removed within an epoch",
            );
            continue;
        };
        compare_messages(old_message, new_message, &mut errors);
    }

    let candidate_schemas = candidate
        .json_schemas
        .iter()
        .map(|schema| (schema.name.as_str(), schema))
        .collect::<HashMap<_, _>>();
    for old_schema in &baseline.json_schemas {
        let Some(new_schema) = candidate_schemas.get(old_schema.name.as_str()) else {
            push_error(
                &mut errors,
                "JSON_SCHEMA_REMOVED",
                format!("json_schemas.{}", old_schema.name),
                "JSON schemas cannot be removed within an epoch",
            );
            continue;
        };
        compare_json_schemas(old_schema, new_schema, &mut errors);
    }
    errors
}

fn compare_messages(old: &ProtoMessage, new: &ProtoMessage, errors: &mut Vec<CompatibilityError>) {
    let old_by_name = old
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect::<HashMap<_, _>>();
    let old_by_number = old
        .fields
        .iter()
        .map(|field| (field.number, field))
        .collect::<HashMap<_, _>>();
    let new_by_name = new
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect::<HashMap<_, _>>();

    for field in &new.fields {
        let path = format!("protobuf_messages.{}.{}", new.name, field.name);
        if let Some(prior) = old_by_name.get(field.name.as_str()) {
            if field.number != prior.number {
                push_error(
                    errors,
                    "FIELD_RENUMBERED",
                    &path,
                    format!(
                        "field number changed from {} to {}",
                        prior.number, field.number
                    ),
                );
            }
            compare_field_semantics(prior, field, &path, errors);
        }
        if let Some(prior) = old_by_number.get(&field.number)
            && field.name != prior.name
        {
            push_error(
                errors,
                "FIELD_REUSE",
                &path,
                format!(
                    "field number {} previously belonged to {}",
                    field.number, prior.name
                ),
            );
        }
    }

    for field in &old.fields {
        if !new_by_name.contains_key(field.name.as_str()) {
            let number_reserved = new.reserved_numbers.contains(&field.number);
            let name_reserved = new.reserved_names.contains(&field.name);
            if !number_reserved || !name_reserved {
                push_error(
                    errors,
                    "DELETION_NOT_RESERVED",
                    format!("protobuf_messages.{}.{}", new.name, field.name),
                    "deleted fields must reserve both their number and name",
                );
            }
        }
    }
}

fn compare_field_semantics(
    old: &ProtoField,
    new: &ProtoField,
    path: &str,
    errors: &mut Vec<CompatibilityError>,
) {
    if old.field_type != new.field_type || old.presence != new.presence {
        push_error(
            errors,
            "FIELD_TYPE_CHANGED",
            path,
            "field type or presence changed within an epoch",
        );
    }
    if old.unit != new.unit {
        push_error(
            errors,
            "UNIT_CHANGED",
            path,
            "field unit changed without an epoch migration",
        );
    }
}

fn compare_json_schemas(old: &JsonSchema, new: &JsonSchema, errors: &mut Vec<CompatibilityError>) {
    let old_properties = old
        .properties
        .iter()
        .map(|property| (property.name.as_str(), property))
        .collect::<HashMap<_, _>>();
    let new_properties = new
        .properties
        .iter()
        .map(|property| (property.name.as_str(), property))
        .collect::<HashMap<_, _>>();
    for property in &old.properties {
        let path = format!("json_schemas.{}.{}", old.name, property.name);
        let Some(candidate) = new_properties.get(property.name.as_str()) else {
            push_error(
                errors,
                "JSON_PROPERTY_REMOVED",
                &path,
                "properties cannot be removed within an epoch",
            );
            continue;
        };
        if property.property_type != candidate.property_type {
            push_error(
                errors,
                "JSON_TYPE_CHANGED",
                &path,
                "property type changed within an epoch",
            );
        }
        if !property.required && candidate.required {
            push_error(
                errors,
                "JSON_REQUIRED_ADDED",
                &path,
                "an existing optional property cannot become required",
            );
        }
        if property.unit != candidate.unit {
            push_error(
                errors,
                "UNIT_CHANGED",
                &path,
                "property unit changed without an epoch migration",
            );
        }
    }
    for property in &new.properties {
        if !old_properties.contains_key(property.name.as_str()) && property.required {
            push_error(
                errors,
                "JSON_REQUIRED_ADDED",
                format!("json_schemas.{}.{}", new.name, property.name),
                "new properties must be optional within an epoch",
            );
        }
    }
}

fn is_numeric_type(field_type: &str) -> bool {
    matches!(
        field_type,
        "double"
            | "float"
            | "int32"
            | "int64"
            | "uint32"
            | "uint64"
            | "sint32"
            | "sint64"
            | "fixed32"
            | "fixed64"
            | "sfixed32"
            | "sfixed64"
    )
}

fn canonical_json(raw: &[u8]) -> Result<Vec<u8>, String> {
    let value: Value = serde_json::from_slice(raw)
        .map_err(|error| format!("parse JSON before canonicalization: {error}"))?;
    let mut output = Vec::new();
    write_canonical(&value, &mut output)?;
    Ok(output)
}

fn write_canonical(value: &Value, output: &mut Vec<u8>) -> Result<(), String> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(true) => output.extend_from_slice(b"true"),
        Value::Bool(false) => output.extend_from_slice(b"false"),
        Value::Number(number) => output.extend_from_slice(number.to_string().as_bytes()),
        Value::String(text) => output.extend_from_slice(
            serde_json::to_string(text)
                .map_err(|error| format!("canonicalize string: {error}"))?
                .as_bytes(),
        ),
        Value::Array(values) => {
            output.push(b'[');
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical(item, output)?;
            }
            output.push(b']');
        }
        Value::Object(properties) => {
            output.push(b'{');
            let mut keys = properties.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                output.extend_from_slice(
                    serde_json::to_string(key)
                        .map_err(|error| format!("canonicalize object key: {error}"))?
                        .as_bytes(),
                );
                output.push(b':');
                write_canonical(&properties[key], output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn push_error(
    errors: &mut Vec<CompatibilityError>,
    code: &'static str,
    path: impl Into<String>,
    detail: impl Into<String>,
) {
    errors.push(CompatibilityError {
        code,
        path: path.into(),
        detail: detail.into(),
    });
}

fn format_errors(context: &str, errors: &[CompatibilityError]) -> String {
    let mut output = context.to_owned();
    for error in errors {
        let _ = write!(
            output,
            "\n- {} at {}: {}",
            error.code, error.path, error.detail
        );
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{canonical_json, run};

    #[test]
    fn canonical_json_sorts_object_keys_without_reordering_arrays() {
        assert_eq!(
            canonical_json(br#"{ "z": [2, 1], "a": {"b": true, "a": null} }"#)
                .expect("fixture is valid JSON"),
            br#"{"a":{"a":null,"b":true},"z":[2,1]}"#
        );
    }

    #[test]
    fn frozen_compatibility_suite_has_expected_outcomes() {
        run().expect("frozen schema compatibility suite must pass");
    }
}
