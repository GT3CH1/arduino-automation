use mysql::Pool;
use std::env;
use crate::device::Device::get_devices;

mod device;
mod daemon;

fn main() {
    println!("Hello, world!");
    let list = get_devices();
    for device in list.iter() {
        println!("{}", device);
    }
    daemon::run();
}

/// Gets a connection to a MySQL database
/// # Return
///     `Pool` A connection to the SQL database.
fn get_pool() -> Pool {
    // Get the SQL database password, parse it.
    let mut user = "".to_string();
    let mut pass = "".to_string();
    let mut host = "".to_string();
    match env::var("SQL_PASS") {
        Ok(val) => pass = val,
        Err(e) => println!("{}", e),
    }
    match env::var("SQL_HOST") {
        Ok(val) => host = val,
        Err(e) => println!("{}", e),
    }
    match env::var("SQL_USER") {
        Ok(val) => user = val,
        Err(e) => println!("{}", e),
    }
    // Build the url for the connection
    let url = format!("mysql://{}:{}@{}:3306/automation", user.as_str(), pass.as_str(), host.as_str());

    let pool = mysql::Pool::new(url).unwrap();
    return pool;
}