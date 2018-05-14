use rocket_contrib::Template;
use users::User;
use rocket::response::Redirect;
use club_coding::models::Groups;
use club_coding::{create_new_group, establish_connection};
use structs::LoggedInContext;
use chrono::NaiveDateTime;
use rocket_contrib::Json;
use diesel::prelude::*;
use rocket::request::Form;
use admin::generate_token;
use rocket::Route;

#[derive(Deserialize, Serialize)]
pub struct GroupC {
    uuid: String,
    name: String,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

pub fn get_all_groups() -> Vec<GroupC> {
    use club_coding::schema::groups::dsl::*;

    let connection = establish_connection();
    let result = groups
        .load::<Groups>(&connection)
        .expect("Error loading groups");

    let mut ret: Vec<GroupC> = vec![];

    for group in result {
        ret.push(GroupC {
            uuid: group.uuid,
            name: group.name,
            created: group.created,
            updated: group.updated,
        })
    }
    ret
}

#[derive(Serialize)]
struct GroupsContext {
    header: String,
    user: User,
    groups: Vec<GroupC>,
}

#[get("/groups")]
pub fn groups(user: User) -> Template {
    let context = GroupsContext {
        header: "Club Coding".to_string(),
        user: user,
        groups: get_all_groups(),
    };
    Template::render("admin/groups", &context)
}

#[get("/groups/new")]
pub fn new_group(user: User) -> Template {
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
pub fn insert_new_group(_user: User, group: Form<NewGroup>) -> Result<Redirect, Redirect> {
    let new_group: NewGroup = group.into_inner();
    let connection = establish_connection();
    match generate_token(24) {
        Ok(uuid) => {
            create_new_group(&connection, uuid.clone(), new_group.name);
            Ok(Redirect::to(&format!("/admin/groups/edit/{}", uuid)))
        }
        Err(_) => Err(Redirect::to("/admin/groups/new")),
    }
}

#[derive(Deserialize, Serialize)]
pub struct EditGroup {
    name: String,
}

#[derive(Serialize)]
struct EditGroupsContext {
    header: String,
    user: User,
    uuid: String,
    group: EditGroup,
}

fn get_group_by_uuid(uid: String) -> Option<Groups> {
    use club_coding::schema::groups::dsl::*;

    let connection = establish_connection();

    let group = groups
        .filter(uuid.eq(uid))
        .load::<Groups>(&connection)
        .expect("Unable to find groups");

    if group.len() == 1 {
        Some(group[0].clone())
    } else {
        None
    }
}

#[get("/groups/edit/<uuid>")]
pub fn edit_group(uuid: String, user: User) -> Option<Template> {
    match get_group_by_uuid(uuid.clone()) {
        Some(group) => {
            let context = EditGroupsContext {
                header: "Club Coding".to_string(),
                user: user,
                uuid: uuid,
                group: EditGroup { name: group.name },
            };
            Some(Template::render("admin/edit_group", &context))
        }
        None => None,
    }
}

#[post("/groups/edit/<uid>", format = "application/json", data = "<data>")]
pub fn update_group(uid: String, _user: User, data: Json<EditGroup>) -> Json<EditGroup> {
    use club_coding::schema::groups::dsl::*;

    let connection = establish_connection();

    diesel::update(groups.filter(uuid.eq(uid)))
        .set(name.eq(data.0.name.clone()))
        .execute(&connection)
        .unwrap();
    data
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
