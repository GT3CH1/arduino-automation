use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use warp::{Filter, http};

use crate::models::device;
use crate::models;
use serde_json::Value;


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DeviceState {
    guid: String,
    state: Value,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DeviceUpdate {
    guid: String,
    ip: String,
    state: bool,
    sw_version: i64,
}

#[tokio::main]
pub(crate) async fn run() {
    // This has to be POST for nodejs to work nicely
    let set_sys_status = warp::post()
        .and(warp::path("device"))
        .and(sys_post())
        .and(warp::path::end())
        .and_then(send_request);
    let list_devices = warp::get()
        .and(warp::path("device"))
        .and(warp::path::end())
        .and_then(list_devices);
    let list_google_devices = warp::get()
        .and(warp::path("google"))
        .and_then(list_devices_google);

    let get_device_status = warp::get()
        .and(warp::path("device"))
        .and(warp::path::param())
        .map(|_guid: String| {
            let device = device::get_device_from_guid(&_guid);
            format!("{}", device)
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
            println!("{:?}", _map);
            if _map.contains_key("guid") && _map.contains_key("ip") && _map.contains_key("state") && _map.contains_key("sw_version") {
                let guid = _map.get("guid").unwrap().to_string();
                let ip = _map.get("ip").unwrap().to_string();
                let _state: String = _map.get("state").unwrap().to_string();
                let _sw_version: String = _map.get("sw_version").unwrap().to_string();
                let state = match _state.parse::<bool>() {
                    Ok(val) => val,
                    Err(_val) => _state == "1"
                };
                let sw_version = _sw_version.parse::<i64>().unwrap();
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
        .or(list_google_devices);
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
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
async fn send_request(state: DeviceState) -> Result<impl warp::Reply, warp::Rejection> {
    let device = device::get_device_from_guid(&state.guid);

    // Parse the state
    let json: serde_json::Value = serde_json::from_value(state.state).unwrap();

    if models::sqlsprinkler::check_if_zone(&state.guid) {
        // Match the device to a sprinkler zone
        let _state = json["state"] == "true";
        let status = models::sqlsprinkler::set_zone(device.ip, _state, device.sw_version - 1);
        let response = match status {
            true => "ok",
            false => "fail",
        };
        Ok(warp::reply::with_status(response.to_string(), http::StatusCode::OK))
    } else {
        if device.kind == models::device_type::Type::SqlSprinklerHost {
            // If the device is a sql sprinkler host, we need to send the request to it...
            let _state = json["state"] == "true";
            let status = models::sqlsprinkler::set_system(device.ip, _state);
            let response = match status {
                true => "ok",
                false => "fail",
            };
            Ok(warp::reply::with_status(response.to_string(), http::StatusCode::OK))
        } else if device.kind == models::device_type::Type::TV {
            // Check if the device is a LG TV.
            let rep = serde_json::to_value(models::tv::run_command()).unwrap().to_string();
            Ok(warp::reply::with_status(rep, http::StatusCode::OK))
        } else {
            // Everything else is an arduino.
            let _state = json["state"] == "true";
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

/// List all devices.
async fn list_devices() -> Result<impl warp::Reply, warp::Rejection> {
    let devices = serde_json::to_string(&device::get_devices()).unwrap();
    Ok(warp::reply::with_status(devices, http::StatusCode::OK))
}

// fn list_devices_google(token: String) -> String {
async fn list_devices_google() -> Result<impl warp::Reply, warp::Rejection> {
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

/* TESTING BELOW */
#[test]
fn test_device_filter_allowed() {
    let res =
        block_on(warp::test::request()
            .path("device")
            .matches(&warp::get()));
    assert!(res);
}