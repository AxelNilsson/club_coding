use rocket::Route;
use rand;
use std;

mod series;
mod group;
mod video;
mod users;
mod statistics;
mod structs;

pub fn generate_token(length: u8) -> Result<String, std::io::Error> {
    let bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
    let strings: Vec<String> = bytes.iter().map(|byte| format!("{:02X}", byte)).collect();
    return Ok(strings.join(""));
}

pub fn create_slug(title: &String) -> String {
    title
        .chars()
        .map(|character| match character {
            'A'...'Z' => ((character as u8) - b'A' + b'a') as char,
            'a'...'z' | '0'...'9' => character,
            _ => '-',
        })
        .collect()
}

/// Assembles all of the endpoints of the admin endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    let mut total = vec![];

    let mut stats = statistics::endpoints();
    total.append(&mut stats);

    let mut series = series::endpoints();
    total.append(&mut series);

    let mut users = users::endpoints();
    total.append(&mut users);

    let mut group = group::endpoints();
    total.append(&mut group);

    let mut video = video::endpoints();
    total.append(&mut video);

    total
}
