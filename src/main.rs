#[macro_use]
extern crate diesel;
#[macro_use]
extern crate anyhow;

use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};

use actix_web::{get, middleware, post, web, App, Error, HttpResponse, HttpServer};
use std::env;

mod models;
mod schema;

mod actions;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/user/{user_id}")]
async fn get_user(
    pool: web::Data<DbPool>,
    user_uid: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, Error> {
    let user_uid = user_uid.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");

    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || actions::find_user_by_uid(user_uid, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let res = HttpResponse::NotFound().body(format!("No user found with uid: {}", user_uid));
        Ok(res)
    }
}

#[post("/login")]
async fn login_user(
    pool: web::Data<DbPool>,
    form: web::Json<models::LoginUser>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let resp = web::block(move || actions::login_user(&form, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(resp))
}

#[post("/user")]
async fn add_user(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || actions::insert_new_user(&form, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(user))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env needs to be set");
    let _secret = std::env::var("SECRET").expect("SECRET env needs to be set");
    let _sub = std::env::var("SUB").expect("SUB env not set");
    let _company = std::env::var("COMPANY").expect("COMPANY env not set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let bind = "127.0.0.1:8080";

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(get_user)
            .service(add_user)
            .service(login_user)
    })
    .bind(&bind)?
    .run()
    .await
}
