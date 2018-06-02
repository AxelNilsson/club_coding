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

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Catcher> {
    errors![not_found, internal_error]
}
