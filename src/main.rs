use actix_files as fs;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashSet, net::SocketAddr, sync::Mutex};

struct AppState<'a> {
    bars: Mutex<Handlebars<'a>>,
    user_visits: Mutex<HashSet<u32>>,
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

#[get("/ip")]
async fn ip(req: HttpRequest) -> impl Responder {
    match req.peer_addr() {
        Some(addr) => return HttpResponse::Ok().body(addr.to_string()),
        None => return HttpResponse::Ok().body("You sneaky bastard"),
    }
}

async fn index(state: web::Data<AppState<'_>>) -> impl Responder {
    let bar_lock = match state.bars.lock() {
        Ok(x) => x,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let data_table_content = DataTableParams {
        ip: "255.255.255.255".to_owned(),
        visitor_rank: 12,
        last_start_time: "Oct 10th 2028".to_owned(),
        total_visitors: 6969,
    };


    let data_table_component = match bar_lock.render("data-table", &data_table_content ) {
        Ok(x) => x,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let index_content = IndexParams {
        data_table: data_table_component,
    };

    return HttpResponse::Ok().body(
        bar_lock
            .render("index", &index_content)
            .unwrap(),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let mut bars = Handlebars::new();
    bars.register_templates_directory(
        "./src/templates/",
        DirectorySourceOptions {
            tpl_extension: ".html".to_owned(),
            hidden: false,
            temporary: false,
        },
    )
    .unwrap();

    let user_visits: HashSet<u32> = HashSet::new();

    let state = web::Data::new(AppState {
        bars: Mutex::new(bars),
        user_visits: Mutex::new(user_visits),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(fs::Files::new("/static", "src/static/"))
            .default_service(web::route().to(index))
    })
    .bind(SocketAddr::from((
        [127, 0, 0, 1],
        std::env::var("PORT")
            .unwrap_or("8080".to_owned())
            .parse()
            .unwrap(),
    )))?
    .run()
    .await
}
