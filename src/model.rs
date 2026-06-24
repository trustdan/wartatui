//! Data model — mirrors `dod-org-data-research-ingest.json`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

/// A single organizational unit.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OrgNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub echelon: u8,
    #[serde(rename = "type")]
    pub org_type: String,
    pub parent: Option<String>,
    /// Statutory authority (e.g. "10 U.S.C. § 161").
    pub source: Option<String>,
    /// Free-form metadata: sourceUrl, confidence, notes, displayAlias, ...
    #[serde(default)]
    pub meta: BTreeMap<String, serde_json::Value>,
}

impl OrgNode {
    /// Convenience accessor for a string-valued meta field.
    pub fn meta_str(&self, key: &str) -> Option<&str> {
        self.meta.get(key).and_then(|v| v.as_str())
    }
}

/// An operational (non-parent) relationship — the second chain of command.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OrgEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OrgMeta {
    pub title: String,
    #[serde(rename = "secondaryTitle", default)]
    pub secondary_title: Option<String>,
    #[serde(rename = "asOf", default)]
    pub as_of: String,
    #[serde(default)]
    pub classification: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OrgData {
    pub nodes: Vec<OrgNode>,
    #[serde(default)]
    pub edges: Vec<OrgEdge>,
    pub meta: OrgMeta,
}

/// Load and parse the org dataset from disk.
pub fn load(path: &str) -> Result<OrgData, Box<dyn Error>> {
    let json = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let data: OrgData = serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {}", e))?;
    Ok(data)
}
