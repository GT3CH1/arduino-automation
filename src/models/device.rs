use mysql::Row;
use serde::{Serialize, Deserialize};
use crate::{get_pool};

use std::fmt;
use std::str::FromStr;
use mysql::serde_json::Value;
use crate::models::{device_type, hardware_type, attributes, sqlsprinkler};
use std::error::Error;
use futures::executor::block_on;
use reqwest::Client;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub ip: String,
    pub guid: String,
    pub kind: device_type::Type,
    pub hardware: hardware_type::Type,
    pub last_state: bool,
    pub last_seen: String,
    pub sw_version: i64,
    pub useruuid: String,
}

impl Device {
    fn get_api_url(&self, endpoint: String) -> String {
        let url = match self.hardware {
            hardware_type::Type::ARDUINO => format!("http://{}/{}", self.ip, endpoint),
            _ => "".to_string(),
        };
        url
    }

    pub fn get_attributes(&self) -> Value {
        let data = match self.kind {
            device_type::Type::GARAGE => attributes::garage_attribute(),
            device_type::Type::LIGHT | device_type::Type::SWITCH | device_type::Type::SPRINKLER | device_type::Type::ROUTER | device_type::Type::SQLSPRINLER_HOST => attributes::on_off_attribute()
        };
        data
    }

    pub fn get_api_url_with_param(&self, endpoint: String, param: String) -> String {
        let url = match self.kind {
            device_type::Type::SQLSPRINLER_HOST => format!("https://api.peasenet.com/sprinkler/systems/{}/state", self.guid),
            _ => format!("{}?param={}", self.get_api_url(endpoint), param)
        };
        url
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
            device_type::Type::LIGHT => "action.devices.types.LIGHT",
            device_type::Type::SWITCH | device_type::Type::SQLSPRINLER_HOST => "action.devices.types.SWITCH",
            device_type::Type::GARAGE => "action.devices.types.GARAGE",
            device_type::Type::SPRINKLER => "action.devices.types.SPRINKLER",
            device_type::Type::ROUTER => "action.devices.types.ROUTER"
        }
    }

    pub fn get_google_device_traits(&self) -> &str {
        match self.kind {
            device_type::Type::LIGHT | device_type::Type::SWITCH | device_type::Type::SPRINKLER | device_type::Type::SQLSPRINLER_HOST => "action.devices.traits.OnOff",
            device_type::Type::GARAGE => "action.devices.traits.OpenClose",
            device_type::Type::ROUTER => "action.devices.traits.Reboot"
        }
    }

    pub fn get_google_device_hardware(&self) -> &str {
        match self.hardware {
            hardware_type::Type::ARDUINO => "Arduino",
            hardware_type::Type::PI => "Raspberry Pi",
            hardware_type::Type::OTHER => "Other"
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
            "attributes": attributes,
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

async fn get_status_from_sqlsprinkler(ip: &String) -> Result<bool, Box<dyn Error>> {
    let url = format!("http://{}:3030/system/state", ip);
    println!("Getting state for {} ({})", ip, url);
    let client = Client::new();
    let response = block_on(client.get(url).send().await?.text())?;
    println!("done");
    let system_status: sqlsprinkler::SystemState = serde_json::from_str(&response).unwrap();
    Ok(system_status.system_enabled)
}

/// Converts a row from the MySQL database to a Device struct.

impl From<Row> for Device {
    fn from(row: Row) -> Device {
        let ip: String = row.get(0).unwrap();
        let guid: String = row.get(1).unwrap();


        let kind_type: String = row.get(2).unwrap();
        let kind = device_type::Type::from_str(kind_type.as_str()).unwrap();

        let hardware_type: String = row.get(3).unwrap();
        let hardware = hardware_type::Type::from_str(hardware_type.as_str()).unwrap();

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

        let useruuid: String = match row.get(7) {
            Some(res) => res,
            None => "000-000-000-000".into(),
        };

        let device = Device {
            ip,
            guid,
            kind,
            hardware,
            last_state,
            last_seen,
            sw_version,
            useruuid,
        };
        device
    }
}

impl ::std::default::Default for Device {
    fn default() -> Device {
        Device {
            ip: "".to_string(),
            guid: "".to_string(),
            kind: device_type::Type::SWITCH,
            hardware: hardware_type::Type::OTHER,
            last_state: false,
            last_seen: "".to_string(),
            sw_version: 0,
            useruuid: "".to_string(),
        }
    }
}

pub async fn get_device_from_guid(guid: String) -> Device {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let query = format!("SELECT * FROM devices WHERE guid = '{}'", guid);
    println!("{}", query);
    let rows = conn.query(query).unwrap();
    let mut _device = Device::default();
    for row in rows {
        let _row = row.unwrap();
        let dev: Device = Device::from(_row);
        return dev;
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
        println!("Got a device.");
        let mut dev = Device::from(_row);
        if dev.kind == device_type::Type::SQLSPRINLER_HOST {
            let ip = &dev.ip;
            dev.last_state = block_on(get_status_from_sqlsprinkler(ip)).unwrap();
        }
        device_list.push(dev);
    }
    device_list
}

pub fn get_devices_useruuid(useruuid: String) -> Vec<Device> {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let mut device_list: Vec<Device> = vec![];
    let query = format!("SELECT * FROM devices WHERE useruuid='{}'", useruuid);
    println!("{}", useruuid);
    let rows = conn.query(query).unwrap();
    for row in rows {
        let _row = row.unwrap();
        let dev = Device::from(_row);
        device_list.push(dev);
    }
    device_list
}