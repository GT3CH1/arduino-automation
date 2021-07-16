use warp::{Filter, http};
use serde::{Serialize, Deserialize};
use crate::device::{device};
use std::collections::HashMap;
use crate::device::device::get_device_from_guid;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DeviceState {
    guid: String,
    state: bool,
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
        .and(warp::path::end())
        .and_then(list_devices_google);
    let get_device_status = warp::get()
        .and(warp::path("device"))
        .and(warp::path::param())
        .map(|_guid: String| {
           let device = get_device_from_guid(_guid);
           format!("{}",device.last_state)
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
                let state = match _state.parse::<bool>(){
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


async fn send_request(state: DeviceState) -> Result<impl warp::Reply, warp::Rejection> {
    let device = device::get_device_from_guid(state.guid);
    let guid = &device.guid;
    let mut endpoint = "";
    if state.state {
        endpoint = "on";
    } else {
        endpoint = "off";
    }


    let url = device.get_api_url_with_param(endpoint.to_string(), guid.to_string());
    let success = reqwest::get(url).await;
    let status = match success {
        Ok(_) => true,
        Err(_e) => false
    };
    //TODO: Update "last_state" in database.
    let json = serde_json::json!({
        "success": status
    });
    Ok(warp::reply::with_status(json.to_string(), http::StatusCode::OK))
}

async fn list_devices() -> Result<impl warp::Reply, warp::Rejection> {
    let devices = serde_json::to_string(&device::get_devices()).unwrap();
    Ok(warp::reply::with_status(devices, http::StatusCode::OK))
}

async fn list_devices_google() -> Result<impl warp::Reply, warp::Rejection> {
    let devices = &device::get_devices();
    let mut json_arr = vec![];
    for device in devices.iter() {
        json_arr.push(device.to_google_device());
    }
    let json_output = serde_json::json!(json_arr);
    let output = format!("{}", json_output);
    Ok(warp::reply::with_status(output, http::StatusCode::OK))
}

async fn do_device_update(_device: DeviceUpdate) -> Result<impl warp::Reply, warp::Rejection> {
    let status = database_update(_device);
    Ok(warp::reply::with_status(status, http::StatusCode::ACCEPTED))
}

fn database_update(_device: DeviceUpdate) -> String {
    let device = device::get_device_from_guid(_device.guid);
    let status = match device.database_update(_device.state, _device.ip, _device.sw_version) {
        true => "updated".to_string(),
        false => "an error occurred.".to_string()
    };
    return status;
}

async fn do_device_update_hashmap(_map: HashMap<String, String>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut status = "".to_string();

    if status != "" {
        Ok(warp::reply::with_status(status, http::StatusCode::OK))
    } else {
        Err(warp::reject())
    }
}