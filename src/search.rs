use rocket::Route;
use rocket_contrib::Template;
use structs::LoggedInContext;
use users::User;

#[get("/search/<_query>")]
fn search(user: User, _query: String) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        username: user.username,
    };
    Template::render("home", &context)
}

pub fn endpoints() -> Vec<Route> {
    routes![search]
}
