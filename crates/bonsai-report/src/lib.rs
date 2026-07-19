//! Deterministic, self-contained reports rendered from machine JSON.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const REPORT_SCHEMA: &str = "bonsai.static-report/v1";

/// Exact report payload shared by the machine JSON and static HTML views.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportData {
    pub schema: String,
    pub title: String,
    pub manifest: Value,
    pub platform: Value,
    pub track: Value,
    pub resources: Value,
    pub overhead: Value,
    pub behavior: Value,
    pub failures: Value,
    pub claims: Value,
    pub limitations: Vec<String>,
    pub hashes: BTreeMap<String, String>,
}

/// Two deterministic representations of the same report data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticReport {
    pub machine_json: String,
    pub html: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReportError {
    Schema,
    Identity,
    MissingSection,
    Hash,
    Serialization,
}

impl fmt::Display for ReportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Schema => "REPORT_SCHEMA_INVALID",
            Self::Identity => "REPORT_IDENTITY_INVALID",
            Self::MissingSection => "REPORT_SECTION_MISSING",
            Self::Hash => "REPORT_HASH_INVALID",
            Self::Serialization => "REPORT_SERIALIZATION_FAILED",
        })
    }
}

impl Error for ReportError {}

/// Generate machine JSON first, then render HTML from its parsed value.
///
/// This sequencing prevents the human and machine representations from using
/// separate calculation paths. The generator performs no metric calculation.
///
/// # Errors
///
/// Rejects malformed identity, missing required sections, non-SHA-256 hashes,
/// and JSON serialization failures.
pub fn generate_static_report(input: &ReportData) -> Result<StaticReport, ReportError> {
    validate(input)?;
    let mut machine_json =
        serde_json::to_string_pretty(input).map_err(|_| ReportError::Serialization)?;
    machine_json.push('\n');
    let parsed: ReportData =
        serde_json::from_str(&machine_json).map_err(|_| ReportError::Serialization)?;
    let html = render_html(&parsed);
    Ok(StaticReport { machine_json, html })
}

fn validate(input: &ReportData) -> Result<(), ReportError> {
    if input.schema != REPORT_SCHEMA {
        return Err(ReportError::Schema);
    }
    if input.title.trim().is_empty() || input.limitations.is_empty() || input.hashes.is_empty() {
        return Err(ReportError::Identity);
    }
    for section in [
        &input.manifest,
        &input.platform,
        &input.track,
        &input.resources,
        &input.overhead,
        &input.behavior,
        &input.failures,
        &input.claims,
    ] {
        if section.is_null() {
            return Err(ReportError::MissingSection);
        }
    }
    if input.hashes.iter().any(|(name, hash)| {
        name.trim().is_empty()
            || hash.len() != 64
            || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
    }) {
        return Err(ReportError::Hash);
    }
    Ok(())
}

fn render_html(data: &ReportData) -> String {
    let mut output = String::from(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<title>",
    );
    output.push_str(&escape_html(&data.title));
    output.push_str(
        "</title><style>body{font-family:system-ui,sans-serif;max-width:80rem;margin:auto;padding:1rem}\
table{border-collapse:collapse;width:100%}caption{text-align:left;font-weight:700}\
th,td{border:1px solid #777;padding:.35rem;text-align:left;vertical-align:top}\
th{width:32%}code{white-space:pre-wrap;overflow-wrap:anywhere}</style></head><body><main><h1>",
    );
    output.push_str(&escape_html(&data.title));
    output.push_str("</h1>");

    let sections = [
        ("manifest", "Manifest", &data.manifest),
        ("platform", "Platform", &data.platform),
        ("track", "Track", &data.track),
        ("resources", "Resources", &data.resources),
        ("overhead", "Overhead", &data.overhead),
        ("behavior", "Behavior", &data.behavior),
        ("failures", "Failures", &data.failures),
        ("claims", "Claims", &data.claims),
    ];
    for (id, title, value) in sections {
        render_section(&mut output, id, title, value);
    }
    render_section(
        &mut output,
        "limitations",
        "Limitations",
        &serde_json::to_value(&data.limitations).expect("serializable limitations"),
    );
    render_section(
        &mut output,
        "hashes",
        "Hashes",
        &serde_json::to_value(&data.hashes).expect("serializable hashes"),
    );
    output.push_str("</main></body></html>\n");
    output
}

