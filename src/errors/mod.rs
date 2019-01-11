use rocket::Catcher;
use rocket_contrib::templates::Template;

#[cfg(test)]
mod tests;

/// Context for rendering tera templates
/// for not logged in endpoints. Mainly
/// the error endpoints.
#[derive(Serialize)]
pub struct Context<'a> {
    /// Header used in tera templates.
    /// Mainly used for the title.
    pub header: &'a str,
}

/// 404 - Not found endpoint.
/// This endpoint will kick in
/// if the requested page doesn't exist.
/// Responds with the 404
/// Template in the Errors folder.
#[catch(404)]
fn not_found() -> Template {
    let context = Context {
        header: "404 - Not Found",
    };
    Template::render("errors/404", &context)
}

/// 500 - Internal server error endpoint.
/// This endpoint will kick in
/// if the requested page doesn't compute.
/// Responds with the 500
/// Template in the Errors folder.
#[catch(500)]
fn internal_error() -> Template {
    let context = Context {
        header: "500 - Internal Server Error",
    };
    Template::render("errors/500", &context)
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Catcher> {
    catchers![not_found, internal_error]
}
