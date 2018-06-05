use club_coding::create_new_user_view;
use club_coding::models::{Users, RequestNetworkPayments, Series, UsersSeriesAccess, UsersStripeCustomer,
                          UsersViews, VideoJoin, Videos};
use std::io::{Error, ErrorKind};
use database::DbConn;
use diesel::prelude::*;

/// Gets all of the videos in the
/// database that are published and
/// not archived by the order of
/// their creation date in an ascending
/// order.
pub fn get_videos(connection: &DbConn) -> Vec<Videos> {
    use club_coding::schema::videos::dsl::*;

    match videos
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .order(created.asc())
        .load::<Videos>(&**connection)
    {
        Ok(vec_of_vids) => vec_of_vids,
        Err(_) => vec![],
    }
}

/// Gets video data and related series
/// data that the watch endpoint requres.
pub fn get_video_data_from_uuid(connection: &DbConn, uid: &str) -> Result<VideoJoin, Error> {
    use club_coding::schema::{series, videos};

    match videos::table
        .inner_join(series::table.on(series::id.eq(videos::serie_id)))
        .filter(videos::uuid.eq(uid))
        .filter(videos::published.eq(true))
        .filter(videos::archived.eq(false))
        .select((
            videos::id,
            videos::uuid,
            videos::title,
            videos::description,
            videos::serie_id,
            videos::vimeo_id,
            videos::membership_only,
            series::title,
            series::price,
            series::in_development,
        ))
        .first::<VideoJoin>(&**connection)
    {
        Ok(result) => Ok(result),
        Err(_) => return Err(Error::new(ErrorKind::Other, "error loading videos")),
    }
}

/// Creates a new view in the database if
/// the user does not already have it.
/// The view is specified by video and user.
pub fn create_new_view(connection: &DbConn, vid: i64, uid: i64) {
    use club_coding::schema::users_views::dsl::*;

    match users_views
        .filter(user_id.eq(uid))
        .filter(video_id.eq(vid))
        .first::<UsersViews>(&**connection)
    {
        Ok(_) => {}
        Err(_) => match create_new_user_view(&connection, uid, vid) {
            Ok(_) => {}
            Err(_) => {}
        },
    }
}

/// Checks if a user has bought a series.
/// Returns a boolean of if the user has
/// bought the series or not.
pub fn user_has_bought(connection: &DbConn, sid: i64, uid: i64) -> bool {
    use club_coding::schema::users_series_access::dsl::*;

    match users_series_access
        .filter(user_id.eq(uid))
        .filter(series_id.eq(sid))
        .first::<UsersSeriesAccess>(&**connection)
    {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Gets a Option Stripe Customer as specified
/// by the User ID. Returns either Some Stripe
/// Customer or None if the customer does not
/// exist.
pub fn get_customer(connection: &DbConn, uid: i64) -> Option<UsersStripeCustomer> {
    use club_coding::schema::users_stripe_customer::dsl::*;

    let result: UsersStripeCustomer = match users_stripe_customer
        .filter(user_id.eq(uid))
        .limit(1)
        .first(&**connection)
    {
        Ok(result) => result,
        Err(_) => return None,
    };

    Some(result)
}

/// Gets a serie.
/// Returns either Some Series or None
/// if the series does not exist.
pub fn get_serie(connection: &DbConn, sid: i64) -> Option<Series> {
    use club_coding::schema::series::dsl::*;

    match series.filter(id.eq(sid)).first(&**connection) {
        Ok(serie) => Some(serie),
        Err(_) => None,
    }
}

/// Gets a request network payment.
/// Returns either Some Series or None
/// if the request network payment does not exist.
pub fn get_request_payment(conn: &DbConn, token: &str) -> Option<RequestNetworkPayments> {
    use club_coding::schema::request_network_payments;

    match request_network_payments::table
        .filter(request_network_payments::uuid.eq(token))
        .first::<RequestNetworkPayments>(&**conn)
    {
        Ok(request_payment) => Some(request_payment),
        Err(_) => None,
    }
}

/// Invalidates request network payment.
/// Returns either OK or Error if the
/// request network payment does not
/// exist.
pub fn invalidate_request_payment(conn: &DbConn, request_network_id: i64) -> Result<(), ()> {
    use club_coding::schema::request_network_payments;

    match diesel::update(request_network_payments::table.find(request_network_id))
        .set(request_network_payments::used.eq(true))
        .execute(&**conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}


/// Gets a request network payment.
/// Returns either Some Series or None
/// if the request network payment does not exist.
pub fn get_user(conn: &DbConn, user_id: i64) -> Option<Users> {
    use club_coding::schema::users;

    match users::table
        .find(user_id)
        .first::<Users>(&**conn)
    {
        Ok(user) => Some(user),
        Err(_) => None,
    }
}
