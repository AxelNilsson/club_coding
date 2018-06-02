use rocket_contrib::Template;
use admin::structs::Administrator;
use club_coding::models::{Users, UsersGroup, UsersSeriesAccess};
use club_coding::{create_new_user_group, create_new_user_series_access};
use database::DbConn;
use chrono::NaiveDateTime;
use rocket_contrib::Json;
use diesel::prelude::*;
use admin::group::get_all_groupsc;
use admin::series::get_all_seriesc;
use admin::series::SerieC;
use admin::group::GroupC;
use authentication::verify::send_verify_email;
use rocket::Route;
use structs::PostmarkToken;
use rocket::State;
use std::io::{Error, ErrorKind};

#[derive(Serialize)]
struct UsersC {
    id: i64,
    username: String,
    email: String,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

fn get_all_users(connection: &DbConn) -> Vec<UsersC> {
    use club_coding::schema::users::dsl::*;

    match users.load::<Users>(&**connection) {
        Ok(result) => {
            let mut ret: Vec<UsersC> = vec![];

            for user in result {
                ret.push(UsersC {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    created: user.created,
                    updated: user.updated,
                })
            }
            ret
        }
        Err(_) => vec![],
    }
}

#[derive(Serialize)]
struct UsersContext<'a> {
    header: &'a str,
    user: Administrator,
    users: Vec<UsersC>,
}

#[get("/users")]
pub fn users(conn: DbConn, user: Administrator) -> Template {
    let context = UsersContext {
        header: "Club Coding",
        user: user,
        users: get_all_users(&conn),
    };
    Template::render("admin/users", &context)
}

pub fn get_all_groups_for_user(connection: &DbConn, uid: i64) -> Vec<i64> {
    use club_coding::schema::users_group::dsl::*;

    let mut returning_groups = vec![];

    match users_group
        .filter(user_id.eq(uid))
        .load::<UsersGroup>(&**connection)
    {
        Ok(matching_groups) => for group in matching_groups {
            returning_groups.push(group.group_id);
        },
        Err(_) => {}
    }
    returning_groups
}

pub fn get_all_series_for_user(connection: &DbConn, uid: i64) -> Vec<i64> {
    use club_coding::schema::users_series_access::dsl::*;

    match users_series_access
        .filter(user_id.eq(uid))
        .load::<UsersSeriesAccess>(&**connection)
    {
        Ok(matching_series) => {
            let mut returning_series = vec![];

            for serie in matching_series {
                returning_series.push(serie.series_id);
            }
            returning_series
        }
        Err(_) => vec![],
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditUser {
    username: String,
    email: String,
    series: Vec<i64>,
    groups: Vec<i64>,
    force_resend_email: bool,
}

#[derive(Serialize)]
struct EditUsersContext<'a> {
    header: &'a str,
    user: &'a Administrator,
    uuid: i64,
    user_data: EditUser,
    groups: Vec<GroupC>,
    series: Vec<SerieC>,
}

fn get_user(connection: &DbConn, uid: i64) -> Option<UsersC> {
    use club_coding::schema::users::dsl::*;

    match users.filter(id.eq(uid)).first::<Users>(&**connection) {
        Ok(result) => Some(UsersC {
            id: result.id,
            username: result.username,
            email: result.email,
            created: result.created,
            updated: result.updated,
        }),
        Err(_) => None,
    }
}

#[get("/users/edit/<uuid>")]
pub fn edit_users(conn: DbConn, uuid: i64, admin: Administrator) -> Option<Template> {
    match get_user(&conn, uuid) {
        Some(user) => {
            let context = EditUsersContext {
                header: "Club Coding",
                user: &admin,
                uuid: uuid,
                groups: get_all_groupsc(&conn),
                series: get_all_seriesc(&conn),
                user_data: EditUser {
                    username: user.username,
                    email: user.email,
                    groups: get_all_groups_for_user(&conn, user.id),
                    series: get_all_series_for_user(&conn, user.id),
                    force_resend_email: false,
                },
            };
            Some(Template::render("admin/edit_user", &context))
        }
        None => None,
    }
}

fn resend_confirmation_email(
    connection: &DbConn,
    postmark_token: &str,
    uid: i64,
) -> Result<(), Error> {
    match get_user(connection, uid) {
        Some(user) => {
            use club_coding::schema::users::dsl::*;

            match diesel::update(users.find(uid))
                .set(verified.eq(false))
                .execute(&**connection)
            {
                Ok(_) => {
                    send_verify_email(connection, postmark_token, user.id, user.email)?;
                    Ok(())
                }
                Err(_) => Err(Error::new(
                    ErrorKind::Other,
                    "Could not set verified to false",
                )),
            }
        }
        None => Err(Error::new(ErrorKind::Other, "No user found")),
    }
}

#[post("/users/edit/<uid>", format = "application/json", data = "<data>")]
pub fn update_user(
    conn: DbConn,
    postmark_token: State<PostmarkToken>,
    uid: i64,
    _user: Administrator,
    data: Json<EditUser>,
) -> Result<(), ()> {
    use club_coding::schema::users::dsl::*;

    match diesel::update(users.find(uid))
        .set((username.eq(&data.0.username), email.eq(&data.0.email)))
        .execute(&*conn)
    {
        Ok(_) => {
            let groups = get_all_groups_for_user(&conn, uid);
            for group in &groups {
                let mut should_delete = true;
                for data_group in &data.0.groups {
                    if *group == *data_group {
                        should_delete = false;
                    }
                }
                if should_delete {
                    use club_coding::schema::users_group::dsl::*;

                    match diesel::delete(
                        users_group
                            .filter(user_id.eq(uid))
                            .filter(group_id.eq(*group)),
                    ).execute(&*conn)
                    {
                        Ok(_) => {}
                        Err(_) => return Err(()),
                    }
                }
            }

            for data_group in &data.0.groups {
                let mut should_create = true;
                for group in &groups {
                    if *group == *data_group {
                        should_create = false;
                    }
                }
                if should_create {
                    match create_new_user_group(&conn, uid, *data_group) {
                        Ok(_) => {}
                        Err(_) => return Err(()),
                    }
                }
            }

            let series = get_all_series_for_user(&conn, uid);
            for serie in &series {
                let mut should_delete = true;
                for data_serie in &data.0.series {
                    if *serie == *data_serie {
                        should_delete = false;
                    }
                }
                if should_delete {
                    use club_coding::schema::users_series_access::dsl::*;

                    match diesel::delete(
                        users_series_access
                            .filter(user_id.eq(uid))
                            .filter(bought.eq(false))
                            .filter(series_id.eq(*serie)),
                    ).execute(&*conn)
                    {
                        Ok(_) => {}
                        Err(_) => return Err(()),
                    }
                }
            }

            for data_serie in &data.0.series {
                let mut should_create = true;
                for serie in &series {
                    if *serie == *data_serie {
                        should_create = false;
                    }
                }
                if should_create {
                    match create_new_user_series_access(&conn, uid, *data_serie, false) {
                        Ok(_) => {}
                        Err(_) => return Err(()),
                    }
                }
            }

            match data.0.force_resend_email {
                true => match resend_confirmation_email(&conn, &postmark_token.0, uid) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(()),
                },
                false => Ok(()),
            }
        }
        Err(_) => return Err(()),
    }
}

/// Assembles all of the endpoints.
/// The upside of assembling all of the endpoints here
/// is that we don't have to update the main function but
/// instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    routes![users, edit_users, update_user]
}
