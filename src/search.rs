use rocket::Route;
use rocket_contrib::Template;
use users::User;
use club_coding::models::{UsersViews, Videos};
use rocket::response::{Flash, Redirect};
use database::DbConn;
use diesel::prelude::*;

#[derive(Serialize)]
pub struct PublicVideo {
    pub episode_number: i32,
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub watched: bool,
}

pub fn get_video_watched(connection: &DbConn, uid: i64, vid: i64) -> bool {
    use club_coding::schema::users_views::dsl::*;

    match users_views
        .filter(user_id.eq(uid))
        .filter(video_id.eq(vid))
        .first::<UsersViews>(&**connection)
    {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_videos(connection: &DbConn, query: String, uid: Option<i64>) -> Vec<PublicVideo> {
    use club_coding::schema::videos::dsl::*;

    match videos
        .filter(description.like(query))
        .load::<Videos>(&**connection)
    {
        Ok(vec_of_videos) => {
            let mut to_return: Vec<PublicVideo> = vec![];
            let mut number = 1;
            for video in vec_of_videos {
                let w_atched = match uid {
                    Some(uid) => get_video_watched(connection, uid, video.id),
                    None => false,
                };
                to_return.push(PublicVideo {
                    episode_number: number,
                    uuid: video.uuid,
                    title: video.title,
                    description: video.description,
                    watched: w_atched,
                });
                number = number + 1;
            }
            to_return
        }
        Err(_) => vec![],
    }
}

#[derive(FromForm)]
struct Query {
    search_query: String,
}

#[derive(Serialize)]
struct SearchContext<'a> {
    header: String,
    videos: &'a Vec<PublicVideo>,
    user: User,
    search_query: &'a String,
}

#[get("/search?<query>")]
fn search(conn: DbConn, user: User, query: Query) -> Result<Template, Flash<Redirect>> {
    if query.search_query.len() > 3 {
        let videos: Vec<PublicVideo> =
            get_videos(&conn, format!("%{}%", query.search_query), Some(user.id));
        if videos.len() > 0 {
            let context = SearchContext {
                header: "Club Coding".to_string(),
                videos: &videos,
                user: user,
                search_query: &query.search_query,
            };
            Ok(Template::render("search/search", &context))
        } else {
            Err(Flash::error(Redirect::to("/"), "No results found"))
        }
    } else {
        Err(Flash::error(
            Redirect::to("/"),
            "Please enter more than 3 characters",
        ))
    }
}

#[derive(Serialize)]
struct SearchContextNoLogin<'a> {
    header: String,
    videos: &'a Vec<PublicVideo>,
    search_query: &'a String,
}

#[get("/search?<query>", rank = 2)]
fn search_nologin(conn: DbConn, query: Query) -> Result<Template, Flash<Redirect>> {
    if query.search_query.len() > 3 {
        let videos: Vec<PublicVideo> = get_videos(&conn, format!("%{}%", query.search_query), None);
        if videos.len() > 0 {
            let context = SearchContextNoLogin {
                header: "Club Coding".to_string(),
                videos: &videos,
                search_query: &query.search_query,
            };
            Ok(Template::render("search/search_nologin", &context))
        } else {
            Err(Flash::error(Redirect::to("/"), "No results found"))
        }
    } else {
        Err(Flash::error(
            Redirect::to("/"),
            "Please enter more than 3 characters",
        ))
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![search, search_nologin]
}
