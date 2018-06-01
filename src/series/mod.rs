pub mod database;

use rocket::Route;
use rocket_contrib::Template;
use database::DbConn;
use users::User;

#[derive(Serialize)]
pub struct PublicSeries {
    uuid: String,
    title: String,
    slug: String,
    description: String,
    price: i32,
}

#[derive(Serialize)]
pub struct PublicVideo {
    pub episode_number: i32,
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub watched: bool,
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
    match database::get_serie(&conn, &uuid) {
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
                videos: database::get_videos(&conn, user.id, serie.id),
            };
            Some(Template::render("series/series", &context))
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

#[get("/<uuid>", rank = 2)]
fn serie_nologin(conn: DbConn, uuid: String) -> Option<Template> {
    match database::get_serie(&conn, &uuid) {
        Some(serie) => {
            let context = SerieNoLogin {
                header: &serie.title,
                uuid: uuid,
                title: &serie.title,
                description: serie.description,
                in_development: serie.in_development,
                videos: database::get_videos_nologin(&conn, serie.id),
            };
            Some(Template::render("series/series_nologin", &context))
        }
        None => None,
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![serie, serie_nologin]
}
