use std::collections::HashMap;

use firebase::Firebase;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warp::{Filter, http, Rejection};

use crate::models;
use crate::models::device;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DeviceState {
    guid: String,
    state: Value,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct QueryAuth {
    key: String,
    email: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DeviceUpdate {
    guid: String,
    ip: String,
    state: Value,
    sw_version: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct FirebaseToken {
    uid: String,
    token: String,
}

#[tokio::main]
pub(crate) async fn run() {
    env_logger::init();
    let cors = warp::cors::cors().allow_any_origin()
        .allow_headers(vec!["x-auth-id", "x-api-key", "User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers", "Content-Type"])
        .allow_methods(vec!["GET", "POST", "PUT"]);

    let route = warp::any().map(warp::reply).with(&cors);
    // This has to be POST for nodejs to work nicely
    let set_sys_status = warp::post()
        .and(warp::path("device"))
        .and(sys_post())
        .and(warp::path::end())
        .and(auth_request())
        .and_then(send_request);
    let list_devices = warp::get()
        .and(warp::path("device"))
        .and(warp::path::end())
        .and(auth_request())
        .and_then(list_devices);

    let list_google_devices = warp::get()
        .and(warp::path("google"))
        .and(auth_request())
        .and_then(list_devices_google);

    let get_device_status = warp::get()
        .and(warp::path("device"))
        .and(warp::path::param())
        .and(auth_request())
        .map(|_guid: String, api_key: String, uid: String| {
            if !check_auth(api_key, uid) {
                "".to_string()
            } else {
                let device = device::get_device_from_guid(&_guid);
                let formatted = format!("{}", device);
                formatted
            }
        });

    let device_update = warp::put()
        .and(warp::path("device"))
        .and(warp::path::end())
        .and(sys_put())
        .and_then(do_device_update);

    let device_update_arduino = warp::put()
        .and(warp::path("update"))
        .and(warp::path::end())
        .and(warp::body::form())
        .map(|_map: HashMap<String, String>| {
            let mut status = "".to_string();
            if _map.contains_key("guid") && _map.contains_key("ip") && _map.contains_key("state") && _map.contains_key("sw_version") {
                let guid = _map.get("guid").unwrap().to_string();
                let ip = _map.get("ip").unwrap().to_string();
                let _state: String = _map.get("state").unwrap().to_string();
                let sw_version: String = _map.get("sw_version").unwrap().to_string();
                let state = Value::from(_state);
                let device_update = DeviceUpdate {
                    guid,
                    ip,
                    state,
                    sw_version,
                };
                status = database_update(device_update);
            }
            status
        });
    let routes = set_sys_status
        .or(list_devices)
        .or(device_update)
        .or(device_update_arduino)
        .or(get_device_status)
        .or(list_google_devices)
        .or(route);
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}

fn auth_request() -> impl Filter<Extract=(String, String, ), Error=Rejection> + Copy {
    warp::header::<String>("x-api-key").and(
        warp::header::<String>("x-auth-id"))
}

/// Used to filter a put request to change the system status
fn sys_post() -> impl Filter<Extract=(DeviceState, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

/// Used to filter a put request to send an update to the database
fn sys_put() -> impl Filter<Extract=(DeviceUpdate, ), Error=warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

/// Sends a change state request to the device.
/// # Params
///     *   `state` A DeviceState representing the device we want to change.
async fn send_request(state: DeviceState, api_token: String, uid: String) -> Result<impl warp::Reply, warp::Rejection> {
    if !check_auth(api_token, uid) {
        Err(warp::reject())
    } else {
        let device = device::get_device_from_guid(&state.guid);

        // Parse the state
        let json = state.state;
        if models::sqlsprinkler::check_if_zone(&state.guid) {
            // Match the device to a sprinkler zone
            let _state: bool = serde_json::from_value(json).unwrap();
            let id = match device.sw_version.parse::<i64>() {
                Ok(r) => r - 1,
                Err(..) => 0
            };
            let status = models::sqlsprinkler::set_zone(device.ip, _state, id);
            let response = match status {
                true => "ok",
                false => "fail",
            };
            Ok(warp::reply::with_status(response.to_string(), http::StatusCode::OK))
        } else {
            if device.kind == models::device_type::Type::SqlSprinklerHost {
                // If the device is a sql sprinkler host, we need to send the request to it...
                let _state: bool = serde_json::from_value(json).unwrap();
                let status = models::sqlsprinkler::set_system(device.ip, _state);
                let response = match status {
                    true => "ok",
                    false => "fail",
                };
                Ok(warp::reply::with_status(response.to_string(), http::StatusCode::OK))
            } else if device.kind == models::device_type::Type::TV {
                // Check if the device is a LG TV.
                if json["volumeLevel"] != serde_json::json!(null) {
                    let vol_state: models::tv::SetVolState = serde_json::from_value(json["volumeLevel"].clone()).unwrap();
                    models::tv::set_volume_state(vol_state);
                } else if json["mute"] != serde_json::json!(null) {
                    let mute_state: models::tv::SetMuteState = serde_json::from_value(json["mute"].clone()).unwrap();
                    models::tv::set_mute_state(mute_state);
                } else {
                    let _state: bool = serde_json::from_value(json).unwrap();
                    models::tv::set_power_state(_state);
                }
                // let volstate: models::tv::SetVolState = serde_json::from_value(json).unwrap();
                // let status = models::tv::set_volume_state(volstate);
                Ok(warp::reply::with_status("set volume state".to_string(), http::StatusCode::OK))
            } else {
                // Everything else is an arduino.
                let _state: bool = serde_json::from_value(json.clone()).unwrap();
                let endpoint = match _state {
                    true => "on",
                    false => "off",
                };
                let url = device.get_api_url_with_param(endpoint.to_string(), device.guid.to_string());
                isahc::get(url).unwrap().status().is_success();
                Ok(warp::reply::with_status("ok".to_string(), http::StatusCode::OK))
            }
        }
    }
}

/// List all devices.
async fn list_devices(api_token: String, uid: String) -> Result<impl warp::Reply, warp::Rejection> {
    let authed = check_auth(api_token, uid);
    if !authed {
        Err(warp::reject())
    } else {
        let devices = serde_json::to_string(&device::get_devices()).unwrap();
        Ok(warp::reply::with_status(devices, http::StatusCode::OK))
    }
}

// fn list_devices_google(token: String) -> String {
async fn list_devices_google(api_token: String, uid: String) -> Result<impl warp::Reply, warp::Rejection> {
    if !check_auth(api_token, uid) {
        Err(warp::reject())
    } else {
        // let devices = device::get_devices_useruuid(token);
        let devices = device::get_devices();
        let mut json_arr = vec![];
        for device in devices.iter() {
            json_arr.push(device.to_google_device());
        }
        println!("Done getting google devices.");
        let json_output = serde_json::json!(json_arr);
        let output = format!("{}", json_output);
        Ok(warp::reply::with_status(output, http::StatusCode::OK))
    }
}

/// Updates the given device in the database.
async fn do_device_update(_device: DeviceUpdate) -> Result<impl warp::Reply, warp::Rejection> {
    let status: String = database_update(_device);
    Ok(warp::reply::with_status(status, http::StatusCode::OK))
}

/// Updates the device in the database.
fn database_update(_device: DeviceUpdate) -> String {
    let device = device::get_device_from_guid(&_device.guid);
    let status = match device.database_update(_device.state, _device.ip, _device.sw_version) {
        true => "updated".to_string(),
        false => "an error occurred.".to_string()
    };
    status
}

/// Gets a firebase instance.
fn get_firebase() -> Firebase {
    Firebase::authed("https://pimation-ee62a-default-rtdb.firebaseio.com", crate::consts::FIREBASE_TOKEN).unwrap()
}

/// Checks whether or not that the user id has the correct api token.
fn check_auth(api_token: String, uid: String) -> bool {
    let query = get_firebase()
        .at(&uid)
        .unwrap()
        .at("api_key")
        .unwrap();
    let token = query.get().unwrap().body;
    let token_str = token.as_str().unwrap();
    let token_equal = token_str == api_token.as_str();
    token_equal
}