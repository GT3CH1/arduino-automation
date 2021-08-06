use std::process::Command;

use serde::{Deserialize, Serialize};
use wake_on_lan;

use crate::models::device::Device;

/// A struct representing the command output for getting the tv volume
#[derive(Serialize, Deserialize, Debug)]
pub struct VolState {
    pub muted: bool,
    pub returnValue: bool,
    pub scenario: String,
    pub volume: u8,
    pub volumeMax: u8,
}

impl ::std::default::Default for VolState {
    fn default() -> Self {
        VolState {
            muted: false,
            returnValue: false,
            scenario: "tv_master_volume".to_string(),
            volume: 0,
            volumeMax: 100,
        }
    }
}


/// Checks to see if the given device is a TV, if so, add the fields required for TV.
/// # Param
/// *   dev : The Device we want to check to see if it is a TV.
/// # Return
/// True if the device is a TV, false otherwise.
pub fn parse_device(mut dev: Device) -> Device {
    if dev.kind == crate::models::device_type::Type::TV {
        dev.last_state = dev.is_online();
        // !!! ONLY QUERY TV WHEN IT IS ON !!!
        if dev.last_state {
            dev.extra_attr = serde_json::json!(get_volume_state());
        } else {
            dev.extra_attr = serde_json::json!(VolState::default())
        }
        return dev;
    }
    dev
}

/// Allows setting TV volume to value
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct SetVolState(u8);

/// Allows toggling mute of TV
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct SetMuteState(bool);

/// Allows turning on/off TV.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct SetPowerState(bool);

/// The output of the requests to the tv.
#[derive(Serialize, Deserialize, Debug)]
struct ReturnVal {
    returnValue: bool,
}

/// Sets the volume state of the TV to the given VolState
/// # Param
/// `state` A SetVolState containing the integer value of the volume we want to set the tv to.
/// # Return
/// The success of the command
pub fn set_volume_state(state: SetVolState) -> bool {
    let mut set_vol_command = Command::new("upstairs-tv");
    let vol_output = set_vol_command.arg("set")
        .arg("vol")
        .arg(state.0.to_string())
        .output().unwrap()
        .stdout;
    let vol_return_str = String::from_utf8(vol_output).unwrap();
    println!("{}", vol_return_str);
    let vol_return: ReturnVal = serde_json::from_str(vol_return_str.as_str()).unwrap();
    vol_return.returnValue
}

/// Sets the power of the TV to the requested value (true/on - false/off)
pub fn set_power_state(state: bool) {
    let mut set_vol_command = Command::new("upstairs-tv");
    set_vol_command.arg("set")
        .arg("power")
        .arg(state.to_string())
        .status()
        .unwrap()
        .success();
}

/// Sets the volume state of the TV to the given VolState
/// # Param
/// `state` A SetMuteState containing the integer value of the volume we want to set the tv to.
/// # Return
/// The success of the command
pub fn set_mute_state(state: SetMuteState) -> bool {
    let mut set_mute_state = Command::new("upstairs-tv");
    let mute_output = set_mute_state.arg("set")
        .arg("mute")
        .arg(state.0.to_string())
        .output().unwrap()
        .stdout;
    let mute_return_str = String::from_utf8(mute_output).unwrap();
    println!("{}", mute_return_str);
    let mute_return: ReturnVal = serde_json::from_str(mute_return_str.as_str()).unwrap();
    mute_return.returnValue
}

/// Gets the volume states from the TV.
/// # Return
/// A VolState struct containing all of the information for the volume of the TV.
pub fn get_volume_state() -> VolState {
    let mut output = Command::new("upstairs-tv");
    output.arg("get")
        .arg("vol");
    let data = String::from_utf8(output.output().unwrap().stdout).unwrap();

    let volstate: VolState = serde_json::from_str(data.as_str()).unwrap();
    println!("{:?}", volstate);
    volstate
}