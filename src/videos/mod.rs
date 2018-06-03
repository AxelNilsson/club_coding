pub mod database;
pub mod charge;

use rocket::Route;
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use users::User;
use series::PublicVideo;
use rocket::request::FlashMessage;
use database::{DbConn, RedisConnection};
use structs::{PostmarkToken, StripeToken};
use rocket::State;
use videos::charge::charge_card;
use series;

/// Context for rendering tera templates
/// for logged in watch endpoints.
#[derive(Serialize)]
struct WatchContext<'a> {
    /// UUID of the video being watched.
    uuid: String,
    /// Title of the series being watched.
    series_title: String,
    /// Price of the series.
    price: i32,
    /// Title of the Video.
    title: String,
    /// Description of the Video.
    description: String,
    /// The user struct used by templates.
    /// For example the username for the toolbar.
    user: &'a User,
    /// The Vimeo ID of the video being watched.
    vimeo_id: String,
    /// Boolean of if the series is
    /// in development or not.
    in_development: bool,
    /// A Vector of the Videos in the same series
    /// as the one currently watched.
    videos: Vec<PublicVideo>,
    /// Flash name if the request is redirected
    /// with one.
    flash_name: String,
    /// Flash message if the request is redirected
    /// with one.
    flash_msg: String,
}

/// GET Endpoint for the page to watch
/// a video. Endpoints checks if the
/// user is logged in by using the
/// user request guard. If the user
/// is not logged in it forwards
/// the request.
/// Takes in an optional FlashMessage
/// incase there is one.
/// The endpoint checks if the video
/// requires that the series is bought and if
/// it requires that it will check if the user
/// has the permission. If the user does not
/// have the persmission it will respond with a
/// buy page (Watch No Member in the videos folder).
/// If the user does have the persmission or the
/// series does not require it, it will respond
/// with the Watch Member Template in the videos
/// folder.
#[get("/watch/<uuid>")]
fn watch_as_user(
    mysql_conn: DbConn,
    redis_conn: RedisConnection,
    user: User,
    flash: Option<FlashMessage>,
    uuid: String,
) -> Result<Template, Redirect> {
    match database::get_video_data_from_uuid(&mysql_conn, &uuid) {
        Ok(video) => {
            let videos: Vec<PublicVideo> =
                series::database::get_videos(&mysql_conn, redis_conn, user.id, video.serie_id);
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            // Javascript needs double backslashes for edit pages.
            // That's the way it's stored in the DB so we need to
            // remove it here.
            let mut description = video.description;
            description.retain(|c| c != '\\');
            let mut context = WatchContext {
                uuid: video.uuid,
                series_title: video.series_title,
                price: video.price,
                title: video.title,
                description: description,
                user: &user,
                vimeo_id: video.vimeo_id,
                in_development: video.in_development,
                videos: videos,
                flash_name: name,
                flash_msg: msg,
            };
            database::create_new_view(&mysql_conn, video.id, user.id);
            if video.membership_only {
                if !database::user_has_bought(&mysql_conn, video.serie_id, user.id) {
                    return Ok(Template::render("videos/watch_nomember", &context));
                }
            }
            Ok(Template::render("videos/watch_member", &context))
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

#[derive(Serialize)]
struct WatchNoUser {
    /// UUID of the video being watched.
    uuid: String,
    /// Title of the series being watched.
    series_title: String,
    /// Title of the Video.
    title: String,
    /// Description of the Video.
    description: String,
    /// Boolean of if the series is
    /// in development or not.
    in_development: bool,
    /// A Vector of the Videos in the same series
    /// as the one currently watched.
    videos: Vec<PublicVideo>,
    /// Flash name if the request is redirected
    /// with one.
    flash_name: String,
    /// Flash message if the request is redirected
    /// with one.
    flash_msg: String,
}

/// GET Endpoint for the page to watch
/// a video. This endpoint will kick in
/// if the user is not logged in.
/// Takes in an optional FlashMessage
/// incase there is one.
/// Responds with the Watch No Login
/// Template in the videos folder.
#[get("/watch/<uuid>", rank = 2)]
fn watch_nouser(
    mysql_conn: DbConn,
    redis_conn: RedisConnection,
    flash: Option<FlashMessage>,
    uuid: String,
) -> Result<Template, Redirect> {
    match database::get_video_data_from_uuid(&mysql_conn, &uuid) {
        Ok(video) => {
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            // Javascript needs double backslashes for edit pages.
            // That's the way it's stored in the DB so we need to
            // remove it here.
            let mut description = video.description;
            description.retain(|c| c != '\\');
            let videos: Vec<PublicVideo> =
                series::database::get_videos_nologin(&mysql_conn, redis_conn, video.serie_id);
            let context = WatchNoUser {
                uuid: video.uuid,
                series_title: video.series_title,
                title: video.title,
                description: description,
                in_development: video.in_development,
                videos: videos,
                flash_name: name,
                flash_msg: msg,
            };
            Ok(Template::render("videos/watch_nologin", &context))
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

/// GET Endpoint to buy a certain series as
/// defined by the video the series is in
/// specified by the UUID. Endpoints checks
/// if the user is logged in by using the
/// user request guard. If the user
/// is not logged in it forwards
/// the request. The endpoint
/// checks if the user already has bought it
/// to avoid double purchases. If the series is already
/// bought it will redirect to the watch page for the
/// video. If the user doesn't have a card, it will redirect
/// to the add card page. If the user has a card and
/// has not already bought the series, it will perform
/// the purchase and redirect to the video.
#[get("/watch/<uuid>/buy")]
fn buy_serie(
    conn: DbConn,
    stripe_token: State<StripeToken>,
    postmark_token: State<PostmarkToken>,
    user: User,
    uuid: String,
) -> Result<Flash<Redirect>, Redirect> {
    match database::get_video_data_from_uuid(&conn, &uuid) {
        Ok(video) => {
            if !database::user_has_bought(&conn, video.serie_id, user.id) {
                match database::get_customer(&conn, user.id) {
                    Some(stripe_customer) => match charge_card(
                        &conn,
                        &stripe_token.secret_key,
                        &postmark_token.0,
                        video.serie_id,
                        &user,
                        &stripe_customer,
                    ) {
                        Ok(_) => Ok(Flash::success(
                            Redirect::to(&format!("/watch/{}", uuid)),
                            "Series unlocked! Congratulations!",
                        )),
                        Err(_) => Ok(Flash::error(
                            Redirect::to(&format!("/watch/{}", uuid)),
                            "An error occured, please try again later.",
                        )),
                    },
                    None => Err(Redirect::to(&format!("/card/add/{}", uuid))),
                }
            } else {
                Err(Redirect::to(&format!("/watch/{}", uuid)))
            }
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![watch_as_user, watch_nouser, buy_serie]
}
