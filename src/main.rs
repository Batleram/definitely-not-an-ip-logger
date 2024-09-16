mod conversion_utils;
mod persistence;
mod template_models;

use actix_files as actix_filesystem;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use conversion_utils::TIME_FORMAT;
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{Pool, Sqlite};
use std::{collections::HashMap, net::SocketAddr, sync::Mutex};
use template_models::*;
use time::OffsetDateTime;

static VALIDATION_TIMEOUT: time::Duration = time::Duration::seconds(10);

#[derive(Debug)]
struct AppState<'a> {
    bars: Mutex<Handlebars<'a>>,
    db: Pool<Sqlite>,
    pending_validations: Mutex<HashMap<String, PendingValidation>>,
}

#[derive(Debug, Clone)]
struct PendingValidation {
    user: u32,
    unique_id: String,
    timestamp: OffsetDateTime,
}

impl PendingValidation {
    pub fn new(user: u32) -> Self {
        return PendingValidation {
            user,
            unique_id: rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect(),
            timestamp: OffsetDateTime::now_utc(),
        };
    }

    pub fn is_expired(&self) -> bool {
        return OffsetDateTime::now_utc() - self.timestamp > VALIDATION_TIMEOUT;
    }
}

#[get("/")]
async fn index(state: web::Data<AppState<'_>>, req: HttpRequest) -> impl Responder {
    let ip_str = match req.connection_info().realip_remote_addr() {
        Some(ip) => ip.to_string(),
        None => return HttpResponse::Ok().body("You sneaky bastard"),
    };

    let ip_dec = match conversion_utils::ip_to_u32(ip_str.to_string()) {
        Some(ip) => ip,
        None => {
            return HttpResponse::Ok().body("Idk how you managed this, but your ip is invalid...")
        }
    };

    let user_info = match persistence::get_user_info(&state.db, ip_dec).await {
        Ok(x) => x,
        Err(_) => {
            println!("Was not able to get/create user info for: {}", ip_str);
            return HttpResponse::InternalServerError()
                .body("Failed to setup your user, please open an issue on my github");
        }
    };

    // Generate a pending validation
    let Ok(mut validation_map_lock) = state.pending_validations.lock() else {
        return HttpResponse::InternalServerError().finish();
    };

    let validation = PendingValidation::new(user_info.ip_dec);
    // create a local copy of the id becasue we move the validation to the hashmap
    let unique_id = validation.unique_id.clone();

    validation_map_lock.insert(unique_id.clone(), validation);

    let data_table_content = DataTableModel {
        ip: ip_str.to_string(),
        visitor_rank: user_info.rank,
        db_init_time: persistence::get_db_setup_time(&state.db)
            .await
            .unwrap_or(OffsetDateTime::UNIX_EPOCH)
            .format(TIME_FORMAT)
            .unwrap_or("Unable to format time".to_owned()),
        total_visitors: persistence::get_total_user_count(&state.db)
            .await
            .unwrap_or(0),
        total_bots: persistence::get_bot_count(&state.db).await.unwrap_or(0),
        bot_validation_id: unique_id.clone(),
    };

    let Ok(bar_lock) = state.bars.lock() else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(data_table_component) = bar_lock.render("data_table", &data_table_content) else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(chat_bot_component) = bar_lock.render("chat_bot", &data_table_content) else {
        return HttpResponse::InternalServerError().finish();
    };

    let index_content = IndexModel {
        data_table: data_table_component,
        chat_bot: chat_bot_component
    };

    return HttpResponse::Ok().body(bar_lock.render("index", &index_content).unwrap());
}

#[get("/contentrender/{render_id}")]
async fn bot_reply(state: web::Data<AppState<'_>>, req: HttpRequest) -> impl Responder {
    let render_id = match req.match_info().get("render_id") {
        Some(render_id) => render_id,
        None => {
            return HttpResponse::BadRequest().body("Missing render_id parameter");
        }
    };

    let Ok(mut validation_map_lock) = state.pending_validations.lock() else {
        return HttpResponse::InternalServerError().finish();
    };

    if !validation_map_lock.contains_key(render_id) {
        return HttpResponse::BadRequest().body("Invalid render_id parameter");
    }

    let validation = match validation_map_lock.remove(render_id) {
        Some(validation) => validation,
        None => return HttpResponse::BadRequest().body("How did this even happen?"),
    };

    if validation.is_expired() {
        return HttpResponse::Unauthorized().body("Validation attempt expired");
    }

    // don't await now so it can run while we clear the expired validations, which may or may not
    // take some time
    let db_async_query = persistence::set_user_not_bot(&state.db, validation.user);

    validation_map_lock.retain(|_, v| !v.is_expired());

    match db_async_query.await {
        Ok(_) => return HttpResponse::Ok().finish(),
        Err(_) => {
            validation_map_lock.insert(validation.unique_id.clone(), validation);
            return HttpResponse::InternalServerError().body("Error confirming the user state");
        }
    }
}

#[get("/api/isbot")]
async fn is_bot(state: web::Data<AppState<'_>>, req: HttpRequest) -> impl Responder {
    let ip_str = match req.connection_info().realip_remote_addr() {
        Some(ip) => ip.to_string(),
        None => return HttpResponse::InternalServerError().body("You broke it"),
    };

    let ip_dec = match conversion_utils::ip_to_u32(ip_str.to_string()) {
        Some(ip) => ip,
        None => return HttpResponse::InternalServerError().body("You broke it"),
    };

    match persistence::get_user_info(&state.db, ip_dec).await {
        Ok(info) => return HttpResponse::Ok().body(if info.is_bot { "YES" } else { "NO" }),
        Err(_) => return HttpResponse::InternalServerError().body("You broke it"),
    }
}

async fn fallback() -> impl Responder {
    return web::Redirect::to("/").see_other();
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
        pending_validations: Mutex::new(HashMap::new()),
    });

    let port: u16 = std::env::var("PORT")
        .unwrap_or("8080".to_owned())
        .parse()
        .unwrap();

    let res = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(actix_filesystem::Files::new("/static", "static/"))
            .service(bot_reply)
            .service(is_bot)
            .service(index)
            .default_service(web::route().to(fallback))
    })
    .bind(SocketAddr::from(([0, 0, 0, 0], port)))
    .unwrap()
    .run();

    println!("Started server on port: {}", port);

    return res.await;
}
