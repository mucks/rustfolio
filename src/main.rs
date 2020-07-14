#[macro_use]
extern crate diesel;
#[macro_use]
extern crate anyhow;

mod schema;
mod user;

use actix_web::{middleware, web, App, HttpServer};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use std::env;

use self::user::user_api::*;

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
            .service(
                web::scope("/api")
                    .service(get_user)
                    .service(add_user)
                    .service(login_user)
                    .service(
                        web::scope("/auth")
                            .wrap(user::UserAuth {})
                            .service(test_user),
                    ),
            )
    })
    .bind(&bind)?
    .run()
    .await
}
