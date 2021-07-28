pub mod device;
mod device_type;
mod hardware_type;
mod attributes;
mod sqlsprinkler;

pub struct SwitchDevice{
    pub ip: String,
    pub guid: String,
    pub kind: device_type::Type,
    pub hardware: hardware_type::Type,
    pub last_state: bool,
    pub last_seen: String,
    pub sw_version: i64,
    pub useruuid: String,
}
