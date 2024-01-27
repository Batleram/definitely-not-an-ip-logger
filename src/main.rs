use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars, Renderable};
use serde_json::json;
use std::{collections::HashSet, net::SocketAddr, sync::Mutex};

struct AppState<'a> {
    bars: Mutex<Handlebars<'a>>,
    user_visits: Mutex<HashSet<u32>>,
}

#[get("/ip")]
async fn ip(req: HttpRequest) -> impl Responder {
    match req.peer_addr() {
        Some(addr) => return HttpResponse::Ok().body(addr.to_string()),
        None => return HttpResponse::Ok().body("You sneaky bastard"),
    }
}

async fn index(state: web::Data<AppState<'_>>) -> impl Responder {
    return HttpResponse::Ok().body(
        state
            .bars
            .lock()
            .unwrap()
            .render("index", &json!({"testString": "asdfasdf"}))
            .unwrap(),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let mut bars = Handlebars::new();
    // bars.register_template_file("index", "src/templates/index.html").unwrap();
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
