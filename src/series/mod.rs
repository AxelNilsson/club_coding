pub mod database;

use rocket::Route;
use rocket_contrib::Template;
use database::{DbConn, RedisConnection};
use users::User;

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize)]
pub struct PublicSeries {
    /// UUID of the series.
    uuid: String,
    /// Title of the series.
    title: String,
    /// Slug of the series
    slug: String,
    /// Description of the series.
    description: String,
    /// The price of the series defined
    /// by USD * 100 and therefor not a float.
    price: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PublicVideo {
    /// Unique ID of the Video in
    /// the database.
    pub id: i64,
    /// Episode number of the Video.
    pub episode_number: i32,
    /// UUID of the Video.
    pub uuid: String,
    /// Title of the Video.
    pub title: String,
    /// Description of the video.
    pub description: String,
    /// Boolean of whether the user
    /// has watched the video or not.
    pub watched: bool,
}

#[derive(Serialize)]
struct SerieStruct<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    header: &'a String,
    /// The user struct used by templates.
    /// For example the username for the toolbar.
    user: &'a User,
    /// UUID of the series.
    uuid: String,
    /// Title of the series.
    title: &'a String,
    /// Description of the series.
    description: String,
    /// Boolean of if the series is
    /// in development or not.
    in_development: bool,
    /// The price of the series defined
    /// by USD * 100 and therefor not a float.
    price: i32,
    /// A Vector of the Videos in the series
    videos: Vec<PublicVideo>,
}

/// GET Endpoint for the page of
/// a series. Endpoints checks if the
/// user is logged in by using the
/// user request guard. If the user
/// is not logged in it forwards
/// the request.
/// Responds with the Series Template in
/// the series folder.
#[get("/<uuid>")]
fn serie(
    mysql_conn: DbConn,
    redis_conn: RedisConnection,
    user: User,
    uuid: String,
) -> Option<Template> {
    match database::get_serie(&mysql_conn, &uuid) {
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
                videos: database::get_videos(&mysql_conn, redis_conn, user.id, serie.id),
            };
            Some(Template::render("series/series", &context))
        }
        None => None,
    }
}

#[derive(Serialize)]
struct SerieNoLogin<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    header: &'a String,
    /// UUID of the series.
    uuid: String,
    /// Title of the series.
    title: &'a String,
    /// Description of the series.
    description: String,
    /// Boolean of if the series is
    /// in development or not.
    in_development: bool,
    /// A Vector of the Videos in the series
    videos: Vec<PublicVideo>,
}

/// GET Endpoint for the page of
/// a series. This endpoint will kick
/// in if the user is not logged in.
/// Responds with the Series No Login
/// Template in the series folder.
#[get("/<uuid>", rank = 2)]
fn serie_nologin(
    mysql_conn: DbConn,
    redis_conn: RedisConnection,
    uuid: String,
) -> Option<Template> {
    match database::get_serie(&mysql_conn, &uuid) {
        Some(serie) => {
            let mut description = serie.description;
            description.retain(|c| c != '\\');
            let context = SerieNoLogin {
                header: &serie.title,
                uuid: uuid,
                title: &serie.title,
                description: description,
                in_development: serie.in_development,
                videos: database::get_videos_nologin(&mysql_conn, redis_conn, serie.id),
            };
            Some(Template::render("series/series_nologin", &context))
        }
        None => None,
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![serie, serie_nologin]
}
