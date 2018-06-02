use club_coding::models::{Series, UsersViews, Videos};
use database::DbConn;
use series::{PublicSeries, PublicVideo};
use diesel::prelude::*;

/// Gets all of the videos in the
/// database that are published and
/// not archived by the order of
/// their update date in an ascending
/// order.
pub fn get_series(connection: &DbConn) -> Vec<Series> {
    use club_coding::schema::series::dsl::*;

    match series
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .order(updated.asc())
        .load::<Series>(&**connection)
    {
        Ok(vec_of_series) => vec_of_series,
        Err(_) => vec![],
    }
}

/// Gets the last 10 series in the
/// database that are published and
/// not archived by the order of
/// their id in an ascending
/// order.
pub fn get_last_10_series(connection: &DbConn) -> Vec<PublicSeries> {
    use club_coding::schema::series::dsl::*;

    match series
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .order(id.asc())
        .load::<Series>(&**connection)
    {
        Ok(s_eries) => {
            let mut to_return: Vec<PublicSeries> = vec![];
            for serie in s_eries {
                let mut mut_description = serie.description;
                mut_description.retain(|c| c != '\\');
                to_return.push(PublicSeries {
                    uuid: serie.uuid,
                    title: serie.title,
                    slug: serie.slug,
                    description: mut_description,
                    price: serie.price,
                });
            }
            to_return
        }
        Err(_) => vec![],
    }
}

/// Gets a specific serie in the
/// database specified by the UUID.
/// Returns some Series if it is
/// found and otherwise returns
/// None.
pub fn get_serie(connection: &DbConn, uid: &String) -> Option<Series> {
    use club_coding::schema::series::dsl::*;

    match series
        .filter(uuid.eq(uid))
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .first(&**connection)
    {
        Ok(serie) => Some(serie),
        Err(_) => None,
    }
}

/// Checks if the user defined by
/// the user_id has watched the video
/// defined by the video id. Returns
/// a boolean True if the user has watched
/// the video  and false if the user has
/// not watched the video.
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

/// Gets all of the videos that belong
/// to a specific series and checks if
/// the user has watched the videos.
pub fn get_videos(connection: &DbConn, uid: i64, sid: i64) -> Vec<PublicVideo> {
    use club_coding::schema::videos::dsl::*;

    match videos
        .filter(serie_id.eq(sid))
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .order(episode_number.asc())
        .load::<Videos>(&**connection)
    {
        Ok(vec_of_videos) => {
            let mut to_return: Vec<PublicVideo> = vec![];
            for video in vec_of_videos {
                to_return.push(PublicVideo {
                    episode_number: video.episode_number,
                    uuid: video.uuid,
                    title: video.title,
                    description: video.description,
                    watched: get_video_watched(connection, uid, video.id),
                });
            }
            to_return
        }
        Err(_) => vec![],
    }
}

/// Gets all of the videos that belong
/// to a specific series and sets watched
/// of every video to false
pub fn get_videos_nologin(connection: &DbConn, sid: i64) -> Vec<PublicVideo> {
    use club_coding::schema::videos::dsl::*;

    match videos
        .filter(serie_id.eq(sid))
        .filter(published.eq(true))
        .filter(archived.eq(false))
        .order(episode_number.asc())
        .load::<Videos>(&**connection)
    {
        Ok(v_ideos) => {
            let mut to_return: Vec<PublicVideo> = vec![];
            for video in v_ideos {
                to_return.push(PublicVideo {
                    episode_number: video.episode_number,
                    uuid: video.uuid,
                    title: video.title,
                    description: video.description,
                    watched: false,
                });
            }
            to_return
        }
        Err(_) => vec![],
    }
}
