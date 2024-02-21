use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Mutex};
use time::{format_description, OffsetDateTime};

struct AppState<'a> {
    bars: Mutex<Handlebars<'a>>,
    user_visits: Mutex<HashMap<u32, i32>>,
    user_count: i32,
    start_time: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
struct DataTableParams {
    ip: String,
    visitor_rank: u32,
    last_start_time: String,
    total_visitors: u32,
}

#[derive(Serialize, Deserialize)]
struct IndexParams {
    data_table: String,
}

async fn index(state: web::Data<AppState<'_>>, req: HttpRequest) -> impl Responder {
    let bar_lock = match state.bars.lock() {
        Ok(x) => x,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let ip = match req.connection_info().realip_remote_addr() {
        Some(addr) => addr.to_string(),
        None => return HttpResponse::Ok().body("You sneaky bastard"),
    };

    let time_format = format_description::parse("[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second]").unwrap();

    let data_table_content = DataTableParams {
        ip,
        visitor_rank: 12,
        last_start_time: state.start_time.format(&time_format).unwrap(),
        total_visitors: 6969,
    };

    let data_table_component = match bar_lock.render("data-table", &data_table_content) {
        Ok(x) => x,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let index_content = IndexParams {
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

    let user_visits: HashMap<u32, i32> = HashMap::new();

    let state = web::Data::new(AppState {
        bars: Mutex::new(bars),
        user_visits: Mutex::new(user_visits),
        user_count: 0,
        start_time: OffsetDateTime::now_local().unwrap(),
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
