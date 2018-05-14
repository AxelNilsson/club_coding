use rocket::Route;
use rand;
use std;

mod series;
mod group;
mod video;
mod users;
mod statistics;

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

pub fn endpoints() -> Vec<Route> {
    routes![
        statistics::index,
        statistics::views,
        series::series,
        series::new_series,
        series::insert_new_series,
        series::edit_series,
        series::update_serie,
        users::users,
        users::edit_users,
        users::update_user,
        group::groups,
        group::new_group,
        group::insert_new_group,
        group::edit_group,
        group::update_group,
        video::videos,
        video::new_video,
        video::insert_new_video,
        video::edit_video,
        video::update_video
    ]
}
