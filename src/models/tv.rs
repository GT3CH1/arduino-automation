use std::process::Command;
use serde::{Serialize, Deserialize};

/// A struct representing the command output for getting the tv volume
#[derive(Serialize, Deserialize, Debug)]
pub struct VolState {
    pub muted: bool,
    pub returnValue: bool,
    pub scenario: String,
    pub volume: u8,
    pub volumeMax: u8,
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
    println!("{}",vol_return_str);
    let vol_return: ReturnVal = serde_json::from_str(vol_return_str.as_str()).unwrap();
    vol_return.returnValue
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
    println!("{}",mute_return_str);
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