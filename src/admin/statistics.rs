use rocket_contrib::Template;
use users::get_users;
use admin::structs::{Administrator, LoggedInContext};
use series::database::get_series;
use videos::database::get_videos;
use rocket::Route;
use database::DbConn;

#[derive(Serialize)]
struct AdminContext<'a> {
    header: &'a str,
    user: Administrator,
    views_today: usize,
    videos_total: usize,
    series_total: usize,
    revenue_month: u64,
    paying_users: usize,
    total_users: usize,
}

#[get("/")]
pub fn index(conn: DbConn, user: Administrator) -> Template {
    let context = AdminContext {
        header: "Club Coding",
        user: user,
        views_today: 187232,
        videos_total: get_videos(&conn).len(),
        series_total: get_series(&conn).len(),
        revenue_month: 102230,
        paying_users: 123,
        total_users: get_users(&conn).len(),
    };
    Template::render("admin/index", &context)
}

#[get("/views")]
pub fn views(user: Administrator) -> Template {
    let context = LoggedInContext {
        header: "Club Coding",
        user: user,
    };
    Template::render("admin/views", &context)
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![index, views]
}
