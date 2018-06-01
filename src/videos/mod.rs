pub mod database;
pub mod charge;

use rocket::Route;
use rocket_contrib::Template;
use rocket::response::{Flash, Redirect};
use users::User;
use series::PublicVideo;
use rocket::request::FlashMessage;
use database::DbConn;
use structs::{PostmarkToken, StripeToken};
use rocket::State;
use videos::charge::charge_card;

#[derive(Serialize)]
struct WatchContext<'a> {
    uuid: String,
    series_title: String,
    price: i32,
    title: String,
    description: String,
    user: &'a User,
    vimeo_id: String,
    videos: Vec<PublicVideo>,
    flash_name: String,
    flash_msg: String,
}

#[get("/watch/<uuid>")]
fn watch_as_user(
    conn: DbConn,
    user: User,
    flash: Option<FlashMessage>,
    uuid: String,
) -> Result<Template, Redirect> {
    match database::get_video_data_from_uuid(&conn, &uuid) {
        Ok(video) => {
            let videos: Vec<PublicVideo> =
                database::get_videos_of_series(&conn, user.id, video.serie_id);
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            let mut context = WatchContext {
                uuid: video.uuid,
                series_title: video.series_title,
                price: video.price,
                title: video.title,
                description: video.description,
                user: &user,
                vimeo_id: video.vimeo_id,
                videos: videos,
                flash_name: name,
                flash_msg: msg,
            };
            database::create_new_view(&conn, video.id, user.id);
            if video.membership_only {
                if !database::user_has_bought(&conn, video.serie_id, user.id) {
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
    uuid: String,
    series_title: String,
    title: String,
    description: String,
    videos: Vec<PublicVideo>,
    flash_name: String,
    flash_msg: String,
}

#[get("/watch/<uuid>", rank = 2)]
fn watch_nouser(
    conn: DbConn,
    flash: Option<FlashMessage>,
    uuid: String,
) -> Result<Template, Redirect> {
    match database::get_video_data_from_uuid(&conn, &uuid) {
        Ok(video) => {
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            let videos: Vec<PublicVideo> =
                database::get_videos_of_series_nologin(&conn, video.serie_id);
            let context = WatchNoUser {
                uuid: video.uuid,
                series_title: video.series_title,
                title: video.title,
                description: video.description,
                videos: videos,
                flash_name: name,
                flash_msg: msg,
            };
            Ok(Template::render("videos/watch_nologin", &context))
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

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

pub fn endpoints() -> Vec<Route> {
    routes![watch_as_user, watch_nouser, buy_serie]
}
