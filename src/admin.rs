use rocket::Route;
use rocket_contrib::Template;
use users::{get_users, User};
use series::get_series;
use videos::get_videos;
use rocket::response::Redirect;
use club_coding::models::{Series, Users, Videos};
use club_coding::{create_new_series, create_new_video, establish_connection};
use structs::LoggedInContext;
use rand;
use std;
use chrono::NaiveDateTime;
use rocket_contrib::Json;
use diesel::prelude::*;
use rocket::request::Form;

fn generate_token(length: u8) -> Result<String, std::io::Error> {
    let bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
    let strings: Vec<String> = bytes.iter().map(|byte| format!("{:02X}", byte)).collect();
    return Ok(strings.join(""));
}

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
fn index(user: User) -> Template {
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
fn views(user: User) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        username: user.username,
    };
    Template::render("admin/views", &context)
}

#[derive(Serialize)]
struct Serie {
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
struct SeriesContext {
    header: String,
    username: String,
    series: Vec<Serie>,
}

fn get_all_series() -> Vec<Serie> {
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
fn series(user: User) -> Template {
    let context = SeriesContext {
        header: "Club Coding".to_string(),
        username: user.username,
        series: get_all_series(),
    };
    Template::render("admin/series", &context)
}
#[get("/series/new")]
fn new_series(user: User) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        username: user.username,
    };
    Template::render("admin/new_serie", &context)
}

#[derive(FromForm)]
struct NewSerie {
    title: String,
    description: String,
}

fn create_slug(title: &String) -> String {
    title
        .chars()
        .map(|character| match character {
            'A'...'Z' => ((character as u8) - b'A' + b'a') as char,
            'a'...'z' | '0'...'9' => character,
            _ => '-',
        })
        .collect()
}

