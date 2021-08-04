use serde_json::Value;

pub fn garage_attribute() -> Value {
    serde_json::json!({
        "discreteOnlyOpenClose": true
    })
}

pub fn on_off_attribute() -> Value {
    serde_json::json!({
        "commandOnlyOnOff": false,
        "queryOnlyOnOff": false
    })
}

pub fn tv_attribute() -> Value {
    let tv = crate::models::tv::run_command();
    serde_json::json!({
        "commandOnlyOnOff": false,
        "queryOnlyOnOff": false,
        "volumeMaxLevel": tv.volumeMax,
        "volumeCanMuteAndUnmute": true,
        "commandOnlyVolume": false,
        "volumeDefaultPercentage": 10
    })
}