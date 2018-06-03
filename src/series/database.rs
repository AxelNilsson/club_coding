use club_coding::models::{Series, UsersViews, Videos};
use database::{DbConn, RedisConnection};
use series::{PublicSeries, PublicVideo};
use redis::Commands;
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
pub fn get_last_10_series(mysql_conn: &DbConn, redis_conn: RedisConnection) -> Vec<PublicSeries> {
    match redis_conn.get::<&str, String>("last_10") {
        Ok(result) => {
            let v: Vec<PublicSeries> = serde_json::from_str(&result).unwrap();
            return v;
        }
        Err(_) => {
            use club_coding::schema::series::dsl::*;

            match series
                .filter(published.eq(true))
                .filter(archived.eq(false))
                .order(id.asc())
                .load::<Series>(&**mysql_conn)
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
                    let json_string = match serde_json::to_string(&to_return) {
                        Ok(json_string) => json_string,
                        Err(_) => return to_return,
                    };
                    match redis_conn.set::<&str, String, String>("last_10", json_string) {
                        Ok(_) => {}
                        Err(_) => {}
                    }

                    to_return
                }
                Err(_) => vec![],
            }
        }
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
pub fn get_videos(
    connection: &DbConn,
    redis_conn: RedisConnection,
    uid: i64,
    sid: i64,
) -> Vec<PublicVideo> {
    match redis_conn.get::<&str, String>(&format!("serie:{}", sid)) {
        Ok(result) => {
            let mut videos: Vec<PublicVideo> = serde_json::from_str(&result).unwrap();
            for mut video in &mut videos {
                video.watched = get_video_watched(connection, uid, video.id);
            }
            return videos;
        }
        Err(_) => {
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
                            id: video.id,
                            episode_number: video.episode_number,
                            uuid: video.uuid,
                            title: video.title,
                            description: video.description,
                            watched: get_video_watched(connection, uid, video.id),
                        });
                    }
                    let json_string = match serde_json::to_string(&to_return) {
                        Ok(json_string) => json_string,
                        Err(_) => return to_return,
                    };
                    match redis_conn
                        .set::<&str, String, String>(&format!("serie:{}", sid), json_string)
                    {
                        Ok(_) => {}
                        Err(_) => {}
                    }

                    to_return
                }
                Err(_) => vec![],
            }
        }
    }
}

/// Gets all of the videos that belong
/// to a specific series and sets watched
/// of every video to false
pub fn get_videos_nologin(
    connection: &DbConn,
    redis_conn: RedisConnection,
    sid: i64,
) -> Vec<PublicVideo> {
    match redis_conn.get::<&str, String>(&format!("serie:{}", sid)) {
        Ok(result) => {
            let mut videos: Vec<PublicVideo> = serde_json::from_str(&result).unwrap();
            for mut video in &mut videos {
                video.watched = false;
            }
            return videos;
        }
        Err(_) => {
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
                            id: video.id,
                            episode_number: video.episode_number,
                            uuid: video.uuid,
                            title: video.title,
                            description: video.description,
                            watched: false,
                        });
                    }
                    let json_string = match serde_json::to_string(&to_return) {
                        Ok(json_string) => json_string,
                        Err(_) => return to_return,
                    };
                    match redis_conn
                        .set::<&str, String, String>(&format!("serie:{}", sid), json_string)
                    {
                        Ok(_) => {}
                        Err(_) => {}
                    }

                    to_return
                }
                Err(_) => vec![],
            }
        }
    }
}
