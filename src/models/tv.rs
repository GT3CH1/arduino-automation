use std::process::Command;
use serde::{Serialize, Deserialize};

/// {"mute":false,"returnValue":true,"scenario":"mastervolume_tv_speaker","volume":30,"volumeMax":100}
#[derive(Serialize, Deserialize, Debug)]
pub struct VolState {
    pub muted: bool,
    pub returnValue: bool,
    pub scenario: String,
    pub volume: u8,
    pub volumeMax: u8,
}


pub fn run_command() -> VolState {
    let mut output = Command::new("upstairs-tv");
    output.arg("get")
        .arg("vol");
    let data = String::from_utf8(output.output().unwrap().stdout).unwrap();
    println!("{}",data);
    let volstate: VolState = serde_json::from_str(data.as_str()).unwrap();
    println!("{:?}", volstate);
    volstate
}