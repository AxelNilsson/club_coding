use rocket_contrib::Template;
use admin::structs::Administrator;
use rocket::response::Redirect;
use club_coding::models::{Series, Videos};
use club_coding::{create_new_video, establish_connection};
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
    serie: Option<String>,
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

fn get_all_videos() -> Vec<Video> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();
    let result = videos
        .load::<Videos>(&connection)
        .expect("Error loading videos");

    let mut ret: Vec<Video> = vec![];

    for video in result {
        let series_name: Option<String> = match video.series {
            Some(serie_id) => {
                use club_coding::schema::series::dsl::*;

                let serie: Series = series
                    .find(serie_id)
                    .first(&connection)
                    .expect("Unable to find series");
                Some(serie.title)
            }
            None => None,
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

#[get("/videos")]
pub fn videos(user: Administrator) -> Template {
    let context = VideosContext {
        header: "Club Coding".to_string(),
        user: user,
        videos: get_all_videos(),
    };
    Template::render("admin/videos", &context)
}

#[get("/videos/new")]
pub fn new_video(user: Administrator) -> Template {
    let context = SeriesContext {
        header: "Club Coding".to_string(),
        user: user,
        series: get_all_series(),
    };
    Template::render("admin/new_video", &context)
}

#[derive(FromForm)]
pub struct NewVideo {
    title: String,
    description: String,
    vimeo_id: String,
    serie: Option<String>,
    membership_only: bool,
}

fn get_series_from_uuid(uid: Option<String>) -> Option<i64> {
    match uid {
        Some(uid) => {
            use club_coding::schema::series::dsl::*;
            let connection = establish_connection();

            let serie: Series = series
                .filter(uuid.eq(uid))
                .first(&connection)
                .expect("Unable to find series");

            Some(serie.id)
        }
        None => None,
    }
}

fn get_highest_episode_from_series(series_id: Option<i64>) -> Option<i32> {
    match series_id {
        Some(series_id) => {
            use club_coding::schema::videos::dsl::*;
            let connection = establish_connection();

            let video: Videos = videos
                .filter(series.eq(series_id))
                .order(episode_number.desc())
                .first(&connection)
                .expect("Unable to find videos");

            match video.episode_number {
                Some(episode) => Some(episode + 1),
                None => None,
            }
        }
        None => None,
    }
}

#[post("/videos/new", data = "<video>")]
pub fn insert_new_video(_user: Administrator, video: Form<NewVideo>) -> Result<Redirect, Redirect> {
    let new_video: NewVideo = video.into_inner();
    let slug = create_slug(&new_video.title);
    let connection = establish_connection();
    let series: Option<i64> = get_series_from_uuid(new_video.serie);
    let episode_number: Option<i32> = get_highest_episode_from_series(series);
    match generate_token(24) {
        Ok(uuid) => {
            create_new_video(
                &connection,
                uuid.clone(),
                new_video.title,
                slug,
                new_video.description,
                false,
                new_video.membership_only,
                series,
                episode_number,
                false,
                new_video.vimeo_id,
            );
            Ok(Redirect::to(&format!("/admin/videos/edit/{}", uuid)))
        }
        Err(_) => Err(Redirect::to("/admin/videos/new")),
    }
}

#[derive(Serialize)]
struct EditVideo {
    header: String,
    user: Administrator,
    uuid: String,
    series: Vec<Serie>,
    video: UpdateVideo,
}

fn get_video(uid: String) -> Option<Videos> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();
    let result = videos
        .filter(uuid.eq(uid))
        .limit(1)
        .load::<Videos>(&connection)
        .expect("Error loading users");

    if result.len() == 1 {
        return Some(result[0].clone());
    } else {
        return None;
    }
}

fn get_serie_from_video(series_id: Option<i64>) -> String {
    match series_id {
        Some(sid) => {
            use club_coding::schema::series::dsl::*;

            let connection = establish_connection();
            let serie: Series = series
                .find(sid)
                .first(&connection)
                .expect("Unable to find series");

            serie.uuid
        }
        None => "".to_string(),
    }
}

#[get("/videos/edit/<uuid>")]
pub fn edit_video(uuid: String, user: Administrator) -> Option<Template> {
    match get_video(uuid.clone()) {
        Some(video) => {
            let context = EditVideo {
                header: "Club Coding".to_string(),
                user: user,
                uuid: uuid,
                series: get_all_series(),
                video: UpdateVideo {
                    title: video.title,
                    description: video.description,
                    vimeo_id: video.vimeo_id,
                    membership: video.membership_only,
                    published: video.published,
                    serie: get_serie_from_video(video.series),
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
pub fn update_video(uid: String, _user: Administrator, data: Json<UpdateVideo>) -> Result<(), ()> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();

    match diesel::update(videos.filter(uuid.eq(uid)))
        .set((
            title.eq(data.0.title.clone()),
            description.eq(data.description.clone()),
            vimeo_id.eq(data.vimeo_id.clone()),
            membership_only.eq(data.0.membership),
            published.eq(data.0.published),
        ))
        .execute(&connection)
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
