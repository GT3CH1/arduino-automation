use serde::{Deserialize, Serialize};
use isahc::prelude::*;
use isahc::Request;

#[derive(Deserialize)]
pub struct SystemState {
    pub system_enabled: bool,
}

#[derive(Deserialize)]
pub struct Zone {
    pub name: String,
    pub gpio: u8,
    pub time: u64,
    pub enabled: bool,
    pub auto_off: bool,
    pub system_order: i8,
    pub state: bool,
    pub id: i8,
}

#[derive(Serialize, Deserialize, Debug)]
struct ZoneToggle {
    id: i64,
    state: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct SystemToggle {
    system_enabled: bool,
}

/// Sets the zone status to the given state
pub fn set_zone(ip: String, state: bool, id: i64) -> bool {
    let url = format!("http://{}:3030/zone", ip);
    let zone_toggle = ZoneToggle {
        id,
        state,
    };
    Request::put(url)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&zone_toggle).unwrap()).unwrap()
        .send().unwrap().status().is_success()
}

/// Sets the sprinkler system on/off
pub fn set_system(ip: String, state: bool) -> bool {
    let url = format!("http://{}:3030/system/state", ip);
    let system_state = SystemToggle {
        system_enabled: state
    };
    Request::put(url)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&system_state).unwrap()).unwrap()
        .send().unwrap().status().is_success()
}