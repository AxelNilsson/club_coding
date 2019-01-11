use rocket::Route;
use rocket::request::FlashMessage;
use rocket_contrib::templates::Template;
use rocket_contrib::json::Json;
use users::User;
use series::PublicSeries;
use series::database::get_last_10_series;
use database::{DbConn, RedisConnection};
use structs::{Context, LoggedInContext};
use rocket::response::NamedFile;
use club_coding::create_new_newsletter_subscriber;

#[cfg(test)]
mod tests;

/// Context for rendering tera templates
/// for the logged in index endpoint.
#[derive(Serialize)]
struct IndexLoggedInContext<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    header: &'a str,
    /// The user struct used by templates.
    /// For example the username for the toolbar.
    user: User,
    /// Flash name if the request is redirected
    /// with one.
    flash_name: String,
    /// Flash message if the request is redirected
    /// with one.
    flash_msg: String,
    /// The last 10 series on the website.
    series: Vec<PublicSeries>,
}

/// Context for rendering tera templates
/// for the not logged in index endpoint.
#[derive(Serialize)]
struct IndexContext<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    header: &'a str,
    /// Flash name if the request is redirected
    /// with one.
    flash_name: String,
    /// Flash message if the request is redirected
    /// with one.
    flash_msg: String,
    /// The last 10 series on the website.
    series: Vec<PublicSeries>,
}

/// GET Endpoint for the index page.
/// Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is not
/// logged in it forwards the request.
/// Takes in an optional FlashMessage
/// incase there is one.
/// Responds with the Home Template
/// in the pages folder.
#[get("/")]
fn index(
    mysql_conn: DbConn,
    redis_conn: RedisConnection,
    user: User,
    flash: Option<FlashMessage>,
) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = IndexLoggedInContext {
        header: "Club Coding",
        user: user,
        flash_name: name,
        flash_msg: msg,
        series: get_last_10_series(&mysql_conn, redis_conn),
    };
    Template::render("pages/home", &context)
}

/// GET Endpoint for the index page.
/// This endpoint will kick in
/// if the user is not logged in.
/// Takes in an optional FlashMessage
/// incase there is one.
/// Responds with the Index Template
/// in the pages folder.
#[get("/", rank = 2)]
fn index_nouser(
    mysql_conn: DbConn,
    redis_conn: RedisConnection,
    flash: Option<FlashMessage>,
) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };
    let context = IndexContext {
        header: "Club Coding",
        flash_name: name,
        flash_msg: msg,
        series: get_last_10_series(&mysql_conn, redis_conn),
    };
    Template::render("pages/index", &context)
}

/// Struct for accepting a new
/// subscriber to the newsletter.
#[derive(Deserialize, Serialize)]
pub struct NewSubscriber {
    /// The email of the new
    /// subscriber.
    email: String,
}

/// POST Endpoint for the page to subscribe
/// to the newsletter.
/// It requires an email parameter as defined in
/// the NewSubscriber Struct.
/// If everything is successful, it will insert
/// the new subscriber to the table in the database
/// and return an OK. If it fails (due to the database)
/// not working it will response with an Err.
#[post("/subscribe", format = "application/json", data = "<data>")]
fn subscribe(conn: DbConn, data: Json<NewSubscriber>) -> Result<(), ()> {
    match create_new_newsletter_subscriber(&*conn, &data.0.email) {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

/// GET Endpoint for the Terms of Service
/// page. Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is not
/// logged in it forwards the request.
/// Responds with the Terms of Service
/// Template in the pages folder.
#[get("/terms_of_service")]
fn terms_of_service(user: User) -> Template {
    let context = LoggedInContext {
        header: "Terms of Service",
        user: user,
    };
    Template::render("pages/terms_of_service", &context)
}

/// GET Endpoint for the Terms of Service
/// page. This endpoint will kick in
/// if the user is not logged in.
/// Responds with the Terms of Service
/// Template in the pages folder.
#[get("/terms_of_service", rank = 2)]
fn terms_of_service_nologin() -> Template {
    let context = Context {
        header: "Terms of Service",
    };
    Template::render("pages/terms_of_service_nologin", &context)
}

/// GET Endpoint for the Cookie Policy
/// page. Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is not
/// logged in it forwards the request.
/// Responds with the Cookie Policy
/// Template in the pages folder.
#[get("/cookie_policy")]
fn cookie_policy(user: User) -> Template {
    let context = LoggedInContext {
        header: "Cookie Policy",
        user: user,
    };
    Template::render("pages/cookie_policy", &context)
}

/// GET Endpoint for the Cookie Policy
/// page. This endpoint will kick in
/// if the user is not logged in.
/// Responds with the Cookie Policy
/// Template in the pages folder.
#[get("/cookie_policy", rank = 2)]
fn cookie_policy_nologin() -> Template {
    let context = Context {
        header: "Cookie Policy",
    };
    Template::render("pages/cookie_policy_nologin", &context)
}

/// GET Endpoint for the Privacy Policy
/// page. Endpoints checks if the user is
/// logged in by using the user
/// request guard. If the user is not
/// logged in it forwards the request.
/// Responds with the Privacy Policy
/// Template in the pages folder.
#[get("/privacy_policy")]
fn privacy_policy(user: User) -> Template {
    let context = LoggedInContext {
        header: "Privacy Policy",
        user: user,
    };
    Template::render("pages/privacy_policy", &context)
}

/// GET Endpoint for the Privacy Policy
/// page. This endpoint will kick in
/// if the user is not logged in.
/// Responds with the Privacy Policy
/// Template in the pages folder.
#[get("/privacy_policy", rank = 2)]
fn privacy_policy_nologin() -> Template {
    let context = Context {
        header: "Privacy Policy",
    };
    Template::render("pages/privacy_policy_nologin", &context)
}

/// Only to be used in dev for thumbails so
/// we don't have to install NGiNX in dev.
/// Checks in the thumbnails directory for
/// thumbnails. Responds with some thumbnail
/// if found. Otherwise with a None.
#[get("/thumbnail/<uuid>")]
fn thumbnail(uuid: String) -> Option<NamedFile> {
    match NamedFile::open(format!("thumbnails/{}", uuid)) {
        Ok(file) => Some(file),
        Err(_) => None,
    }
}

/// Only to be used in dev for thumbails so
/// we don't have to install NGiNX in dev.
/// Checks in the images directory for
/// images. Responds with some image
/// if found. Otherwise with a None.
#[get("/img/<uuid>")]
fn images(uuid: String) -> Option<NamedFile> {
    match NamedFile::open(format!("images/{}", uuid)) {
        Ok(file) => Some(file),
        Err(_) => None,
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![
        index,
        index_nouser,
        subscribe,
        terms_of_service,
        terms_of_service_nologin,
        cookie_policy,
        cookie_policy_nologin,
        privacy_policy,
        privacy_policy_nologin,
        thumbnail,
        images
    ]
}
