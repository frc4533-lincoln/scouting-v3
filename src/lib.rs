#![no_std]

extern crate alloc;

#[macro_use]
extern crate serde;
extern crate serde_json;

use alloc::{collections::BTreeMap, string::String, vec::Vec};

#[derive(Deserialize, Serialize, Clone)]
pub struct EventCfg {
    /// Event name
    ///
    /// e.g. `SCRIW 2023`
    pub name: String,
    /// Year (used to find game details)s
    pub year: u16,

    /// Qualification
    #[serde(rename = "qualification")]
    pub qual_matches: BTreeMap<u8, Vec<u32>>,
    /// Semi-finals
    #[serde(rename = "semifinals")]
    pub semi_matches: Option<BTreeMap<u8, Vec<u32>>>,
    /// Finals
    #[serde(rename = "finals")]
    pub final_matches: BTreeMap<u8, Vec<u32>>,
}

#[derive(Deserialize, Serialize)]
pub struct Status {
    pub version: String,
    pub enabled: bool,
    pub managed: bool,
    pub event: String,
}

#[derive(Deserialize, Serialize)]
pub struct ListEvents {
    pub current: String,
    pub events: Vec<Event>,
}

#[derive(Deserialize, Serialize)]
pub struct Event {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct ListScouters {
    pub scouters: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterScouterRes {
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct ManagerSetReq {
    pub enabled: Option<bool>,
    pub managed: Option<bool>,
    pub event: Option<String>,
}
