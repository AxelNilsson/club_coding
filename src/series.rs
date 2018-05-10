use rocket::Route;
use club_coding::establish_connection;
use diesel::prelude::*;
use club_coding::models::Series;
use structs::Context;
use rocket_contrib::Template;

pub fn get_series() -> Vec<Series> {
    use club_coding::schema::series::dsl::*;

    let connection = establish_connection();
    series
        .filter(published.eq(true))
        .filter(is_archived.eq(false))
        .order(updated.asc())
        .load::<Series>(&connection)
        .expect("Error loading users")
}

#[derive(Serialize)]
pub struct PublicSeries {
    uuid: String,
    name: String,
    slug: String,
    description: String,
}

pub fn get_last_10_series() -> Vec<PublicSeries> {
    use club_coding::schema::series::dsl::*;

    let connection = establish_connection();
    let s_eries = series
        .filter(published.eq(true))
        .filter(is_archived.eq(false))
        .limit(10)
        .order(updated.asc())
        .load::<Series>(&connection)
        .expect("Error loading users");

    let mut to_return: Vec<PublicSeries> = vec![];
    for serie in s_eries {
        to_return.push(PublicSeries {
            uuid: serie.uuid,
            name: serie.name,
            slug: serie.slug,
            description: serie.description,
        });
    }
    to_return
}

#[get("/<_uuid>")]
fn serie(_uuid: String) -> Template {
    let context = Context {
        header: "Sign up!".to_string(),
    };
    Template::render("series", &context)
}

pub fn endpoints() -> Vec<Route> {
    routes![serie]
}
