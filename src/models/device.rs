use std::fmt;
use std::process::Command;
use std::str::FromStr;

use mysql::Row;
use mysql::serde_json::Value;
use serde::{Deserialize, Serialize};

use crate::get_pool;
use crate::models::*;
use crate::models::sqlsprinkler::check_if_zone;

/// Data representing a device that can be automated/remotely controlled.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    /// The IP of the device (sometimes used)
    pub ip: String,

    /// The GUID of the device
    pub guid: String,

    /// What kind the device is
    pub kind: device_type::Type,

    /// The hardware used on the device
    pub hardware: hardware_type::Type,

    /// The last state of the device (can be changed)
    pub last_state: bool,

    /// When the device in the database was last updated.
    pub last_seen: String,

    /// The current software version on the device.
    pub sw_version: i64,

    /// The user this device belongs to.
    pub useruuid: String,

    /// The name of the device
    pub name: String,

    /// A list of nicknames for the device
    pub nicknames: Vec<String>,

    /// Any extra attributes in JSON
    pub extra_attr: serde_json::Value,
}

impl Device {
    /// Gets the API Url of the device, with the endpoint.
    /// # Return
    /// A formatted string we can use to send requests to.
    fn get_api_url(&self, endpoint: String) -> String {
        let url = match self.hardware {
            hardware_type::Type::ARDUINO => format!("http://{}/{}", self.ip, endpoint),
            _ => "".to_string(),
        };
        url
    }

    /// Get the attributes of this device.
    /// # Return
    /// The attributes for this device.
    pub fn get_attributes(&self) -> Value {
        let data = match self.kind {
            device_type::Type::GARAGE => attributes::garage_attribute(),
            device_type::Type::LIGHT | device_type::Type::SWITCH | device_type::Type::SPRINKLER | device_type::Type::ROUTER | device_type::Type::SqlSprinklerHost => attributes::on_off_attribute(),
            device_type::Type::TV => attributes::tv_attribute(),
        };
        data
    }

