use std::path::PathBuf;

use sqlx::{SqlitePool, Sqlite};
use sqlx::migrate::MigrateDatabase;

mod user_info;

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

