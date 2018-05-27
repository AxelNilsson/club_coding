use rocket::Catcher;
use rocket_contrib::Template;

#[derive(Serialize)]
struct Context<'a> {
    header: &'a str,
}

#[error(404)]
fn not_found() -> Template {
    let context = Context {
        header: "Club Coding",
    };
    Template::render("errors/404", &context)
}

#[error(500)]
fn internal_error() -> Template {
    let context = Context {
        header: "Club Coding",
    };
    Template::render("errors/500", &context)
}

pub fn endpoints() -> Vec<Catcher> {
    errors![not_found, internal_error]
}
