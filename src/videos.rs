use rocket::Route;
use rocket_contrib::Template;
use rocket::response::{Flash, NamedFile, Redirect};
use club_coding::{create_new_user_series_access, create_new_user_view, establish_connection};
use club_coding::models::{Series, UsersSeriesAccess, UsersStripeCustomer, UsersViews, Videos};
use diesel::prelude::*;
use users::User;
use std;
use series::{get_video_watched, PublicVideo};
use rocket::request::FlashMessage;

pub fn get_videos() -> Vec<Videos> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();
    videos
        .order(created.asc())
        .load::<Videos>(&connection)
        .expect("Error loading videos")
}

fn get_video_data_from_uuid(uid: &String) -> Result<Videos, std::io::Error> {
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
    flash_name: String,
    flash_msg: String,
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

fn user_has_bought(sid: i64, uid: i64) -> bool {
    use club_coding::schema::users_series_access::dsl::*;
    let connection = establish_connection();

    let series = users_series_access
        .filter(user_id.eq(uid))
        .filter(series_id.eq(sid))
        .limit(1)
        .load::<UsersSeriesAccess>(&connection)
        .expect("Error loading user series");

    return series.len() == 1;
}

#[get("/watch/<uuid>")]
fn watch_as_user(
    user: User,
    flash: Option<FlashMessage>,
    uuid: String,
) -> Result<Template, Redirect> {
    match get_video_data_from_uuid(&uuid) {
        Ok(video) => {
            let videos: Vec<PublicVideo> = match video.series {
                Some(series_id) => get_videos_of_series(user.id, series_id),
                None => vec![],
            };
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            let context = WatchContext {
                uuid: video.uuid,
                series_title: get_series_title(video.series),
                title: video.title,
                description: video.description,
                user: &user,
                vimeo_id: video.vimeo_id,
                videos: videos,
                flash_name: name,
                flash_msg: msg,
            };
            if video.membership_only {
                match video.series {
                    Some(series_id) => {
                        if user_has_bought(series_id, user.id) {
                            create_new_view(video.id, user.id);
                            Ok(Template::render("watch_member", &context))
                        } else {
                            Ok(Template::render("watch_nomember", &context))
                        }
                    }
                    None => Ok(Template::render("watch_nomember", &context)),
                }
            } else {
                Ok(Template::render("watch_member", &context))
            }
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

fn get_videos_of_series_nologin(sid: i64) -> Vec<PublicVideo> {
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
            watched: false,
        });
    }
    to_return
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
fn watch_nouser(flash: Option<FlashMessage>, uuid: String) -> Result<Template, Redirect> {
    match get_video_data_from_uuid(&uuid) {
        Ok(video) => {
            let (name, msg) = match flash {
                Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
                None => ("".to_string(), "".to_string()),
            };
            let videos: Vec<PublicVideo> = match video.series {
                Some(series_id) => get_videos_of_series_nologin(series_id),
                None => vec![],
            };
            let context = WatchNoUser {
                uuid: video.uuid,
                series_title: get_series_title(video.series),
                title: video.title,
                description: video.description,
                videos: videos,
                flash_name: name,
                flash_msg: msg,
            };
            Ok(Template::render("watch_nologin", &context))
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

#[get("/thumbnail/<uuid>")]
fn thumbnail(uuid: String) -> Option<NamedFile> {
    match NamedFile::open(format!("thumbnails/{}.png", uuid)) {
        Ok(file) => Some(file),
        Err(_) => None,
    }
}

fn get_customer(uid: i64) -> Option<UsersStripeCustomer> {
    use club_coding::schema::users_stripe_customer::dsl::*;

    let connection = establish_connection();

    let user: Vec<UsersStripeCustomer> = users_stripe_customer
        .filter(user_id.eq(uid))
        .limit(1)
        .load::<UsersStripeCustomer>(&connection)
        .expect("Error loading users");

    if user.len() == 1 {
        Some(user[0].clone())
    } else {
        None
    }
}

fn get_serie(sid: i64) -> Series {
    use club_coding::schema::series::dsl::*;

    let connection = establish_connection();
    series
        .filter(id.eq(sid))
        .first(&connection)
        .expect("Error loading serie")
}

#[get("/watch/<uuid>/buy")]
fn buy_serie(user: User, uuid: String) -> Result<Flash<Redirect>, Redirect> {
    match get_video_data_from_uuid(&uuid) {
        Ok(video) => {
            match video.series {
                Some(series_id) => {
                    match get_customer(user.id) {
                        Some(stripe_customer) => {
                            let serie = get_serie(series_id);

                            match stripe_customer.default_source {
                                Some(customer_source) => {
                                    // Create the customer
                                    let client =
                                        stripe::Client::new("sk_test_cztFtKdeTEnlPLL6DpvkbjFf");

                                    let charge = stripe::Charge::create(
                                        &client,
                                        stripe::ChargeParams {
                                            amount: Some(serie.price as u64),
                                            currency: Some(stripe::Currency::USD),
                                            application_fee: None,
                                            capture: None,
                                            description: None,
                                            destination: None,
                                            fraud_details: None,
                                            transfer_group: None,
                                            on_behalf_of: None,
                                            metadata: None,
                                            receipt_email: None,
                                            shipping: None,
                                            customer: Some(stripe_customer.uuid),
                                            source: Some(stripe::CustomerSource::Token(
                                                &customer_source,
                                            )),
                                            statement_descriptor: None,
                                        },
                                    ).unwrap();
                                    println!("{:?}", charge);
                                    let connection = establish_connection();
                                    create_new_user_series_access(
                                        &connection,
                                        user.id,
                                        series_id,
                                        true,
                                    );
                                    Ok(Flash::success(
                                        Redirect::to(&format!("/watch/{}", uuid)),
                                        "Series unlocked! Congratulations!",
                                    ))
                                }
                                None => Ok(Flash::error(
                                    Redirect::to(&format!("/watch/{}", uuid)),
                                    "An error occured, please try again later.",
                                )),
                            }
                        }
                        None => Err(Redirect::to(&format!("/card/add/{}", uuid))),
                    }
                }
                None => Err(Redirect::to("/")),
            }
        }
        Err(_video_not_found) => Err(Redirect::to("/")),
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![thumbnail, watch_as_user, watch_nouser, buy_serie]
}
