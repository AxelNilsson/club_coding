use rocket_contrib::Template;
use admin::structs::Administrator;
use club_coding::models::{Users, UsersGroup, UsersSeriesAccess};
use club_coding::{create_new_user_group, create_new_user_series_access, establish_connection};
use chrono::NaiveDateTime;
use rocket_contrib::Json;
use diesel::prelude::*;
use admin::group::get_all_groupsc;
use admin::series::get_all_seriesc;
use admin::series::SerieC;
use admin::group::GroupC;
use authentication::send_verify_email;
use rocket::Route;

#[derive(Serialize)]
struct UsersC {
    id: i64,
    username: String,
    email: String,
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
            email: user.email,
            created: user.created,
            updated: user.updated,
        })
    }
    ret
}

#[derive(Serialize)]
struct UsersContext {
    header: String,
    user: Administrator,
    users: Vec<UsersC>,
}

#[get("/users")]
pub fn users(user: Administrator) -> Template {
    let context = UsersContext {
        header: "Club Coding".to_string(),
        user: user,
        users: get_all_users(),
    };
    Template::render("admin/users", &context)
}

pub fn get_all_groups_for_user(uid: i64) -> Vec<i64> {
    use club_coding::schema::users_group::dsl::*;

    let connection = establish_connection();

    let matching_groups = users_group
        .filter(user_id.eq(uid))
        .load::<UsersGroup>(&connection)
        .expect("Unable to find users group");

    let mut returning_groups = vec![];

    for group in matching_groups {
        returning_groups.push(group.group_id);
    }
    returning_groups
}

pub fn get_all_series_for_user(uid: i64) -> Vec<i64> {
    use club_coding::schema::users_series_access::dsl::*;

    let connection = establish_connection();

    let matching_series = users_series_access
        .filter(user_id.eq(uid))
        .load::<UsersSeriesAccess>(&connection)
        .expect("Unable to find users group");

    let mut returning_series = vec![];

    for serie in matching_series {
        returning_series.push(serie.series_id);
    }
    returning_series
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
    header: String,
    user: &'a Administrator,
    uuid: i64,
    user_data: EditUser,
    groups: Vec<GroupC>,
    series: Vec<SerieC>,
}

fn get_user(uid: i64) -> Option<UsersC> {
    use club_coding::schema::users::dsl::*;

    let connection = establish_connection();
    let result = users
        .filter(id.eq(uid))
        .load::<Users>(&connection)
        .expect("Error loading users");

    if result.len() == 1 {
        Some(UsersC {
            id: result[0].id,
            username: result[0].username.clone(),
            email: result[0].email.clone(),
            created: result[0].created,
            updated: result[0].updated,
        })
    } else {
        None
    }
}

#[get("/users/edit/<uuid>")]
pub fn edit_users(uuid: i64, admin: Administrator) -> Option<Template> {
    match get_user(uuid) {
        Some(user) => {
            let context = EditUsersContext {
                header: "Club Coding".to_string(),
                user: &admin,
                uuid: uuid,
                groups: get_all_groupsc(),
                series: get_all_seriesc(),
                user_data: EditUser {
                    username: user.username.clone(),
                    email: user.email.clone(),
                    groups: get_all_groups_for_user(user.id),
                    series: get_all_series_for_user(user.id),
                    force_resend_email: false,
                },
            };
            Some(Template::render("admin/edit_user", &context))
        }
        None => None,
    }
}

fn resend_confirmation_email(uid: i64) {
    match get_user(uid) {
        Some(user) => {
            use club_coding::schema::users::dsl::*;
            let connection = establish_connection();

            diesel::update(users.find(uid))
                .set(verified.eq(false))
                .execute(&connection)
                .unwrap();

            send_verify_email(&connection, user.id, user.email);
        }
        None => {}
    }
}

#[post("/users/edit/<uid>", format = "application/json", data = "<data>")]
pub fn update_user(uid: i64, _user: Administrator, data: Json<EditUser>) -> Result<(), ()> {
    use club_coding::schema::users::dsl::*;
    let connection = establish_connection();

    diesel::update(users.find(uid))
        .set((
            username.eq(data.0.username.clone()),
            email.eq(data.0.email.clone()),
        ))
        .execute(&connection)
        .unwrap();

    let groups = get_all_groups_for_user(uid);
    for group in &groups {
        let mut should_delete = true;
        for data_group in &data.0.groups {
            if *group == *data_group {
                should_delete = false;
            }
        }
        if should_delete {
            use club_coding::schema::users_group::dsl::*;

            diesel::delete(
                users_group
                    .filter(user_id.eq(uid))
                    .filter(group_id.eq(*group)),
            ).execute(&connection)
                .unwrap();
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
            create_new_user_group(&connection, uid, *data_group);
        }
    }

    let series = get_all_series_for_user(uid);
    for serie in &series {
        let mut should_delete = true;
        for data_serie in &data.0.series {
            if *serie == *data_serie {
                should_delete = false;
            }
        }
        if should_delete {
            use club_coding::schema::users_series_access::dsl::*;

            diesel::delete(
                users_series_access
                    .filter(user_id.eq(uid))
                    .filter(bought.eq(false))
                    .filter(series_id.eq(*serie)),
            ).execute(&connection)
                .unwrap();
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
            create_new_user_series_access(&connection, uid, *data_serie, false);
        }
    }

    if data.0.force_resend_email {
        resend_confirmation_email(uid);
    }

    Ok(())
}

pub fn endpoints() -> Vec<Route> {
    routes![users, edit_users, update_user]
}
