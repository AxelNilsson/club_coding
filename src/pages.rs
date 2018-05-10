use rocket::Route;
use rocket::request::FlashMessage;
use rocket_contrib::Template;
use structs::Context;
use users::User;
use series::PublicSeries;
use series::get_last_10_series;

#[derive(Serialize)]
struct IndexLoggedInContext {
    header: String,
    username: String,
    flash_name: String,
    flash_msg: String,
    series: Vec<PublicSeries>,
}

#[get("/")]
fn index(flash: Option<FlashMessage>, user: User) -> Template {
    let (name, msg) = match flash {
        Some(flash) => (flash.name().to_string(), flash.msg().to_string()),
        None => ("".to_string(), "".to_string()),
    };

    let context = IndexLoggedInContext {
        header: "Club Coding".to_string(),
        username: user.username,
        flash_name: name,
        flash_msg: msg,
        series: get_last_10_series(),
    };
    Template::render("home", &context)
}

#[get("/", rank = 2)]
fn index_nouser() -> Template {
    let context = Context {
        header: "Club Coding".to_string(),
    };
    Template::render("index", &context)
}

pub fn endpoints() -> Vec<Route> {
    routes![index, index_nouser]
}
