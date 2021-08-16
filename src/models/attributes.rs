use serde_json::Value;

/// Gets attributes for garage doors
/// # Return
/// The attributes needed for garage doors.
pub fn garage_attribute() -> Value {
    serde_json::json!({
        "discreteOnlyOpenClose": true
    })
}

/// Gets the attributes for on/off devices (switches, outlets, some lights)
/// # Return
/// The attributes needed for on/off devices
pub fn on_off_attribute() -> Value {
    serde_json::json!({
        "commandOnlyOnOff": false,
        "queryOnlyOnOff": false
    })
}

/// Gets all the attributes needed for TV's
/// # Return
/// The attributes needed for TV's
pub fn tv_attribute() -> Value {
    let _tv = crate::models::tv::get_tv_state();
    serde_json::json!({
        "commandOnlyOnOff": false,
        "queryOnlyOnOff": false,
        "volumeMaxLevel": _tv.volumeMax,
        "volumeCanMuteAndUnmute": true,
        "commandOnlyVolume": false,
        "volumeDefaultPercentage": 10
    })
}