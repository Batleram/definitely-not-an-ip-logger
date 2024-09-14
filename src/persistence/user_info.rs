use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Executor, Pool, Row, Sqlite};

use crate::conversion_utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInfo {
    pub ip_str: String,
    pub ip_dec: u32,
    pub rank: u32,
    pub is_bot: bool,
}

impl UserInfo {
    pub fn new(ip_dec: u32, rank: u32, is_bot: bool) -> Self {
        return UserInfo {
            ip_str: conversion_utils::u32_to_ip(ip_dec),
            ip_dec,
            rank,
            is_bot,
        };
    }
}

pub async fn get_user_info(db: &Pool<Sqlite>, ip_dec: u32) -> Result<UserInfo, ()> {
    if let Some(user_info) = get_user_by_id(db, ip_dec).await {
        return Ok(user_info);
    }

    if let Err(e) = insert_user(db, ip_dec).await {
        println!("Error while inserting user: {} | {:#?}", ip_dec, e);
        return Err(());
    }

    if let Some(user_info) = get_user_by_id(db, ip_dec).await {
        return Ok(user_info);
    }

    return Err(());
}

pub async fn is_user_bot(db: &Pool<Sqlite>, ip_dec: u32) -> Result<bool, ()> {
    if let Some(user_info) = get_user_by_id(db, ip_dec).await {
        return Ok(user_info.is_bot);
    }

    if let Err(e) = insert_user(db, ip_dec).await {
        println!("Error while inserting user: {} | {:#?}", ip_dec, e);
        return Err(());
    }

    // New users are automatically bots, they become not bots once they're verified
    return Ok(true);
}

pub async fn get_total_user_count(db: &Pool<Sqlite>) -> Option<u32> {
    let max_user_id = sqlx::query("SELECT count(*) as count FROM `user_visits`;");

    if let Ok(user_count) = max_user_id.fetch_one(db).await {
        return user_count.get("count");
    }

    return None;
}

pub async fn get_user_count(db: &Pool<Sqlite>) -> Option<u32> {
    let max_user_id = sqlx::query(
        "SELECT count(*) as count FROM `user_visits` WHERE `is_bot` = FALSE;",
    );

    if let Ok(user_count) = max_user_id.fetch_one(db).await {
        return user_count.get("count");
    }

    return None;
}

pub async fn get_bot_count(db: &Pool<Sqlite>) -> Option<u32> {
    let max_user_id = sqlx::query(
        "SELECT count(*) as count FROM `user_visits` WHERE `is_bot` = TRUE;",
    );

    if let Ok(user_count) = max_user_id.fetch_one(db).await {
        return user_count.get("count");
    }

    return None;
}

pub async fn set_user_not_bot(
    db: &Pool<Sqlite>,
    ip_dec: u32,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let set_not_bot =
        sqlx::query("UPDATE `user_visits` SET `is_bot` = FALSE WHERE `user` = ?").bind(ip_dec);

    return db.execute(set_not_bot).await;
}

async fn get_user_by_id(db: &Pool<Sqlite>, ip_dec: u32) -> Option<UserInfo> {
    let rank_query = sqlx::query(
        "SELECT `id`, `user`, `is_bot` FROM `user_visits` WHERE `user` = ? ORDER BY `id` DESC LIMIT 1;",
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
        return Some(UserInfo::new(ip_dec, row.get("id"), row.get("is_bot")));
    }

    return None;
}

async fn insert_user(db: &Pool<Sqlite>, ip_dec: u32) -> Result<SqliteQueryResult, sqlx::Error> {
    let rank_insert = sqlx::query("INSERT INTO `user_visits` (user) VALUES (?);").bind(ip_dec);

    return db.execute(rank_insert).await;
}
