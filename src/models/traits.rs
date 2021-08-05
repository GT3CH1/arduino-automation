
/// Gets all the traits that belong to a TV.
pub fn tv_traits() -> Vec<&'static str > {
    vec!["action.devices.traits.OnOff", "action.devices.traits.Volume"]
}

/// Gets all the traits that belong to opening/closing doors
pub fn open_close_traits() -> Vec<&'static str > {
    vec!["action.devices.traits.OpenClose"]
}

/// Gets all traits that belong to turning things on/off
pub fn on_off_traits() -> Vec<&'static str > {
    vec!["action.devices.traits.OnOff"]
}

/// Gets all traits that belong to things that can be rebooted
pub fn reboot_traits() -> Vec<&'static str > {
    vec!["action.devices.traits.Reboot"]
}