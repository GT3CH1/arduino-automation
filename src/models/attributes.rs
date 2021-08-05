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
    let _tv = crate::models::tv::get_volume_state();
    serde_json::json!({
        "commandOnlyOnOff": false,
        "queryOnlyOnOff": false,
        "volumeMaxLevel": _tv.volumeMax,
        "volumeCanMuteAndUnmute": true,
        "commandOnlyVolume": false,
        "volumeDefaultPercentage": 10
    })
}