    /// Gets a URL to use for turning on/off relays on arduinos
    /// # Params
    ///     * endpoint : The UUID of the device we want to control.
    ///     * param :   The state we want to set this device to.
    /// # Return
    /// A formatted URL we can send a request to.
    pub fn get_api_url_with_param(&self, endpoint: String, param: String) -> String {
        let url = match self.kind {
            device_type::Type::SqlSprinklerHost => format!("https://api.peasenet.com/sprinkler/systems/{}/state", self.guid),
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

    /// Gets the device type for use in google home
    /// # Return
    /// A str representing the type of device that google home recognizes.
    pub fn get_google_device_type(&self) -> &str {
        match self.kind {
            device_type::Type::LIGHT => "action.devices.types.LIGHT",
            device_type::Type::SWITCH | device_type::Type::SqlSprinklerHost => "action.devices.types.SWITCH",
            device_type::Type::GARAGE => "action.devices.types.GARAGE",
            device_type::Type::SPRINKLER => "action.devices.types.SPRINKLER",
            device_type::Type::ROUTER => "action.devices.types.ROUTER",
            device_type::Type::TV => "action.devices.types.TV"
        }
    }

    /// Gets a list of traits for google home that pertains to this device
    /// # Return
    /// A list (vec) of traits that this device has.
    pub fn get_google_device_traits(&self) -> Vec<&str> {
        match self.kind {
            device_type::Type::GARAGE => traits::open_close_traits(),
            device_type::Type::ROUTER => traits::reboot_traits(),
            device_type::Type::TV => traits::tv_traits(),
            _ => traits::on_off_traits()
        }
    }

    /// Gets the hardware type for google home
    /// # Return
    /// The hardware in a nice string format.
    pub fn get_google_device_hardware(&self) -> &str {
        match self.hardware {
            hardware_type::Type::ARDUINO => "Arduino",
            hardware_type::Type::PI => "Raspberry Pi",
            hardware_type::Type::OTHER => "Other",
            hardware_type::Type::LG => "LG",
        }
    }

    /// Gets the name of this device.
    pub fn get_name(&self) -> &String {
        if &self.name == "" {
            return &self.guid;
        }
        return &self.name;
    }

    /// Converts this device into a json object that google smart home can understand.
    /// # Return
    /// A JSON representation of the device in the format that google home uses.
    pub fn to_google_device(&self) -> Value {
        let traits = self.get_google_device_traits();
        let device_type = self.get_google_device_type();
        let hardware_model = self.get_google_device_hardware();
        let attributes = self.get_attributes();
        let json = serde_json::json!({

            "id": self.guid,
            "type": device_type,
            "traits": traits,
            "name": {
                "defaultNames": [
                    self.get_name()
                ],
                "name": [
                    self.get_name()
                ],
                "nicknames": self.nicknames
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

    /// Checks whether or not this device is online by pinging its IP address.
    /// # Return
    /// True if the ping was successful.
    pub fn is_online(&self) -> bool {
        let mut cmd = Command::new("ping");
        cmd
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .arg(&self.ip)
            .arg("-W")
            .arg("1")
            .arg("-c")
            .arg("1")
            .status()
            .unwrap()
            .success()
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

        let name: String = match row.get(8) {
            Some(res) => res,
            None => "none".into()
        };
        let nicknames = vec![format!("{}", name)];
        let extra_attr: Value = serde_json::json!(null);
        let device = Device {
            ip,
            guid,
            kind,
            hardware,
            last_state,
            last_seen,
            sw_version,
            useruuid,
            name,
            nicknames,
            extra_attr,
        };
        device
    }
}

impl From<sqlsprinkler::Zone> for Device {
    fn from(zone: sqlsprinkler::Zone) -> Device {
        let zone_name = format!("Zone {}", &zone.system_order + 1);
        let pretty_name = format!("{}", &zone.name);
        let nicknames = vec![pretty_name, zone_name];
        let extra_attr: Value = serde_json::json!(null);
        Device {
            ip: "".to_string(),
            guid: zone.id.to_string(),
            kind: device_type::Type::SPRINKLER,
            hardware: hardware_type::Type::PI,
            last_state: zone.state,
            last_seen: "".to_string(),
            sw_version: zone.id as i64,
            useruuid: "".to_string(),
            name: zone.name,
            nicknames,
            extra_attr,
        }
    }
}

impl ::std::default::Default for Device {
    fn default() -> Device {
        let extra_attr: Value = serde_json::json!(null);
        Device {
            ip: "".to_string(),
            guid: "".to_string(),
            kind: device_type::Type::SWITCH,
            hardware: hardware_type::Type::OTHER,
            last_state: false,
            last_seen: "".to_string(),
            sw_version: 0,
            useruuid: "".to_string(),
            name: "".to_string(),
            nicknames: vec!["".to_string()],
            extra_attr,
        }
    }
}

impl Clone for Device {
    fn clone(&self) -> Self {
        Device {
            ip: self.ip.clone(),
            guid: self.guid.clone(),
            kind: self.kind,
            hardware: self.hardware,
            last_state: self.last_state,
            last_seen: self.last_seen.clone(),
            sw_version: self.sw_version,
            useruuid: self.useruuid.clone(),
            name: self.name.clone(),
            nicknames: self.nicknames.clone(),
            extra_attr: self.extra_attr.clone(),
        }
    }
}

/// Gets the device from the database that corresponds to the given UUID.  If the device has the following pattern:
/// xxxxxxxx-yyy-zzzzzzzzzzzz-n then we will get the device status from the SQLSprinkler host.
/// # Params
///     *   `guid`  The GUID of the device we want to get.
/// # Return
///     *   A device that corresponds to the given uuid, if there is no match, return a default device.
pub fn get_device_from_guid(guid: &String) -> Device {
    // Match the device to a sprinkler zone
    if check_if_zone(guid) {
        return sqlsprinkler::get_device_from_sqlsprinkler(guid.clone());
    }

    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let query = format!("SELECT * FROM devices WHERE guid = '{}'", guid);
    println!("{}", query);
    let rows = conn.query(query).unwrap();
    let mut _device = Device::default();
    for row in rows {
        let _row = row.unwrap();
        let mut dev: Device = Device::from(_row);
        if dev.kind == device_type::Type::SqlSprinklerHost {
            let ip = &dev.ip;
            dev.last_state = sqlsprinkler::get_status_from_sqlsprinkler(ip).unwrap();
        }
        dev = tv::parse_device(dev.clone());
        return dev;
    }
    return _device;
}


/// Gets all of the devices that are connected to this user in the database.
/// # Return
///     * A `Vec<Device>` containing all of the device information.
pub fn get_devices() -> Vec<Device> {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let mut device_list: Vec<Device> = vec![];
    let rows = conn.query("SELECT * FROM devices").unwrap();
    for row in rows {
        let _row = row.unwrap();
        println!("Got a device.");
        let mut dev = Device::from(_row);
        sqlsprinkler::check_if_device_is_sqlsprinkler_host(&mut dev, &mut device_list);
        let dev = tv::parse_device(dev.clone());
        device_list.push(dev);
    }
    device_list
}

/// Gets all of the devices that are connected to this user in the database.
/// # Return
///     * A `Vec<Device>` containing all of the device information.
pub fn get_devices_uuid(user_uuid: String) -> Vec<Device> {
    let pool = get_pool();
    let mut conn = pool.get_conn().unwrap();
    let mut device_list: Vec<Device> = vec![];
    let query = format!("SELECT * FROM devices WHERE useruuid='{}'", user_uuid);
    let rows = conn.query(query).unwrap();
    for row in rows {
        let _row = row.unwrap();
        println!("Got a device.");
        let mut dev = Device::from(_row);
        sqlsprinkler::check_if_device_is_sqlsprinkler_host(&mut dev, &mut device_list);
        let dev = tv::parse_device(dev.clone());
        device_list.push(dev);
    }
    device_list
}