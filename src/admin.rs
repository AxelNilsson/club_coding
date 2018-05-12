use rocket::Route;
use rocket_contrib::Template;
use users::{get_users, User};
use series::get_series;
use videos::get_videos;
use multipart::server::Multipart;
use multipart::server::save::SaveResult::*;
use rocket::response::status::Custom;
use rocket::Data;
use rocket::response::Redirect;
use rocket::http::{ContentType, Status};
use club_coding::models::{Series, Users, Videos};
use club_coding::{create_new_video, establish_connection};
use structs::{Context, LoggedInContext};
use std::{io, fs::File, io::Write};
use rand;
use std;
use chrono::NaiveDateTime;
use rocket_contrib::Json;
use diesel::prelude::*;

fn generate_token(length: u8) -> Result<String, std::io::Error> {
    let bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
    let strings: Vec<String> = bytes.iter().map(|byte| format!("{:02X}", byte)).collect();
    return Ok(strings.join(""));
}

#[get("/upload")]
fn upload_page(_user: User) -> Template {
    let context = Context {
        header: "Sign up!".to_string(),
    };

    Template::render("admin/upload_video", &context)
}

#[post("/upload", data = "<data>")]
// signature requires the request to have a `Content-Type`
fn upload(cont_type: &ContentType, data: Data, _user: User) -> Result<Redirect, Custom<String>> {
    // this and the next check can be implemented as a request guard but it seems like just
    // more boilerplate than necessary
    if !cont_type.is_form_data() {
        return Err(Custom(
            Status::BadRequest,
            "Content-Type not multipart/form-data".into(),
        ));
    }

    let (_, boundary) = cont_type
        .params()
        .find(|&(k, _)| k == "boundary")
        .ok_or_else(|| {
            Custom(
                Status::BadRequest,
                "`Content-Type: multipart/form-data` boundary param not provided".into(),
            )
        })?;

    match generate_token(24) {
        Ok(token) => match process_upload(token.clone(), boundary, data) {
            Ok(_) => {
                let connection = establish_connection();
                create_new_video(
                    &connection,
                    token.clone(),
                    token.clone(),
                    token.clone(),
                    token.clone(),
                    false,
                    false,
                    Some(1),
                    Some(1),
                );
                Ok(Redirect::to(&format!("/video/{}", token)))
            }
            Err(err) => Err(Custom(Status::InternalServerError, err.to_string())),
        },
        Err(err) => Err(Custom(Status::InternalServerError, err.to_string())),
    }
}

fn process_upload(filename: String, boundary: &str, data: Data) -> io::Result<()> {
    Multipart::with_body(data.open(), boundary).foreach_entry(|mut field| {
        let mut bytes = Vec::new();
        match field.data.save().size_limit(None).write_to(&mut bytes) {
            Full(_) => {
                let mut file = File::create(&format!("{}.mp4", filename) as &str).unwrap();
                match file.write(&bytes) {
                    Ok(_) => {}
                    Err(_) => {}
                };
            }
            _ => {}
        };
    })?;

    Ok(())
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
    name: String,
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
            name: serie.name,
            views: 0,
            comments: 0,
            published: serie.published,
            archived: serie.is_archived,
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
                title: serie.name,
                description: serie.description,
                published: serie.published,
                archived: serie.is_archived,
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
            name.eq(data.0.title.clone()),
            description.eq(data.description.clone()),
            published.eq(data.0.published),
            is_archived.eq(data.0.archived),
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
                Some(serie.name)
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
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        username: user.username,
    };
    Template::render("admin/new_video", &context)
}

#[get("/videos/upload/<_uuid>")]
fn upload_video(_uuid: String, user: User) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        username: user.username,
    };
    Template::render("admin/upload_video", &context)
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
        upload,
        upload_page,
        series,
        new_series,
        edit_series,
        update_serie,
        users,
        edit_users,
        update_user,
        videos,
        new_video,
        upload_video,
        edit_video,
        update_video
    ]
}
