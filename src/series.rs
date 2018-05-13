use rocket::Route;
use club_coding::establish_connection;
use diesel::prelude::*;
use club_coding::models::{Series, Videos};
use rocket_contrib::Template;
use users::User;

pub fn get_series() -> Vec<Series> {
    use club_coding::schema::series::dsl::*;

    let connection = establish_connection();
    series
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .order(updated.asc())
        .load::<Series>(&connection)
        .expect("Error loading users")
}

#[derive(Serialize)]
pub struct PublicSeries {
    uuid: String,
    title: String,
    slug: String,
    description: String,
}

pub fn get_last_10_series() -> Vec<PublicSeries> {
    use club_coding::schema::series::dsl::*;

    let connection = establish_connection();
    let s_eries = series
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .limit(10)
        .order(updated.asc())
        .load::<Series>(&connection)
        .expect("Error loading users");

    let mut to_return: Vec<PublicSeries> = vec![];
    for serie in s_eries {
        to_return.push(PublicSeries {
            uuid: serie.uuid,
            title: serie.title,
            slug: serie.slug,
            description: serie.description,
        });
    }
    to_return
}

fn get_serie(uid: String) -> Series {
    use club_coding::schema::series::dsl::*;

    let connection = establish_connection();
    series
        .filter(uuid.eq(uid))
        .first(&connection)
        .expect("Error loading serie")
}

#[derive(Serialize)]
pub struct PublicVideo {
    pub episode_number: Option<i32>,
    pub uuid: String,
    pub title: String,
    pub description: String,
}

fn get_videos(uid: i64) -> Vec<PublicVideo> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();
    let v_ideos = videos
        .filter(series.eq(uid))
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
        });
    }
    to_return
}

#[derive(Serialize)]
struct SerieStruct {
    header: String,
    username: String,
    uuid: String,
    title: String,
    description: String,
    videos: Vec<PublicVideo>,
}

#[get("/<uuid>")]
fn serie(user: User, uuid: String) -> Template {
    let serie = get_serie(uuid.clone());
    let mut description = serie.description;
    description.retain(|c| c != '\\');
    let context = SerieStruct {
        header: serie.title.clone(),
        username: user.username,
        uuid: uuid,
        title: serie.title,
        description: description,
        videos: get_videos(serie.id),
    };
    Template::render("series", &context)
}

pub fn endpoints() -> Vec<Route> {
    routes![serie]
}
