#[cfg(test)]
mod tests {
    use crate::models::*;

    const UUID: &str = "rtr1";

    fn get_device_from_test_uuid() -> device::Device {
        let device = device::get_device_from_guid(&String::from(UUID));
        device
    }

    fn get_device_from_bad_uuid() -> device::Device {
        let device = device::get_device_from_guid(&"bad".to_string());
        device
    }

    #[test]
    fn test_get_device_from_bad_uuid() {
        let device = get_device_from_bad_uuid();
        assert_eq!(device::Device::default(), device);
    }

    #[test]
    fn test_get_device_from_good_uuid() {
        let device = get_device_from_test_uuid();
        assert_ne!(device::Device::default(), device);
    }

    #[test]
    fn test_device_has_correct_kind() {
        let device = get_device_from_test_uuid();
        assert_eq!(device_type::Type::ROUTER, device.kind);
    }

    #[test]
    fn test_device_has_correct_hardware_type() {
        let device = get_device_from_test_uuid();
        assert_eq!(hardware_type::Type::OTHER, device.hardware);
    }

    #[test]
    fn test_device_has_timestamp() {
        let device = get_device_from_test_uuid();
        assert_ne!("".to_string(), device.last_seen);
    }

    #[test]
    fn test_device_has_ip() {
        let device = get_device_from_test_uuid();
        assert_ne!("".to_string(), device.ip);
    }

    #[test]
    fn test_device_has_sw_version() {
        let device = get_device_from_test_uuid();
        assert_ne!("", device.sw_version);
    }

    #[test]
    fn test_device_has_name() {
        let device = get_device_from_test_uuid();
        assert_ne!("".to_string(), device.name);
    }

    #[test]
    fn test_device_has_nicknames() {
        let device = get_device_from_test_uuid();
        let empty_nicknames = vec!["".to_string()];
        assert_ne!(device.nicknames, empty_nicknames);
    }

    #[test]
    fn test_device_to_google_device() {
        let device = get_device_from_test_uuid();
        let device_json =
            serde_json::json!({
            "attributes": {
                "commandOnlyOnOff": false,
                "queryOnlyOnOff": false
            },
            "deviceInfo": {
                "hwVersion": "1.0",
                "manufacturer": "GTECH",
                "model": "Other",
                "swVersion": "64"
            },
            "id": "rtr1",
            "name": {
                "defaultNames": ["Basement Router"],
                "name": "Basement Router",
                "nicknames": ["Basement Router"]
            },
            "traits": ["action.devices.traits.Reboot"],
            "type": "action.devices.types.ROUTER",
            "willReportState": true
        });
        assert_eq!(device.to_google_device().to_string(), device_json.to_string());
    }
}