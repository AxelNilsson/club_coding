use rocket_contrib::Template;
use admin::structs::{Administrator, LoggedInContext};
use rocket::response::Redirect;
use club_coding::models::Groups;
use club_coding::create_new_group;
use database::DbConn;
use chrono::NaiveDateTime;
use rocket_contrib::Json;
use diesel::prelude::*;
use rocket::request::Form;
use admin::generate_token;
use rocket::Route;

#[derive(Deserialize, Serialize)]
pub struct GroupC {
    id: i64,
    name: String,
}

pub fn get_all_groupsc(connection: &DbConn) -> Vec<GroupC> {
    use club_coding::schema::groups::dsl::*;

    match groups.load::<Groups>(&**connection) {
        Ok(result) => {
            let mut ret: Vec<GroupC> = vec![];

            for group in result {
                ret.push(GroupC {
                    id: group.id,
                    name: group.name,
                })
            }
            ret
        }
        Err(_) => vec![],
    }
}

#[derive(Deserialize, Serialize)]
pub struct GroupContext {
    uuid: String,
    name: String,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

pub fn get_all_groups(connection: &DbConn) -> Vec<GroupContext> {
    use club_coding::schema::groups::dsl::*;

    match groups.load::<Groups>(&**connection) {
        Ok(result) => {
            let mut ret: Vec<GroupContext> = vec![];

            for group in result {
                ret.push(GroupContext {
                    uuid: group.uuid,
                    name: group.name,
                    created: group.created,
                    updated: group.updated,
                })
            }
            ret
        }
        Err(_) => vec![],
    }
}

#[derive(Serialize)]
struct GroupsContext {
    header: String,
    user: Administrator,
    groups: Vec<GroupContext>,
}

#[get("/groups")]
pub fn groups(conn: DbConn, user: Administrator) -> Template {
    let context = GroupsContext {
        header: "Club Coding".to_string(),
        user: user,
        groups: get_all_groups(&conn),
    };
    Template::render("admin/groups", &context)
}

#[get("/groups/new")]
pub fn new_group(user: Administrator) -> Template {
    let context = LoggedInContext {
        header: "Club Coding".to_string(),
        user: user,
    };
    Template::render("admin/new_group", &context)
}

#[derive(FromForm)]
pub struct NewGroup {
    name: String,
}

#[post("/groups/new", data = "<group>")]
pub fn insert_new_group(
    conn: DbConn,
    _user: Administrator,
    group: Form<NewGroup>,
) -> Result<Redirect, Redirect> {
    let new_group: NewGroup = group.into_inner();
    match generate_token(24) {
        Ok(uuid) => match create_new_group(&*conn, &uuid, &new_group.name) {
            Ok(_) => Ok(Redirect::to(&format!("/admin/groups/edit/{}", uuid))),
            Err(_) => Ok(Redirect::to(&format!("/admin/groups/edit/{}", uuid))),
        },
        Err(_) => Err(Redirect::to("/admin/groups/new")),
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditGroup {
    name: String,
}

#[derive(Serialize)]
struct EditGroupsContext<'a> {
    header: String,
    user: Administrator,
    uuid: &'a String,
    group: EditGroup,
}

fn get_group_by_uuid<'a>(connection: &DbConn, uid: &'a String) -> Option<Groups> {
    use club_coding::schema::groups::dsl::*;

    let result = match groups.filter(uuid.eq(uid)).load::<Groups>(&**connection) {
        Ok(result) => result,
        Err(_) => None,
    };

    if result.len() == 1 {
        Some(result[0].clone())
    } else {
        None
    }
}

#[get("/groups/edit/<uuid>")]
pub fn edit_group(conn: DbConn, uuid: String, user: Administrator) -> Option<Template> {
    match get_group_by_uuid(&conn, &uuid) {
        Some(group) => {
            let context = EditGroupsContext {
                header: "Club Coding".to_string(),
                user: user,
                uuid: &uuid,
                group: EditGroup { name: group.name },
            };
            Some(Template::render("admin/edit_group", &context))
        }
        None => None,
    }
}

#[post("/groups/edit/<uid>", format = "application/json", data = "<data>")]
pub fn update_group(
    conn: DbConn,
    uid: String,
    _user: Administrator,
    data: Json<EditGroup>,
) -> Result<(), ()> {
    use club_coding::schema::groups::dsl::*;

    match diesel::update(groups.filter(uuid.eq(uid)))
        .set(name.eq(data.0.name))
        .execute(&*conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub fn endpoints() -> Vec<Route> {
    routes![
        groups,
        new_group,
        insert_new_group,
        edit_group,
        update_group,
    ]
}
