use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Pil {
    #[serde(rename = "nCommitments")]
    pub n_commitments: u64,
    #[serde(rename = "nQ")]
    n_q: u64,
    #[serde(rename = "nIm")]
    n_im: u64,
    #[serde(rename = "nConstants")]
    pub n_constants: u64,
    pub references: HashMap<String, PilReference>,
    #[serde(rename = "publics")]
    public: Vec<PilPublic>,
    #[serde(rename = "expressions")]
    expressions: Vec<PilExpression>,
    #[serde(rename = "polIdentities")]
    pol_identities: Vec<PilPolIdentity>,
    #[serde(rename = "plookupIdentities")]
    plookup_identities: Vec<PilPlookupIdentity>,
    #[serde(rename = "permutationIdentities")]
    permutation_identities: Vec<PilPermutationIdentity>,
    #[serde(rename = "connectionIdentities")]
    connection_identities: Vec<PilConnectionIdentity>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PilConnectionIdentity {
    pols: Vec<u64>,
    connections: Vec<u64>,
    #[serde(rename = "fileName")]
    file_name: String,
    line: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct PilPermutationIdentity {
    f: Vec<u64>,
    t: Vec<u64>,
    #[serde(rename = "selF")]
    sel_f: u64,
    #[serde(rename = "selT")]
    sel_t: u64,
    #[serde(rename = "fileName")]
    file_name: String,
    line: u64,
}

#[derive(Deserialize, Serialize, Debug)]
struct PilPlookupIdentity {
    f: Vec<u64>,
    t: Vec<u64>,
    #[serde(rename = "selF")]
    sel_f: Option<serde_json::Value>,
    #[serde(rename = "selT")]
    sel_t: Option<serde_json::Value>,
    #[serde(rename = "fileName")]
    file_name: String,
    line: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct PilPolIdentity {
    e: u64,
    #[serde(rename = "fileName")]
    file_name: String,
    line: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct PilPublic {
    #[serde(rename = "polType")]
    pol_type: String,
    #[serde(rename = "polId")]
    pol_id: u64,
    idx: u64,
    id: u64,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PilReference {
    #[serde(rename = "type")]
    pub reference_type: String,
    pub id: u64,
    #[serde(rename = "polDeg")]
    pub pol_deg: u64,
    #[serde(rename = "isArray")]
    pub is_array: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub len: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PilExpression {
    op: String,
    deg: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<PilExpressionValues>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deps: Option<Vec<u64>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PilExpressionValues {
    op: String,
    deg: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<Box<PilExpressionValues>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<bool>,
}

impl Pil {
    pub fn from_json_str(json: &str) -> Result<Pil, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn from_json_file(file: &str) -> Result<Pil, serde_json::Error> {
        let json = std::fs::read_to_string(file).unwrap();
        Pil::from_json_str(&json)
    }
}

#[cfg(test)]
mod test {
    use super::Pil;
    #[test]
    fn test_load_from_json_file() {
        println!("Start Parsing Pil");
        let _ = Pil::from_json_file("test_data/pil/main.pil.json").unwrap();
        let _ = Pil::from_json_file("test_data/pil/main_2.pil.json").unwrap();
        println!("End Parsing Pil");
    }

    #[test]
    fn test_pol_array() {
        let pil = Pil::from_json_file("test_data/pil/main.pil.json").unwrap();

    }
}
