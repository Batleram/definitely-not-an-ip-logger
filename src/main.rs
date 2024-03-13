use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
use serde::{Deserialize, Serialize};
use core::sync;
use std::{collections::HashMap, net::SocketAddr, sync::{Mutex, Arc, atomic::AtomicU32}};
use time::{format_description, OffsetDateTime};

struct AppState<'a> {
    bars: Mutex<Handlebars<'a>>,
    user_visits: Mutex<HashMap<u32, u32>>,
    user_count: Arc<AtomicU32>,
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

fn ip_to_i32(ip: String) -> Option<u32> {
    let port_split: Vec<&str> = ip.split(":").collect();
    if port_split.len() < 1 {
        return None;
    }

    let ip_split: Vec<&str> = port_split[0].split(".").collect();
    if ip_split.len() < 4 {
        return None;
    }

    let mut out: u32 = 0;

    for i in 0..4 {
        match ip_split[i].parse::<u32>() {
            Ok(s) => out += s * (i as u32 + 1),
            Err(_) => return None,
        };
    }

    return Some(out);
}

async fn index(state: web::Data<AppState<'_>>, req: HttpRequest) -> impl Responder {
    let time_format = format_description::parse(
        "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second]",
    )
    .unwrap();

    let ip_str = match req.connection_info().realip_remote_addr() {
        Some(addr) => addr.to_string(),
        None => return HttpResponse::Ok().body("You sneaky bastard"),
    };

    let ip_int = match ip_to_i32(ip_str.to_owned()) {
        Some(addr) => addr,
        None => return HttpResponse::Ok().body("You sneaky bastard"),
    };

    let mut user_visit_lock = match state.user_visits.lock() {
        Ok(x) => x,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let data_table_content = DataTableParams {
        ip: ip_str,
        visitor_rank: {
            match user_visit_lock.get(&ip_int) {
                Some(x) => x.clone(),
                None => {
                    // not sure what the different orderings do...
                    let prev = state.user_count.fetch_add(1, sync::atomic::Ordering::Relaxed);
                    user_visit_lock.insert(ip_int, prev+1);

                    prev+1
                }
            }
        },
        last_start_time: state.start_time.format(&time_format).unwrap(),
        total_visitors: state.user_count.fetch_add(0, sync::atomic::Ordering::Relaxed),
    };

    let bar_lock = match state.bars.lock() {
        Ok(x) => x,
        Err(_) => return HttpResponse::InternalServerError().finish(),
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

    let user_visits: HashMap<u32, u32> = HashMap::new();

    let state = web::Data::new(AppState {
        bars: Mutex::new(bars),
        user_visits: Mutex::new(user_visits),
        user_count: Arc::new(AtomicU32::new(0)),
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
