use std::path::PathBuf;

use sqlx::{SqlitePool, Sqlite, Pool, Row};
use sqlx::migrate::MigrateDatabase;

mod user_info;

use time::OffsetDateTime;
pub use user_info::*;


pub async fn init_db_conn(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    match PathBuf::from(db_path).parent() {
        Some(p) => {
            match std::fs::create_dir_all(p) {
                Err(_) => println!("Error creating folder for database"),
                Ok(_) => {}
            };
        }
        None => {}
    };

    if !Sqlite::database_exists(db_path).await? {
        Sqlite::create_database(db_path).await?;
    }

    let db = SqlitePool::connect(db_path).await?;

    sqlx::migrate::Migrator::new(PathBuf::from("migrations"))
        .await?
        .run(&db)
        .await?;

    Ok(db)
}

pub async fn get_db_setup_time(db: &Pool<Sqlite>) -> Result<OffsetDateTime, ()> {
    let setup_query = sqlx::query("SELECT `installed_on` FROM `_sqlx_migrations` ORDER BY `installed_on` ASC LIMIT 1;");

    if let Ok(time) = setup_query.fetch_one(db).await {
        return Ok(time.get::<sqlx::types::time::OffsetDateTime, &str>("installed_on"));

    }

    return Err(());
}
