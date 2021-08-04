use std::process::Command;

/// {"mute":false,"returnValue":true,"scenario":"mastervolume_tv_speaker","volume":30,"volumeMax":100}
pub struct VolState {
    pub mute: bool,
    pub returnValue: bool,
    pub scenario: String,
    pub volume: u8,
    pub volumeMax: u8,
}


pub fn run_command() -> VolState {
    let output = Command::new("upstairs-tv")
        .arg("get")
        .arg("vol");
    let data = String::from_utf8(output.output().unwrap().stderr).unwrap().as_str();
    let volstate: VolState = serde_json::from_str(data).unwrap();
    volstate
}