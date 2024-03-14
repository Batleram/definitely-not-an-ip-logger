mod conversion_utils;
mod persistence;
mod template_models;

use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use conversion_utils::TIME_FORMAT;
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
use sqlx::{Pool, Sqlite};
use std::{net::SocketAddr, sync::Mutex};
use template_models::*;
use time::OffsetDateTime;

struct AppState<'a> {
    bars: Mutex<Handlebars<'a>>,
    db: Pool<Sqlite>,
}

async fn index(state: web::Data<AppState<'_>>, req: HttpRequest) -> impl Responder {
    let ip_str = match req.connection_info().realip_remote_addr() {
        Some(ip) => ip.to_string(),
        None => return HttpResponse::Ok().body("You sneaky bastard"),
    };

    let data_table_content = DataTableModel {
        ip: ip_str.to_string(),
        visitor_rank: {
            match persistence::determine_user_rank(&state.db, ip_str.as_str()).await {
                Ok(x) => x.clone(),
                Err(_) => {
                    println!("Was not able to set a rank for {}", ip_str);

                    u32::MAX
                }
            }
        },
        db_init_time: persistence::get_db_setup_time(&state.db)
            .await
            .unwrap_or(OffsetDateTime::UNIX_EPOCH)
            .format(TIME_FORMAT)
            .unwrap_or("Unable to format time".to_owned()),
        total_visitors: persistence::get_user_count(&state.db).await.unwrap_or(0),
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

    let db_pool = persistence::init_db_conn("database/notiplogger_storage.sqlite3")
        .await
        .unwrap();

    let state = web::Data::new(AppState {
        bars: Mutex::new(bars),
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
