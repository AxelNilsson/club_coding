use rocket_contrib::Template;
use users::{get_users, User};
use series::get_series;
use videos::get_videos;
use structs::LoggedInContext;

#[derive(Serialize)]
struct AdminContext {
    header: String,
    username: String,
    views_today: usize,
    videos_total: usize,
    series_total: usize,
    revenue_month: u64,
    paying_users: usize,
    total_users: usize,
}

#[get("/")]
pub fn index(user: User) -> Template {
    let context = AdminContext {
        header: "Club Coding".to_string(),
        username: user.username,
        views_today: 187232,
        videos_total: get_videos().len(),
        series_total: get_series().len(),
        revenue_month: 102230,
        paying_users: 123,
        total_users: get_users().len(),
    };
    Template::render("admin/index", &context)
}

#[get("/views")]
pub fn views(user: User) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        username: user.username,
    };
    Template::render("admin/views", &context)
}
