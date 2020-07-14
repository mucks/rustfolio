use actix_web::{get, post, web, Error, HttpRequest, HttpResponse};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};

use super::user_actions::*;
use super::user_models::*;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/test-user")]
async fn test_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    match req.headers().get("user-id") {
        Some(header) => println!("header found {:?}", header),
        None => println!("header not found"),
    }
    Ok(HttpResponse::Ok().json("{}"))
}

#[get("user/{user_id}")]
async fn get_user(
    pool: web::Data<DbPool>,
    user_uid: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, Error> {
    let user_uid = user_uid.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");

    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || find_user_by_uid(user_uid, &conn))
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
    form: web::Json<LoginUser>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let resp = web::block(move || try_login_user(&form, &conn))
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
    form: web::Json<NewUser>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    println!("adding user");

    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || insert_new_user(&form, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(user))
}
