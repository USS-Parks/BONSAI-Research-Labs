//! Lifelong comparison tables with uncertainty labels (BV-08).

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CurvePoint {
    pub step: u64,
    pub value: i64,
    pub lower: i64,
    pub upper: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonTable {
    pub schema: String,
    pub title: String,
    pub rows: Vec<ComparisonRow>,
    pub hidden_failed_seeds: bool,
    pub truncated_axes: bool,
    pub incomparable_overlay: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonRow {
    pub series_id: String,
    pub curve: Vec<CurvePoint>,
    pub worst_window: i64,
    pub resource_front: u64,
    pub suppressed_work: u64,
    pub uncertainty_label: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompareError {
    Identity,
    Uncertainty,
    HiddenFailure,
}

impl fmt::Display for CompareError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Identity => "COMPARE_IDENTITY_INVALID",
            Self::Uncertainty => "COMPARE_UNCERTAINTY_MISSING",
            Self::HiddenFailure => "COMPARE_HIDDEN_FAILURE",
        })
    }
}

impl Error for CompareError {}

/// Build an accessible comparison table. Hidden seeds and truncated axes fail.
///
/// # Errors
///
/// Rejects empty series, missing uncertainty, or hidden-failure flags.
pub fn comparison_table(
    title: &str,
    rows: Vec<ComparisonRow>,
    hidden_failed_seeds: bool,
    truncated_axes: bool,
    incomparable_overlay: bool,
) -> Result<ComparisonTable, CompareError> {
    if title.is_empty() || rows.is_empty() {
        return Err(CompareError::Identity);
    }
    if hidden_failed_seeds || truncated_axes || incomparable_overlay {
        return Err(CompareError::HiddenFailure);
    }
    if rows.iter().any(|row| {
        row.series_id.is_empty()
            || row.curve.is_empty()
            || row.uncertainty_label.is_empty()
            || row.curve.iter().any(|point| point.lower > point.upper)
    }) {
        return Err(CompareError::Uncertainty);
    }
    Ok(ComparisonTable {
        schema: "bonsai.comparison-table/v1".to_owned(),
        title: title.to_owned(),
        rows,
        hidden_failed_seeds: false,
        truncated_axes: false,
        incomparable_overlay: false,
    })
}

/// Render a captioned HTML table that matches the machine rows.
#[must_use]
pub fn comparison_html(table: &ComparisonTable) -> String {
    let mut html = String::from(
        "<table><caption>Uncertainty-labeled comparison</caption>\
<thead><tr><th scope=\"col\">Series</th><th scope=\"col\">Worst window</th>\
<th scope=\"col\">Resource front</th><th scope=\"col\">Suppressed work</th>\
<th scope=\"col\">Uncertainty</th></tr></thead><tbody>",
    );
    for row in &table.rows {
        html.push_str("<tr><th scope=\"row\">");
        html.push_str(&row.series_id);
        html.push_str("</th><td>");
        html.push_str(&row.worst_window.to_string());
        html.push_str("</td><td>");
        html.push_str(&row.resource_front.to_string());
        html.push_str("</td><td>");
        html.push_str(&row.suppressed_work.to_string());
        html.push_str("</td><td>");
        html.push_str(&row.uncertainty_label);
        html.push_str("</td></tr>");
    }
    html.push_str("</tbody></table>");
    html
}