fn render_section(output: &mut String, id: &str, title: &str, value: &Value) {
    output.push_str("<section aria-labelledby=\"");
    output.push_str(id);
    output.push_str("-heading\"><h2 id=\"");
    output.push_str(id);
    output.push_str("-heading\">");
    output.push_str(title);
    output.push_str("</h2><table><caption>");
    output.push_str(title);
    output.push_str(" machine values</caption><thead><tr><th scope=\"col\">Field</th><th scope=\"col\">Value</th></tr></thead><tbody>");
    let mut rows = Vec::new();
    flatten_value("$", value, &mut rows);
    for (path, rendered) in rows {
        output.push_str("<tr><th scope=\"row\"><code>");
        output.push_str(&escape_html(&path));
        output.push_str("</code></th><td><code>");
        output.push_str(&escape_html(&rendered));
        output.push_str("</code></td></tr>");
    }
    output.push_str("</tbody></table></section>");
}

fn flatten_value(path: &str, value: &Value, output: &mut Vec<(String, String)>) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (name, child) in map {
                flatten_value(&format!("{path}.{name}"), child, output);
            }
        }
        Value::Array(values) if !values.is_empty() => {
            for (index, child) in values.iter().enumerate() {
                flatten_value(&format!("{path}[{index}]"), child, output);
            }
        }
        Value::String(text) => output.push((path.to_owned(), text.clone())),
        _ => output.push((path.to_owned(), value.to_string())),
    }
}

fn escape_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{ReportData, ReportError, generate_static_report};
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn fixture() -> ReportData {
        ReportData {
            schema: "bonsai.static-report/v1".to_owned(),
            title: "M1 <heartbeat>".to_owned(),
            manifest: json!({"bundle_id": "fixture-1", "seed": 7}),
            platform: json!({"os": "fixture-os", "counter": "unavailable"}),
            track: json!({"derived": "A", "reason": "STRICT_EXPERIENTIAL_FACTS"}),
            resources: json!({"cpu_ns": 125, "energy": null}),
            overhead: json!({"throughput_ppm": 1250, "latency_ppm": 2100}),
            behavior: json!({"reward": [1, 2, 3], "rate": "2/1"}),
            failures: json!([]),
            claims: json!({"C0": "indeterminate", "C1": "indeterminate"}),
            limitations: vec!["diagnostic fixture; no physical counters".to_owned()],
            hashes: BTreeMap::from([("manifest.json".to_owned(), HASH.to_owned())]),
        }
    }

    #[test]
    fn report_is_offline_self_contained_and_accessible() {
        let report = generate_static_report(&fixture()).expect("report");
        let lowercase = report.html.to_ascii_lowercase();
        for prohibited in ["<script", "<link", " src=", " href=", "url("] {
            assert!(!lowercase.contains(prohibited), "found {prohibited}");
        }
        for required in [
            "<!doctype html>",
            "<html lang=\"en\">",
            "<title>",
            "<main>",
            "<h1>",
            "<h2 id=\"manifest-heading\">",
            "<caption>",
            "<th scope=\"col\">",
            "<th scope=\"row\">",
        ] {
            assert!(report.html.contains(required), "missing {required}");
        }
        assert!(report.html.contains("M1 &lt;heartbeat&gt;"));
        assert!(!report.html.contains("M1 <heartbeat>"));
    }

    #[test]
    fn every_displayed_leaf_reconciles_with_machine_json() {
        let report = generate_static_report(&fixture()).expect("report");
        let parsed: ReportData = serde_json::from_str(&report.machine_json).expect("machine JSON");
        assert_eq!(parsed, fixture());
        let serialized: Value = serde_json::from_str(&report.machine_json).expect("JSON value");
        for field in [
            "manifest",
            "platform",
            "track",
            "resources",
            "overhead",
            "behavior",
            "failures",
            "claims",
            "limitations",
            "hashes",
        ] {
            assert!(serialized.get(field).is_some(), "missing {field}");
        }
        for expected in [
            "fixture-1",
            "fixture-os",
            "STRICT_EXPERIENTIAL_FACTS",
            "125",
            "1250",
            "2/1",
            "indeterminate",
            HASH,
        ] {
            assert!(report.html.contains(expected), "missing value {expected}");
        }
    }

    #[test]
    fn incomplete_or_malformed_inputs_fail_closed() {
        let mut input = fixture();
        input.claims = Value::Null;
        assert_eq!(
            generate_static_report(&input),
            Err(ReportError::MissingSection)
        );
        input = fixture();
        input
            .hashes
            .insert("bad".to_owned(), "not-a-hash".to_owned());
        assert_eq!(generate_static_report(&input), Err(ReportError::Hash));
    }
}
