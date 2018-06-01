use rocket_contrib::Template;
use admin::structs::Administrator;
use rocket::response::Redirect;
use club_coding::models::{Series, Videos};
use club_coding::create_new_video;
use database::DbConn;
use chrono::NaiveDateTime;
use rocket_contrib::Json;
use diesel::prelude::*;
use rocket::request::Form;
use admin::series::get_all_series;
use admin::series::Serie;
use admin::generate_token;
use admin::create_slug;
use admin::series::SeriesContext;
use rocket::Route;

#[derive(Serialize)]
struct Video {
    uuid: String,
    name: String,
    views: u64,
    comments: u64,
    serie: String,
    membership: bool,
    published: bool,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

#[derive(Serialize)]
struct VideosContext {
    header: String,
    user: Administrator,
    videos: Vec<Video>,
}

fn get_all_videos(connection: &DbConn) -> Vec<Video> {
    use club_coding::schema::videos::dsl::*;

    match videos.load::<Videos>(&**connection) {
        Ok(result) => {
            let mut ret: Vec<Video> = vec![];

            for video in result {
                use club_coding::schema::series::dsl::*;
                let series_name = match series.find(video.serie_id).first::<Series>(&**connection) {
                    Ok(serie) => serie.title,
                    Err(_) => "".to_string(),
                };

                ret.push(Video {
                    uuid: video.uuid,
                    name: video.title,
                    views: 0,
                    comments: 0,
                    serie: series_name,
                    membership: video.membership_only,
                    published: video.published,
                    created: video.created,
                    updated: video.updated,
                })
            }
            ret
        }
        Err(_) => vec![],
    }
}

#[get("/videos")]
pub fn videos(conn: DbConn, user: Administrator) -> Template {
    let context = VideosContext {
        header: "Club Coding".to_string(),
        user: user,
        videos: get_all_videos(&conn),
    };
    Template::render("admin/videos", &context)
}

#[get("/videos/new")]
pub fn new_video(conn: DbConn, user: Administrator) -> Template {
    let context = SeriesContext {
        header: "Club Coding".to_string(),
        user: user,
        series: get_all_series(&conn),
    };
    Template::render("admin/new_video", &context)
}

#[derive(FromForm)]
pub struct NewVideo {
    title: String,
    description: String,
    vimeo_id: String,
    serie: String,
    membership_only: bool,
}

fn get_series_from_uuid(connection: &DbConn, uid: String) -> i64 {
    use club_coding::schema::series::dsl::*;

    match series.filter(uuid.eq(uid)).first::<Series>(&**connection) {
        Ok(serie) => serie.id,
        Err(_) => 0,
    }
}

fn get_highest_episode_from_series(connection: &DbConn, sid: i64) -> i32 {
    use club_coding::schema::videos::dsl::*;

    match videos
        .filter(serie_id.eq(sid))
        .order(episode_number.desc())
        .first::<Videos>(&**connection)
    {
        Ok(video) => video.episode_number + 1,
        Err(_) => 1,
    }
}

#[post("/videos/new", data = "<video>")]
pub fn insert_new_video(
    conn: DbConn,
    _user: Administrator,
    video: Form<NewVideo>,
) -> Result<Redirect, Redirect> {
    let new_video: NewVideo = video.into_inner();
    let slug = create_slug(&new_video.title);
    let series: i64 = get_series_from_uuid(&conn, new_video.serie);
    let episode_number: i32 = get_highest_episode_from_series(&conn, series);
    match generate_token(24) {
        Ok(uuid) => match create_new_video(
            &conn,
            &uuid,
            &new_video.title,
            &slug,
            &new_video.description,
            false,
            new_video.membership_only,
            series,
            episode_number,
            false,
            &new_video.vimeo_id,
        ) {
            Ok(_) => Ok(Redirect::to(&format!("/admin/videos/edit/{}", uuid))),
            Err(_) => Ok(Redirect::to(&format!("/admin/videos/edit/{}", uuid))),
        },
        Err(_) => Err(Redirect::to("/admin/videos/new")),
    }
}

#[derive(Serialize)]
struct EditVideo<'a> {
    header: String,
    user: Administrator,
    uuid: &'a str,
    series: Vec<Serie>,
    video: UpdateVideo,
}

fn get_video(connection: &DbConn, uid: &str) -> Option<Videos> {
    use club_coding::schema::videos::dsl::*;

    match videos.filter(uuid.eq(uid)).first::<Videos>(&**connection) {
        Ok(result) => Some(result),
        Err(_) => return None,
    }
}

fn get_serie_from_video(connection: &DbConn, sid: i64) -> String {
    use club_coding::schema::series::dsl::*;

    match series.find(sid).first::<Series>(&**connection) {
        Ok(serie) => serie.uuid,
        Err(_) => "".to_string(),
    }
}

#[get("/videos/edit/<uuid>")]
pub fn edit_video(conn: DbConn, uuid: String, user: Administrator) -> Option<Template> {
    match get_video(&conn, &uuid) {
        Some(video) => {
            let serie_title = get_serie_from_video(&conn, video.serie_id);
            let context = EditVideo {
                header: "Club Coding".to_string(),
                user: user,
                uuid: &uuid,
                series: get_all_series(&conn),
                video: UpdateVideo {
                    title: video.title,
                    description: video.description,
                    vimeo_id: video.vimeo_id,
                    membership: video.membership_only,
                    published: video.published,
                    serie: serie_title,
                },
            };
            Some(Template::render("admin/edit_video", &context))
        }
        None => None,
    }
}

#[derive(Deserialize, Serialize)]
pub struct UpdateVideo {
    title: String,
    description: String,
    vimeo_id: String,
    membership: bool,
    published: bool,
    serie: String,
}

#[post("/videos/edit/<uid>", format = "application/json", data = "<data>")]
pub fn update_video(
    conn: DbConn,
    uid: String,
    _user: Administrator,
    data: Json<UpdateVideo>,
) -> Result<(), ()> {
    use club_coding::schema::videos::dsl::*;

    match diesel::update(videos.filter(uuid.eq(uid)))
        .set((
            title.eq(&data.0.title),
            description.eq(&data.description),
            vimeo_id.eq(&data.vimeo_id),
            membership_only.eq(data.0.membership),
            published.eq(data.0.published),
        ))
        .execute(&*conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![
        videos,
        new_video,
        insert_new_video,
        edit_video,
        update_video
    ]
}
