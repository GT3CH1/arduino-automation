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