#[post("/series/new", data = "<serie>")]
fn insert_new_series(_user: User, serie: Form<NewSerie>) -> Result<Redirect, Redirect> {
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
struct EditSeries {
    header: String,
    username: String,
    uuid: String,
    title: String,
    description: String,
    published: bool,
    archived: bool,
}

#[get("/series/edit/<uuid>")]
fn edit_series(uuid: String, user: User) -> Option<Template> {
    match get_serie(uuid.clone()) {
        Some(serie) => {
            let context = EditSeries {
                header: "Club Coding".to_string(),
                username: user.username,
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
struct UpdateSerie {
    title: String,
    description: String,
    published: bool,
    archived: bool,
}

#[post("/series/edit/<uid>", format = "application/json", data = "<data>")]
fn update_serie(uid: String, _user: User, data: Json<UpdateSerie>) -> Json<UpdateSerie> {
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

#[derive(Serialize)]
struct UsersC {
    id: i64,
    username: String,
    paying: bool,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

fn get_all_users() -> Vec<UsersC> {
    use club_coding::schema::users::dsl::*;

    let connection = establish_connection();
    let result = users
        .load::<Users>(&connection)
        .expect("Error loading users");

    let mut ret: Vec<UsersC> = vec![];

    for user in result {
        ret.push(UsersC {
            id: user.id,
            username: user.username,
            paying: true,
            created: user.created,
            updated: user.updated,
        })
    }
    ret
}

#[derive(Serialize)]
struct UsersContext {
    header: String,
    username: String,
    users: Vec<UsersC>,
}

#[get("/users")]
fn users(user: User) -> Template {
    let context = UsersContext {
        header: "Club Coding".to_string(),
        username: user.username,
        users: get_all_users(),
    };
    Template::render("admin/users", &context)
}

#[derive(Deserialize, Serialize)]
struct EditUser {
    email: String,
    force_change_password: bool,
    force_resend_email: bool,
    free_membership: bool,
    deactivated: bool,
}

#[derive(Serialize)]
struct EditUsersContext {
    header: String,
    username: String,
    uuid: String,
    user: EditUser,
}

#[get("/users/edit/<uuid>")]
fn edit_users(uuid: String, user: User) -> Template {
    let context = EditUsersContext {
        header: "Club Coding".to_string(),
        username: user.username.clone(),
        uuid: uuid,
        user: EditUser {
            email: user.username,
            force_change_password: true,
            force_resend_email: true,
            free_membership: true,
            deactivated: true,
        },
    };
    Template::render("admin/edit_user", &context)
}

#[post("/users/edit/<uid>", format = "application/json", data = "<data>")]
fn update_user(uid: i64, _user: User, data: Json<EditUser>) -> Json<EditUser> {
    use club_coding::schema::users::dsl::*;

    let connection = establish_connection();

    diesel::update(users.find(uid))
        .set(username.eq(data.0.email.clone()))
        .execute(&connection)
        .unwrap();
    data
}

#[derive(Serialize)]
struct Video {
    uuid: String,
    name: String,
    views: u64,
    comments: u64,
    serie: Option<String>,
    membership: bool,
    published: bool,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

#[derive(Serialize)]
struct VideosContext {
    header: String,
    username: String,
    videos: Vec<Video>,
}

fn get_all_videos() -> Vec<Video> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();
    let result = videos
        .load::<Videos>(&connection)
        .expect("Error loading videos");

    let mut ret: Vec<Video> = vec![];

    for video in result {
        let series_name: Option<String> = match video.series {
            Some(serie_id) => {
                use club_coding::schema::series::dsl::*;

                let serie: Series = series
                    .find(serie_id)
                    .first(&connection)
                    .expect("Unable to find series");
                Some(serie.title)
            }
            None => None,
        };

        ret.push(Video {
            uuid: video.uuid,
            name: video.title,
            views: 0,
            comments: 0,
            serie: series_name,
            membership: video.membership_only,
            published: video.published,
            created: video.created,
            updated: video.updated,
        })
    }
    ret
}

#[get("/videos")]
fn videos(user: User) -> Template {
    let context = VideosContext {
        header: "Club Coding".to_string(),
        username: user.username,
        videos: get_all_videos(),
    };
    Template::render("admin/videos", &context)
}

#[get("/videos/new")]
fn new_video(user: User) -> Template {
    let context = SeriesContext {
        header: "Club Coding".to_string(),
        username: user.username,
        series: get_all_series(),
    };
    Template::render("admin/new_video", &context)
}

#[derive(FromForm)]
struct NewVideo {
    title: String,
    description: String,
    vimeo_id: String,
    serie: Option<String>,
    membership_only: bool,
}

#[post("/videos/new", data = "<video>")]
fn insert_new_video(_user: User, video: Form<NewVideo>) -> Result<Redirect, Redirect> {
    let new_video: NewVideo = video.into_inner();
    let slug = create_slug(&new_video.title);
    let connection = establish_connection();
    let series: Option<i64> = Some(0);
    let episode_number: Option<i32> = Some(0);
    match generate_token(24) {
        Ok(uuid) => {
            create_new_video(
                &connection,
                uuid.clone(),
                new_video.title,
                slug,
                new_video.description,
                false,
                new_video.membership_only,
                series,
                episode_number,
                false,
                new_video.vimeo_id,
            );
            Ok(Redirect::to(&format!("/admin/videos/edit/{}", uuid)))
        }
        Err(_) => Err(Redirect::to("/admin/videos/new")),
    }
}

#[derive(Serialize)]
struct EditVideo {
    header: String,
    username: String,
    uuid: String,
    title: String,
    description: String,
    published: bool,
    membership: bool,
}

fn get_video(uid: String) -> Option<Videos> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();
    let result = videos
        .filter(uuid.eq(uid))
        .limit(1)
        .load::<Videos>(&connection)
        .expect("Error loading users");

    if result.len() == 1 {
        return Some(result[0].clone());
    } else {
        return None;
    }
}

#[get("/videos/edit/<uuid>")]
fn edit_video(uuid: String, user: User) -> Option<Template> {
    match get_video(uuid.clone()) {
        Some(video) => {
            let context = EditVideo {
                header: "Club Coding".to_string(),
                username: user.username,
                uuid: uuid,
                title: video.title,
                description: video.description,
                published: video.published,
                membership: video.membership_only,
            };
            Some(Template::render("admin/edit_video", &context))
        }
        None => None,
    }
}

#[derive(Deserialize, Serialize)]
struct UpdateVideo {
    title: String,
    description: String,
    membership: bool,
    published: bool,
}

#[post("/videos/edit/<uid>", format = "application/json", data = "<data>")]
fn update_video(uid: String, _user: User, data: Json<UpdateVideo>) -> Json<UpdateVideo> {
    use club_coding::schema::videos::dsl::*;

    let connection = establish_connection();

    diesel::update(videos.filter(uuid.eq(uid)))
        .set((
            title.eq(data.0.title.clone()),
            description.eq(data.description.clone()),
            membership_only.eq(data.0.membership),
            published.eq(data.0.published),
        ))
        .execute(&connection)
        .unwrap();
    data
}

pub fn endpoints() -> Vec<Route> {
    routes![
        index,
        views,
        series,
        new_series,
        insert_new_series,
        edit_series,
        update_serie,
        users,
        edit_users,
        update_user,
        videos,
        new_video,
        insert_new_video,
        edit_video,
        update_video
    ]
}
