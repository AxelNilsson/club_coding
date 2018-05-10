use rocket::Route;
use rocket_contrib::Template;
use rocket::response::{NamedFile, Redirect};
use rocket::http::ContentType;
use club_coding::establish_connection;
use club_coding::models::{Series, Videos};
use std::fs::File;
use rocket::response::content::Content;
use diesel::prelude::*;
use member::Member;
use users::User;
use std;

pub fn get_videos() -> Vec<Videos> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();
    videos
        .order(created.asc())
        .load::<Videos>(&connection)
        .expect("Error loading videos")
}

fn get_video_data_from_uuid(uid: String) -> Result<Videos, std::io::Error> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();

    let results = videos
        .filter(uuid.eq(uid))
        .limit(1)
        .load::<Videos>(&connection)
        .expect("Error loading videos");

    if results.len() == 1 {
        return Ok(results[0].clone());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "no video found",
        ));
    }
}

fn get_series_title(uid: Option<i64>) -> String {
    match uid {
        Some(uid) => {
            use club_coding::schema::series::dsl::*;

            let connection = establish_connection();

            let results = series
                .filter(id.eq(uid))
                .limit(1)
                .load::<Series>(&connection)
                .expect("Error loading series");

            if results.len() == 1 {
                return results[0].name.clone();
            } else {
                return "".to_string();
            }
        }
        None => "".to_string(),
    }
}

#[derive(Serialize)]
struct WatchContext {
    uuid: String,
    series_title: String,
    title: String,
    description: String,
    username: String,
}

#[get("/watch/<uuid>")]
fn watch_as_member(_member: Member, user: User, uuid: String) -> Result<Template, Redirect> {
    match get_video_data_from_uuid(uuid) {
        Ok(video) => {
            let context = WatchContext {
                uuid: video.uuid,
                series_title: get_series_title(video.series),
                title: video.title,
                description: video.description,
                username: user.username,
            };
            Ok(Template::render("watch", &context))
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

#[get("/watch/<uuid>", rank = 2)]
fn watch_as_user(user: User, uuid: String) -> Result<Template, Redirect> {
    match get_video_data_from_uuid(uuid) {
        Ok(video) => {
            let context = WatchContext {
                uuid: video.uuid,
                series_title: get_series_title(video.series),
                title: video.title,
                description: video.description,
                username: user.username,
            };
            Ok(Template::render("watch", &context))
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

#[get("/watch/<_uuid>", rank = 3)]
fn watch_nouser(_uuid: String) -> Redirect {
    Redirect::to("/login")
}

#[get("/video/<uuid>")]
fn video(_user: Member, uuid: String) -> Result<Content<File>, String> {
    match File::open(format!("videos/{}.mp4", uuid)) {
        Ok(file) => Ok(Content(ContentType::new("video", "mp4"), file)),
        Err(err) => Err(err.to_string()),
    }
}

#[get("/thumbnail/<uuid>")]
fn thumbnail(uuid: String) -> Result<NamedFile, String> {
    match NamedFile::open(format!("thumbnails/{}.png", uuid)) {
        Ok(file) => Ok(file),
        Err(err) => Err(err.to_string()),
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![
        thumbnail,
        watch_as_member,
        watch_as_user,
        watch_nouser,
        video
    ]
}
