use rocket::Route;
use rocket_contrib::Template;
use rocket::response::{NamedFile, Redirect};
use club_coding::{create_new_user_view, establish_connection};
use club_coding::models::{Series, UsersViews, Videos};
use diesel::prelude::*;
use member::Member;
use users::User;
use std;
use series::{get_video_watched, PublicVideo};

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
                return results[0].title.clone();
            } else {
                return "".to_string();
            }
        }
        None => "".to_string(),
    }
}

fn get_videos_of_series(uid: i64, sid: i64) -> Vec<PublicVideo> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();
    let v_ideos = videos
        .filter(series.eq(sid))
        .order(episode_number.asc())
        .load::<Videos>(&connection)
        .expect("Error loading users");

    let mut to_return: Vec<PublicVideo> = vec![];
    for video in v_ideos {
        to_return.push(PublicVideo {
            episode_number: video.episode_number,
            uuid: video.uuid,
            title: video.title,
            description: video.description,
            watched: get_video_watched(uid, video.id),
        });
    }
    to_return
}

#[derive(Serialize)]
struct WatchContext<'a> {
    uuid: String,
    series_title: String,
    title: String,
    description: String,
    user: &'a User,
    vimeo_id: String,
    videos: Vec<PublicVideo>,
}

fn create_new_view(vid: i64, uid: i64) {
    use club_coding::schema::users_views::dsl::*;
    let connection = establish_connection();

    let view = users_views
        .filter(user_id.eq(uid))
        .filter(video_id.eq(vid))
        .load::<UsersViews>(&connection)
        .expect("Error loading user views");

    if view.len() == 0 {
        create_new_user_view(&connection, uid, vid);
    }
}

#[get("/watch/<uuid>")]
fn watch_as_member(_member: Member, user: User, uuid: String) -> Result<Template, Redirect> {
    match get_video_data_from_uuid(uuid) {
        Ok(video) => {
            let videos: Vec<PublicVideo> = match video.series {
                Some(series_id) => get_videos_of_series(user.id, series_id),
                None => vec![],
            };
            let context = WatchContext {
                uuid: video.uuid,
                series_title: get_series_title(video.series),
                title: video.title,
                description: video.description,
                user: &user,
                vimeo_id: video.vimeo_id,
                videos: videos,
            };
            create_new_view(video.id, user.id);
            Ok(Template::render("watch", &context))
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

/*#[get("/watch/<uuid>", rank = 2)]
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
}*/

#[get("/watch/<_uuid>", rank = 2)]
fn watch_nouser(_uuid: String) -> Redirect {
    Redirect::to("/login")
}

#[get("/thumbnail/<uuid>")]
fn thumbnail(uuid: String) -> Result<NamedFile, String> {
    match NamedFile::open(format!("thumbnails/{}.png", uuid)) {
        Ok(file) => Ok(file),
        Err(err) => Err(err.to_string()),
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![thumbnail, watch_as_member, watch_nouser]
}
