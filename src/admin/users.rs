use rocket_contrib::Template;
use admin::structs::Administrator;
use club_coding::models::{Groups, Users, UsersGroup};
use club_coding::{create_new_user_group, establish_connection};
use chrono::NaiveDateTime;
use rocket_contrib::Json;
use diesel::prelude::*;
use admin::group::get_all_groups;
use admin::group::GroupC;
use rocket::Route;

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

pub fn get_all_groups_for_user(uid: i64) -> Vec<String> {
    use club_coding::schema::users_group::dsl::*;

    let connection = establish_connection();

    let matching_groups = users_group
        .filter(user_id.eq(uid))
        .load::<UsersGroup>(&connection)
        .expect("Unable to find users group");

    let mut returning_groups = vec![];

    for group in matching_groups {
        use club_coding::schema::groups::dsl::*;

        let connection = establish_connection();

        let m_groups = groups
            .filter(id.eq(group.group_id))
            .load::<Groups>(&connection)
            .expect("Unable to find group");

        if m_groups.len() == 1 {
            returning_groups.push(m_groups[0].uuid.clone());
        }
    }
    returning_groups
}

#[derive(Deserialize, Serialize)]
pub struct EditUser {
    email: String,
    groups: Vec<String>,
    force_change_password: bool,
    force_resend_email: bool,
    free_membership: bool,
    deactivated: bool,
}

#[derive(Serialize)]
struct EditUsersContext<'a> {
    header: String,
    user: &'a Administrator,
    uuid: String,
    user_data: EditUser,
    groups: Vec<GroupC>,
}

#[get("/users/edit/<uuid>")]
pub fn edit_users(uuid: String, user: Administrator) -> Template {
    let context = EditUsersContext {
        header: "Club Coding".to_string(),
        user: &user,
        uuid: uuid,
        groups: get_all_groups(),
        user_data: EditUser {
            email: user.username.clone(),
            groups: get_all_groups_for_user(user.id),
            force_change_password: false,
            force_resend_email: false,
            free_membership: false,
            deactivated: true,
        },
    };
    Template::render("admin/edit_user", &context)
}

#[post("/users/edit/<uid>", format = "application/json", data = "<data>")]
pub fn update_user(uid: i64, _user: Administrator, data: Json<EditUser>) -> Result<(), ()> {
    use club_coding::schema::users::dsl::*;
    let connection = establish_connection();

    diesel::update(users.find(uid))
        .set(username.eq(data.0.email.clone()))
        .execute(&connection)
        .unwrap();

    for group in data.0.groups {
        use club_coding::schema::groups::dsl::*;

        let m_groups = groups
            .filter(uuid.eq(group))
            .limit(1)
            .load::<Groups>(&connection)
            .expect("Unable to find group");

        if m_groups.len() == 1 {
            use club_coding::schema::users_group::dsl::*;

            let usergroups = users_group
                .filter(user_id.eq(uid))
                .filter(group_id.eq(m_groups[0].id))
                .limit(1)
                .load::<UsersGroup>(&connection)
                .expect("Unable to find user group");

            if usergroups.len() == 0 {
                create_new_user_group(&connection, uid, m_groups[0].id);
            }
        }
    }

    Ok(())
}

pub fn endpoints() -> Vec<Route> {
    routes![users, edit_users, update_user]
}
