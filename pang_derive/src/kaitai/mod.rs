use std::collections::BTreeMap;

use anyhow::Result;
use serde::Deserialize;

pub mod convert;

// Define an enum to handle the fact that 'type' can be a string or a map.
#[derive(Debug, Deserialize)]
// Serde will try to match one variant after the other.
#[serde(untagged)]
pub enum KsyType {
    Simple(String),
    Switch(SwitchDef),
}

#[derive(Debug, Deserialize)]
pub struct SwitchDef {
    #[serde(rename = "switch-on")]
    pub switch_on: String,
    pub cases: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub id: String,
    #[serde(rename = "endian", default)]
    pub endian: String,
}

#[derive(Debug, Deserialize)]
pub struct SeqItem {
    pub id: String,
    #[serde(rename = "type")]
    pub type_def: Option<KsyType>,
    pub contents: Option<Vec<u8>>,
    #[serde(default)]
    pub repeat: Option<RepeatType>,
    pub size: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepeatType {
    Eos, // End of Stream
}

#[derive(Debug, Deserialize)]
pub struct TypeDef {
    #[serde(default)]
    pub seq: Vec<SeqItem>,
    // Add more fields like `params`, `instances` etc. later
}

#[derive(Debug, Deserialize)]
pub struct KsySpec {
    pub meta: Meta,
    #[serde(default)]
    pub seq: Vec<SeqItem>,
    #[serde(default)]
    pub types: BTreeMap<String, TypeDef>,
}

pub fn parse_ksy(ksy_content: &str) -> Result<KsySpec> {
    let spec: KsySpec = serde_yaml::from_str(ksy_content)?;
    Ok(spec)
}

pub fn convert(ksy_content: &str) -> Result<String> {
    let spec = parse_ksy(ksy_content)?;
    let code = convert::generate_rust_code(&spec)?;
    Ok(code)
}
