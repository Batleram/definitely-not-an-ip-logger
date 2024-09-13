use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Executor, Pool, Row, Sqlite};

use crate::conversion_utils;

pub async fn determine_user_rank(db: &Pool<Sqlite>, ip_str: &str) -> Result<u32, ()> {
    let ip_dec = conversion_utils::ip_to_i32(ip_str.to_string()).unwrap();

    if let Some(rank) = get_user_id(db, ip_dec).await {
        return Ok(rank);
    }

    if let Err(e) = insert_user(db, ip_dec).await {
        println!("Error while inserting user: {} | {:#?}", ip_str, e);
        return Err(());
    }

    if let Some(rank) = get_user_id(db, ip_dec).await {
        return Ok(rank);
    }

    return Err(());
}

pub async fn get_total_user_count(db: &Pool<Sqlite>) -> Option<u32> {
    let max_user_id = sqlx::query("SELECT `id` FROM `user_visits` ORDER BY `id` DESC LIMIT 1;");

    if let Ok(user_count) = max_user_id.fetch_one(db).await {
        return user_count.get("id");
    }

    return None;
}


pub async fn get_user_count(db: &Pool<Sqlite>) -> Option<u32> {
    let max_user_id = sqlx::query("SELECT `id` FROM `user_visits` WHERE `is_bot` = FALSE ORDER BY `id` DESC LIMIT 1;");

    if let Ok(user_count) = max_user_id.fetch_one(db).await {
        return user_count.get("id");
    }

    return None;
}

pub async fn get_bot_count(db: &Pool<Sqlite>) -> Option<u32> {
    let max_user_id = sqlx::query("SELECT `id` FROM `user_visits` WHERE `is_bot` = TRUE ORDER BY `id` DESC LIMIT 1;");

    if let Ok(user_count) = max_user_id.fetch_one(db).await {
        return user_count.get("id");
    }

    return None;
}


async fn get_user_id(db: &Pool<Sqlite>, ip_dec: u32) -> Option<u32> {
    let rank_query = sqlx::query(
        "SELECT `id`, `user` FROM `user_visits` WHERE `user` = ? ORDER BY `id` DESC LIMIT 1;",
    )
    .bind(ip_dec);

    let query_res = rank_query.fetch_optional(db).await;

    if let Err(e) = query_res {
        println!(
            "Error in get user id query for dec ip: {} | {:#?}",
            ip_dec, e
        );

        return None;
    }

    if let Some(row) = query_res.unwrap() {
        return Some(row.get("id"));
    }

    return None;
}

async fn insert_user(db: &Pool<Sqlite>, ip_dec: u32) -> Result<SqliteQueryResult, sqlx::Error> {
    let rank_insert = sqlx::query("INSERT INTO `user_visits` (user) VALUES (?);").bind(ip_dec);

    return db.execute(rank_insert).await;
}
