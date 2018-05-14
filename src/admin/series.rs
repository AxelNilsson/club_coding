use rocket_contrib::Template;
use users::User;
use rocket::response::Redirect;
use club_coding::models::Series;
use club_coding::{create_new_series, establish_connection};
use structs::LoggedInContext;
use chrono::NaiveDateTime;
use rocket_contrib::Json;
use diesel::prelude::*;
use rocket::request::Form;
use admin::generate_token;
use admin::create_slug;
use rocket::Route;

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
pub struct SeriesContext {
    pub header: String,
    pub user: User,
    pub series: Vec<Serie>,
}

pub fn get_all_series() -> Vec<Serie> {
    use club_coding::schema::series::dsl::*;

    let connection = establish_connection();
    let result = series
        .load::<Series>(&connection)
        .expect("Error loading series");

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

#[get("/series")]
pub fn series(user: User) -> Template {
    let context = SeriesContext {
        header: "Club Coding".to_string(),
        user: user,
        series: get_all_series(),
    };
    Template::render("admin/series", &context)
}

#[get("/series/new")]
pub fn new_series(user: User) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        user: user,
    };
    Template::render("admin/new_serie", &context)
}

#[derive(FromForm)]
pub struct NewSerie {
    title: String,
    description: String,
}

#[post("/series/new", data = "<serie>")]
pub fn insert_new_series(_user: User, serie: Form<NewSerie>) -> Result<Redirect, Redirect> {
    let new_serie: NewSerie = serie.into_inner();
    let slug = create_slug(&new_serie.title);
    let connection = establish_connection();
    match generate_token(24) {
        Ok(uuid) => {
            create_new_series(
                &connection,
                uuid.clone(),
                new_serie.title,
                slug,
                new_serie.description,
                false,
                false,
            );
            Ok(Redirect::to(&format!("/admin/series/edit/{}", uuid)))
        }
        Err(_) => Err(Redirect::to("/admin/series/new")),
    }
}

fn get_serie(uid: String) -> Option<Series> {
    use club_coding::schema::series::dsl::*;

    let connection = establish_connection();
    let result = series
        .filter(uuid.eq(uid))
        .limit(1)
        .load::<Series>(&connection)
        .expect("Error loading series");

    if result.len() == 1 {
        return Some(result[0].clone());
    } else {
        return None;
    }
}

#[derive(Serialize)]
pub struct EditSeries {
    header: String,
    user: User,
    uuid: String,
    title: String,
    description: String,
    published: bool,
    archived: bool,
}

#[get("/series/edit/<uuid>")]
pub fn edit_series(uuid: String, user: User) -> Option<Template> {
    match get_serie(uuid.clone()) {
        Some(serie) => {
            let context = EditSeries {
                header: "Club Coding".to_string(),
                user: user,
                uuid: uuid,
                title: serie.title,
                description: serie.description,
                published: serie.published,
                archived: serie.archived,
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
    published: bool,
    archived: bool,
}

#[post("/series/edit/<uid>", format = "application/json", data = "<data>")]
pub fn update_serie(uid: String, _user: User, data: Json<UpdateSerie>) -> Json<UpdateSerie> {
    use club_coding::schema::series::dsl::*;

    let connection = establish_connection();

    diesel::update(series.filter(uuid.eq(uid)))
        .set((
            title.eq(data.0.title.clone()),
            description.eq(data.description.clone()),
            published.eq(data.0.published),
            archived.eq(data.0.archived),
        ))
        .execute(&connection)
        .unwrap();
    data
}

pub fn endpoints() -> Vec<Route> {
    routes![
        series,
        new_series,
        insert_new_series,
        edit_series,
        update_serie,
    ]
}
