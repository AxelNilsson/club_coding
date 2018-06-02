use rocket::Route;

mod database;
mod register;
mod login;
mod recover;
pub mod verify;

/// Generate a random token of size length.
fn generate_token(length: u8) -> String {
    let bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
    let strings: Vec<String> = bytes.iter().map(|byte| format!("{:02X}", byte)).collect();
    return strings.join("");
}

/// Assembles all of the endpoints of the authentication
/// endpoints. The upside of assembling all of the endpoints
/// here is that we don't have to update the main function
/// but instead we can keep all of the changes in here.
pub fn endpoints() -> Vec<Route> {
    let mut total = vec![];

    let mut register = register::endpoints();
    total.append(&mut register);

    let mut login = login::endpoints();
    total.append(&mut login);

    let mut recover = recover::endpoints();
    total.append(&mut recover);

    let mut verify = verify::endpoints();
    total.append(&mut verify);

    total
}
