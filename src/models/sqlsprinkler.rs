use serde::{Deserialize, Serialize};
use isahc::prelude::*;
use isahc::Request;
use crate::models::device::{Device, get_devices};
use crate::models::device_type;
use std::error::Error;
use regex::Regex;

/// Represents the state of an SQLSprinkler host.
#[derive(Deserialize)]
pub struct SystemState {
    pub system_enabled: bool,
}

/// A struct representing the data from SQLSprinkler zones.
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

/// Gets the status from the SQLSprinkler host
/// # Params
///     * `ip` The IP Address of the SQLSprinkler host.
/// # Return
///     A boolean representing the state of the SQLSprinkler host, or an error if something happened.
pub(crate) fn get_status_from_sqlsprinkler(ip: &String) -> Result<bool, Box<dyn Error>> {
    let url = format!("http://{}:3030/system/state", ip);
    println!("Getting state for {} ({})", ip, url);
    let response = isahc::get(url).unwrap().text().unwrap();
    println!("{}:?", response);
    println!("done");
    let system_status: SystemState = serde_json::from_str(&response).unwrap();
    Ok(system_status.system_enabled)
}

/// Gets all the zones from the SQLSprinkler host.
/// # Params
///     *   `ip` A string representing the IP address of the SQLSprinkler host.
/// # Returns
///     * A `Vec<Zone>` Representing all of the SQLSprinkler zones on the given host.  Or, if an
/// error occurs, we will get that error.
fn get_zones_from_sqlsprinkler(ip: &String) -> Result<Vec<Zone>, Box<dyn Error>> {
    let url = format!("http://{}:3030/zone/info", ip);
    println!("Getting state for {} ({})", ip, url);
    let response = isahc::get(url).unwrap().text().unwrap();
    println!("{}:?", response);
    let zone_list: Vec<Zone> = serde_json::from_str(&response).unwrap();
    Ok(zone_list)
}


/// Checks to see if the given device is an SQLSprinkler Host.  If it is, push the zones that are
/// connected to that SQLSprinkler host.
/// # Params
///     * `dev` -> A mutable device representing the SQLSprinkler host
///     * `device_list` -> A Vec containing the list of devices we want to add all of the zones to.
/// # Return
///     * True if the device is a sqlsprinkler host.
pub fn check_if_device_is_sqlsprinkler_host(dev: &mut Device, device_list: &mut Vec<Device>) -> bool {
    if dev.kind == device_type::Type::SqlSprinklerHost {
        let ip = &dev.ip;
        dev.last_state = get_status_from_sqlsprinkler(ip).unwrap();
        let sprinkler_list = get_zones_from_sqlsprinkler(ip).unwrap();
        for zone in sprinkler_list {
            // Create a device from a sprinkler zone
            let mut sprinkler_device = Device::from(zone);
            // Make a new guid in the form of deviceguid-zoneid
            let new_guid = format!("{}-{}", dev.guid, sprinkler_device.guid);
            sprinkler_device.guid = new_guid;
            sprinkler_device.ip = dev.ip.to_string();
            device_list.push(sprinkler_device);
        }
        return true;
    }
    return false;
}

/// Checks to see if the given guid is a SQLSprinkler zone.
/// # Param
///     *   `guid`  The GUID of the device we are checking.
/// # Return
///     True if there is a match to the pattern of a SQLSprinkler zone.
pub fn check_if_zone(guid: &String) -> bool {
    let re = Regex::new(r"(?im)^[0-9A-Fa-f]{8}[-]?(?:[0-9A-Fa-f]{4}[-]?){3}[0-9A-Fa-f]{12}[-][0-9].?$").unwrap();
    re.is_match(guid.as_str())
}

/// Checks to see if the given device is an SQLSprinkler zone. If it is, return a device of that zone.
/// If the guid is not a SQLSprinkler zone, nothing will happen.
/// # Param
///     *   `dev` The device we want to check to see if it is a sqlsprinkler zone
/// # Return
///     * A device representing the `Zone` if it exists, an empty device if there was no match.
pub fn get_device_from_sqlsprinkler(guid: String) -> Device {
    let device_list = get_devices();
    for dev in device_list {
        if dev.guid == guid {
            return dev;
        }
    }
    Device::default()
}