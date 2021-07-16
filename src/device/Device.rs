use mysql::Row;
use serde::{Serialize, Deserialize};
use crate::{get_pool, device};
use std::fmt;
use std::str::FromStr;
use crate::device::{HardwareType, DeviceType, GarageAttribute, OnOffAttribute};
use mysql::serde_json::Value;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub ip: String,
    pub guid: String,
    pub kind: DeviceType,
    pub hardware: HardwareType,
    pub last_state: bool,
    pub last_seen: String,
    pub sw_version: i64,
}

impl Device {
    fn get_api_url(&self, endpoint: String) -> String {
        let url = match self.hardware {
            HardwareType::ARDUINO => format!("http://{}/{}", self.ip, endpoint),
            _ => "".to_string(),
        };
        url
    }

    pub fn get_attributes(&self) -> Value {
        let data = match self.kind {
            DeviceType::GARAGE => GarageAttribute::new().attributes,
            DeviceType::LIGHT | DeviceType::SWITCH | DeviceType::SPRINKLER => OnOffAttribute::new().attributes
        };
        data
    }

    pub fn get_api_url_with_param(&self, endpoint: String, param: String) -> String {
        format!("{}?param={}", self.get_api_url(endpoint), param)
    }

    pub fn database_update(&self, state: bool, ip: String, sw_version: i64) -> bool {
        let pool = get_pool();
        let query = format!("UPDATE `devices` SET last_state={}, ip='{}', swVersion='{}', last_seen=CURRENT_TIMESTAMP WHERE guid='{}'",
                            state, ip, sw_version, self.guid);
        println!("{}", query);
        let res = match pool.prep_exec(query, ()) {
            Ok(res) => res.affected_rows() > 0,
            Err(..) => false
        };
        return res;
    }

    pub fn get_google_device_type(&self) -> &str {
        match self.kind {
            DeviceType::LIGHT => "action.devices.types.LIGHT",
            DeviceType::SWITCH => "action.devices.types.SWITCH",
            DeviceType::GARAGE => "action.devices.types.GARAGE",
            DeviceType::SPRINKLER => "action.devices.types.SPRINKLER",
        }
    }

    pub fn get_google_device_traits(&self) -> &str {
        match self.kind {
            DeviceType::LIGHT => "action.devices.traits.OnOff",
            DeviceType::SWITCH => "action.devices.traits.OnOff",
            DeviceType::GARAGE => "action.devices.traits.OpenClose",
            DeviceType::SPRINKLER => "action.devices.traits.OnOff",
        }
    }

    pub fn get_google_device_hardware(&self) -> &str {
        match self.hardware {
            HardwareType::ARDUINO => "Arduino",
            HardwareType::PI => "Raspberry Pi",
            HardwareType::OTHER => "Other"
        }
    }

    /// Converts this device into a json object that google smart home can understand.
    pub fn to_google_device(&self) -> Value {
        let traits = self.get_google_device_traits();
        let device_type = self.get_google_device_type();
        let hardware_model = self.get_google_device_hardware();
        let attributes = self.get_attributes();
        let json = serde_json::json!({

            "id": self.guid,
            "type": device_type,
            "traits": [ traits ],
            "name": {
                "defaultNames": [
                    self.guid
                ],
                "name": [
                    self.guid
                ],
            },
            "deviceInfo": {
                "manufacturer": "GTECH",
                "model": hardware_model,
                "hwVersion": "1.0",
                "swVersion": self.sw_version
            },
            "willReportState": true
        });
        json
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let serialized = serde_json::to_string(&self).unwrap();
        write!(f, "{}", serialized)
    }
}


/// Converts a row from the MySQL database to a Device struct.
impl From<Row> for Device {
    fn from(row: Row) -> Self {
        let ip: String = row.get(0).unwrap();
        let guid: String = row.get(1).unwrap();


        let kind_type: String = row.get(2).unwrap();
        let kind = DeviceType::from_str(kind_type.as_str()).unwrap();

        let hardware_type: String = row.get(3).unwrap();
        let hardware = HardwareType::from_str(hardware_type.as_str()).unwrap();

        let state: String = row.get(4).unwrap();
        let last_state = state == "1";

        let last_seen: String = match row.get(5) {
            Some(res) => res,
            None => "".to_string()
        };

        let _sw_version: String = match row.get(6) {
            Some(res) => res,
            None => "0".to_string()
        };

        let sw_version = match _sw_version.parse::<i64>() {
            Ok(res) => res,
            Err(..) => 0
        };

        let device = Device {
            ip,
            guid,
            kind,
            hardware,
            last_state,
            last_seen,
            sw_version,
        };
        device
    }
}

pub fn get_device_from_guid(guid: String) -> Device {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let query = format!("SELECT * FROM devices WHERE guid = '{}'", guid);
    println!("{}", query);
    let rows = conn.query(query).unwrap();
    let mut _device = device::empty_device();
    for row in rows {
        let _row = row.unwrap();
        return Device::from(_row);
    }
    return _device;
}

pub fn get_devices() -> Vec<Device> {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let mut device_list: Vec<Device> = vec![];
    let rows = conn.query("SELECT * FROM devices").unwrap();
    for row in rows {
        let _row = row.unwrap();
        device_list.push(Device::from(_row));
    }
    device_list
}
