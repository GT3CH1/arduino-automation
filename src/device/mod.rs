pub mod googledevice;
pub mod Device;

use std::str::FromStr;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    LIGHT,
    SWITCH,
    GARAGE,
    SPRINKLER,
    ROUTER,
}

impl FromStr for DeviceType {
    type Err = ();
    fn from_str(s: &str) -> Result<DeviceType, ()> {
        match s {
            "LIGHT" => Ok(DeviceType::LIGHT),
            "SWITCH" => Ok(DeviceType::SWITCH),
            "GARAGE" => Ok(DeviceType::GARAGE),
            "SPRINKLER" => Ok(DeviceType::SPRINKLER),
            "ROUTER" => Ok(DeviceType::ROUTER),
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


pub fn empty_device() -> Device::Device {
    Device::Device {
        ip: "".to_string(),
        guid: "".to_string(),
        kind: DeviceType::LIGHT,
        hardware: HardwareType::ARDUINO,
        last_state: false,
    }
}

