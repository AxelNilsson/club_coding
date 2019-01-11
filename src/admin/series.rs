use rocket_contrib::templates::Template;
use admin::structs::{Administrator, LoggedInContext};
use rocket::response::Redirect;
use club_coding::models::Series;
use club_coding::create_new_series;
use database::{DbConn, RedisConnection};
use chrono::NaiveDateTime;
use rocket_contrib::json::Json;
use diesel::prelude::*;
use rocket::request::Form;
use admin::generate_token;
use admin::create_slug;
use rocket::Route;
use redis::Commands;

#[derive(Serialize)]
pub struct Serie {
    uuid: String,
    title: String,
    views: u64,
    comments: u64,
    published: bool,
    archived: bool,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

#[derive(Serialize)]
pub struct SeriesContext<'a> {
    pub header: &'a str,
    pub user: Administrator,
    pub series: Vec<Serie>,
}

#[derive(Deserialize, Serialize)]
pub struct SerieC {
    id: i64,
    name: String,
}

pub fn get_all_seriesc(connection: &DbConn) -> Vec<SerieC> {
    use club_coding::schema::series::dsl::*;

    match series.load::<Series>(&**connection) {
        Ok(result) => {
            let mut ret: Vec<SerieC> = vec![];

            for serie in result {
                ret.push(SerieC {
                    id: serie.id,
                    name: serie.title,
                })
            }
            ret
        }
        Err(_) => vec![],
    }
}

pub fn get_all_series(connection: &DbConn) -> Vec<Serie> {
    use club_coding::schema::series::dsl::*;

    match series.load::<Series>(&**connection) {
        Ok(result) => {
            let mut ret: Vec<Serie> = vec![];

            for serie in result {
                ret.push(Serie {
                    uuid: serie.uuid,
                    title: serie.title,
                    views: 0,
                    comments: 0,
                    published: serie.published,
                    archived: serie.archived,
                    created: serie.created,
                    updated: serie.updated,
                })
            }
            ret
        }
        Err(_) => vec![],
    }
}

#[get("/series")]
pub fn series(conn: DbConn, user: Administrator) -> Template {
    let context = SeriesContext {
        header: "Club Coding",
        user: user,
        series: get_all_series(&conn),
    };
    Template::render("admin/series", &context)
}

#[get("/series/new")]
pub fn new_series(user: Administrator) -> Template {
    let context = LoggedInContext {
        header: "Club Coding",
        user: user,
    };
    Template::render("admin/new_serie", &context)
}

#[derive(FromForm)]
pub struct NewSerie {
    title: String,
    description: String,
    price: i32,
}

#[post("/series/new", data = "<serie>")]
pub fn insert_new_series(
    mysql_conn: DbConn,
    redis_conn: RedisConnection,
    _user: Administrator,
    serie: Form<NewSerie>,
) -> Result<Redirect, Redirect> {
    let new_serie: NewSerie = serie.into_inner();
    match redis_conn.del::<&str, String>("last10") {
        Ok(_) => {}
        Err(_) => {}
    }
    let slug = create_slug(&new_serie.title);
    match generate_token(24) {
        Ok(uuid) => match create_new_series(
            &*mysql_conn,
            &uuid,
            &new_serie.title,
            &slug,
            &new_serie.description,
            new_serie.price,
            false,
            false,
        ) {
            Ok(_) => Ok(Redirect::to(format!("/admin/series/edit/{}", uuid))),
            Err(_) => Ok(Redirect::to(format!("/admin/series/edit/{}", uuid))),
        },
        Err(_) => Err(Redirect::to("/admin/series/new")),
    }
}

fn get_serie(connection: &DbConn, uid: &str) -> Option<Series> {
    use club_coding::schema::series::dsl::*;

    match series.filter(uuid.eq(uid)).first::<Series>(&**connection) {
        Ok(result) => Some(result),
        Err(_) => return None,
    }
}

#[derive(Serialize)]
pub struct EditSeries<'a> {
    header: &'a str,
    user: Administrator,
    uuid: &'a str,
    title: String,
    description: String,
    price: i32,
    published: bool,
    archived: bool,
    in_development: bool,
}

#[get("/series/edit/<uuid>")]
pub fn edit_series(conn: DbConn, uuid: String, user: Administrator) -> Option<Template> {
    match get_serie(&conn, &uuid) {
        Some(serie) => {
            let context = EditSeries {
                header: "Club Coding",
                user: user,
                uuid: &uuid,
                title: serie.title,
                description: serie.description,
                price: serie.price,
                published: serie.published,
                archived: serie.archived,
                in_development: serie.in_development,
            };
            Some(Template::render("admin/edit_serie", &context))
        }
        None => None,
    }
}

#[derive(Deserialize, Serialize)]
pub struct UpdateSerie {
    title: String,
    description: String,
    price: i32,
    published: bool,
    archived: bool,
    in_development: bool,
}

#[post("/series/edit/<uid>", format = "application/json", data = "<data>")]
pub fn update_serie(
    mysql_conn: DbConn,
    redis_conn: RedisConnection,
    uid: String,
    _user: Administrator,
    data: Json<UpdateSerie>,
) -> Result<(), ()> {
    match redis_conn.del::<&str, String>("last10") {
        Ok(_) => {}
        Err(_) => {}
    }
    use club_coding::schema::series::dsl::*;

    match diesel::update(series.filter(uuid.eq(uid)))
        .set((
            title.eq(&data.0.title),
            description.eq(&data.description),
            price.eq(data.price),
            published.eq(data.0.published),
            archived.eq(data.0.archived),
            in_development.eq(data.0.in_development),
        ))
        .execute(&*mysql_conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![
        series,
        new_series,
        insert_new_series,
        edit_series,
        update_serie,
    ]
}
