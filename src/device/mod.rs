pub mod device;

use std::str::FromStr;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    LIGHT,
    SWITCH,
    GARAGE,
    SPRINKLER,
}

impl FromStr for DeviceType {
    type Err = ();
    fn from_str(s: &str) -> Result<DeviceType, ()> {
        match s {
            "LIGHT" => Ok(DeviceType::LIGHT),
            "SWITCH" => Ok(DeviceType::SWITCH),
            "GARAGE" => Ok(DeviceType::GARAGE),
            "SPRINKLER" => Ok(DeviceType::SPRINKLER),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareType {
    ARDUINO,
    PI,
    OTHER,
}

impl FromStr for HardwareType {
    type Err = ();
    fn from_str(s: &str) -> Result<HardwareType, ()> {
        match s {
            "ARDUINO" => Ok(HardwareType::ARDUINO),
            "PI" => Ok(HardwareType::PI),
            "OTHER" => Ok(HardwareType::OTHER),
            _ => Err(())
        }
    }
}


pub fn empty_device() -> device::Device {
    device::Device {
        ip: "".to_string(),
        guid: "".to_string(),
        kind: DeviceType::LIGHT,
        hardware: HardwareType::ARDUINO,
        last_state: false,
        last_seen: "".to_string(),
        sw_version: 0,
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GarageAttribute {
    pub attributes: serde_json::Value,
}

impl GarageAttribute {
    fn new() -> GarageAttribute {
        GarageAttribute {
            attributes: serde_json::json!({
              "discreteOnlyOpenClose": true
            })
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnOffAttribute {
    pub attributes: serde_json::Value,
}

impl OnOffAttribute {
    fn new() -> OnOffAttribute {
        OnOffAttribute {
            attributes: serde_json::json!({
                    "commandOnlyOnOff": false,
                    "queryOnlyOnOff": false
            })
        }
    }
}
