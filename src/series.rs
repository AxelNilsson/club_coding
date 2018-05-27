use rocket::Route;
use diesel::prelude::*;
use club_coding::models::{Series, UsersViews, Videos};
use rocket_contrib::Template;
use database::DbConn;
use users::User;

pub fn get_series(connection: &DbConn) -> Vec<Series> {
    use club_coding::schema::series::dsl::*;

    match series
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .order(updated.asc())
        .load::<Series>(&**connection)
    {
        Ok(vec_of_series) => vec_of_series,
        Err(_) => vec![],
    }
}

#[derive(Serialize)]
pub struct PublicSeries {
    uuid: String,
    title: String,
    slug: String,
    description: String,
    price: i32,
}

pub fn get_last_10_series(connection: &DbConn) -> Vec<PublicSeries> {
    use club_coding::schema::series::dsl::*;

    match series
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .limit(10)
        .order(updated.asc())
        .load::<Series>(&**connection)
    {
        Ok(s_eries) => {
            let mut to_return: Vec<PublicSeries> = vec![];
            for serie in s_eries {
                to_return.push(PublicSeries {
                    uuid: serie.uuid,
                    title: serie.title,
                    slug: serie.slug,
                    description: serie.description,
                    price: serie.price,
                });
            }
            to_return
        }
        Err(_) => vec![],
    }
}

fn get_serie(connection: &DbConn, uid: &String) -> Option<Series> {
    use club_coding::schema::series::dsl::*;

    match series
        .filter(uuid.eq(uid))
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .first(&**connection)
    {
        Ok(serie) => Some(serie),
        Err(_) => None,
    }
}

#[derive(Serialize)]
pub struct PublicVideo {
    pub episode_number: Option<i32>,
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub watched: bool,
}

pub fn get_video_watched(connection: &DbConn, uid: i64, vid: i64) -> bool {
    use club_coding::schema::users_views::dsl::*;

    match users_views
        .filter(user_id.eq(uid))
        .filter(video_id.eq(vid))
        .first::<UsersViews>(&**connection)
    {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_videos(connection: &DbConn, uid: i64, sid: i64) -> Vec<PublicVideo> {
    use club_coding::schema::videos::dsl::*;

    match videos
        .filter(series.eq(sid))
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .order(episode_number.asc())
        .load::<Videos>(&**connection)
    {
        Ok(vec_of_videos) => {
            let mut to_return: Vec<PublicVideo> = vec![];
            for video in vec_of_videos {
                to_return.push(PublicVideo {
                    episode_number: video.episode_number,
                    uuid: video.uuid,
                    title: video.title,
                    description: video.description,
                    watched: get_video_watched(connection, uid, video.id),
                });
            }
            to_return
        }
        Err(_) => vec![],
    }
}

#[derive(Serialize)]
struct SerieStruct<'a> {
    header: &'a String,
    user: &'a User,
    uuid: String,
    title: &'a String,
    description: String,
    in_development: bool,
    price: i32,
    videos: Vec<PublicVideo>,
}

#[get("/<uuid>")]
fn serie(conn: DbConn, user: User, uuid: String) -> Option<Template> {
    match get_serie(&conn, &uuid) {
        Some(serie) => {
            let mut description = serie.description;
            description.retain(|c| c != '\\');
            let context = SerieStruct {
                header: &serie.title,
                user: &user,
                uuid: uuid,
                title: &serie.title,
                description: description,
                in_development: serie.in_development,
                price: serie.price,
                videos: get_videos(&conn, user.id, serie.id),
            };
            Some(Template::render("series", &context))
        }
        None => None,
    }
}

#[derive(Serialize)]
struct SerieNoLogin<'a> {
    header: &'a String,
    uuid: String,
    title: &'a String,
    description: String,
    in_development: bool,
    videos: Vec<PublicVideo>,
}

fn get_videos_nologin(connection: &DbConn, sid: i64) -> Vec<PublicVideo> {
    use club_coding::schema::videos::dsl::*;

    match videos
        .filter(series.eq(sid))
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .order(episode_number.asc())
        .load::<Videos>(&**connection)
    {
        Ok(v_ideos) => {
            let mut to_return: Vec<PublicVideo> = vec![];
            for video in v_ideos {
                to_return.push(PublicVideo {
                    episode_number: video.episode_number,
                    uuid: video.uuid,
                    title: video.title,
                    description: video.description,
                    watched: false,
                });
            }
            to_return
        }
        Err(_) => vec![],
    }
}

#[get("/<uuid>", rank = 2)]
fn serie_nologin(conn: DbConn, uuid: String) -> Option<Template> {
    match get_serie(&conn, &uuid) {
        Some(serie) => {
            let context = SerieNoLogin {
                header: &serie.title,
                uuid: uuid,
                title: &serie.title,
                description: serie.description,
                in_development: serie.in_development,
                videos: get_videos_nologin(&conn, serie.id),
            };
            Some(Template::render("series_nologin", &context))
        }
        None => None,
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![serie, serie_nologin]
}
