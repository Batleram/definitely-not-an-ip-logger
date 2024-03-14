mod conversion_utils;
mod template_models;

use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Executor, Pool, Row, Sqlite, SqlitePool};

use template_models::*;
use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
use std::path::PathBuf;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Mutex,
};
use time::OffsetDateTime;

struct AppState<'a> {
    bars: Mutex<Handlebars<'a>>,
    start_time: OffsetDateTime,
    db: Pool<Sqlite>,
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

async fn determine_user_rank(db: &Pool<Sqlite>, ip_str: &str) -> Result<u32, ()> {
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

async fn get_user_count(db: &Pool<Sqlite>) -> Option<u32> {
    let max_user_id = sqlx::query("SELECT `id` FROM `user_visits` ORDER BY `id` DESC LIMIT 1;");

    if let Ok(user_count) = max_user_id.fetch_one(db).await {
        return user_count.get("id");
    }

    return None;
}

async fn index(state: web::Data<AppState<'_>>, req: HttpRequest) -> impl Responder {
    let ip_str = match req.connection_info().realip_remote_addr() {
        Some(ip) => ip.to_string(),
        None => return HttpResponse::Ok().body("You sneaky bastard"),
    };

    let data_table_content = DataTableModel {
        ip: ip_str.to_string(),
        visitor_rank: {
            match determine_user_rank(&state.db, ip_str.as_str()).await {
                Ok(x) => x.clone(),
                Err(_) => {
                    println!("Was not able to set a rank for {}", ip_str);

                    u32::MAX
                }
            }
        },
        last_start_time: state
            .start_time
            .format(conversion_utils::TIME_FORMAT)
            .unwrap(),
        total_visitors: get_user_count(&state.db).await.unwrap_or(0),
    };

    let Ok(bar_lock) = state.bars.lock() else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(data_table_component) = bar_lock.render("data_table", &data_table_content) else {
        return HttpResponse::InternalServerError().finish();
    };

    let index_content = IndexModel {
        data_table: data_table_component,
    };

    return HttpResponse::Ok().body(bar_lock.render("index", &index_content).unwrap());
}

async fn init_db_conn(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let mut bars = Handlebars::new();
    bars.register_templates_directory(
        "./templates/",
        DirectorySourceOptions {
            tpl_extension: ".html".to_owned(),
            hidden: false,
            temporary: false,
        },
    )
    .unwrap();

    let db_pool = init_db_conn("persistence/amg_storage.sqlite3")
        .await
        .unwrap();

    let user_visits: HashMap<u32, u32> = HashMap::new();

    let state = web::Data::new(AppState {
        bars: Mutex::new(bars),
        start_time: OffsetDateTime::now_utc(),
        db: db_pool,
    });

    let port: u16 = std::env::var("PORT")
        .unwrap_or("8080".to_owned())
        .parse()
        .unwrap();

    let res = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(fs::Files::new("/static", "static/"))
            .default_service(web::route().to(index))
    })
    .bind(SocketAddr::from(([0, 0, 0, 0], port)))
    .unwrap()
    .run();

    println!("Started server on port: {}", port);

    return res.await;
}
