use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::SystemTime;
use uuid::Uuid;

use super::user_models::*;

pub fn decrypt_token(token_str: &str) -> Result<String, anyhow::Error> {
    let secret = std::env::var("SECRET").expect("SECRET env not set");

    let token = decode::<Claims>(
        &token_str,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;
    Ok(token.claims.user_id)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn find_user_by_uid(
    uid: Uuid,
    conn: &PgConnection,
) -> Result<Option<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let user = users
        .filter(id.eq(uid.to_string()))
        .first::<User>(conn)
        .optional()?;

    Ok(user)
}

pub fn update_last_login(uid: String, conn: &PgConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(uid)))
        .set(last_login.eq(std::time::SystemTime::now()))
        .execute(conn)?;
    Ok(())
}

pub fn try_login_user(
    login_user: &LoginUser,
    conn: &PgConnection,
) -> Result<LoginResponse, anyhow::Error> {
    let user = find_user_by_email(&login_user.email, conn)?;
    let valid = verify(login_user.password.clone(), &user.password)?;

    if valid {
        let now = SystemTime::now().elapsed()?.as_millis();
        let day_millis = 3600_000 * 24;
        let tomorrow = now + day_millis;

        let sub = std::env::var("SUB").expect("SUB env not set");
        let company = std::env::var("COMPANY").expect("COMPANY env not set");

        let claims = Claims {
            company,
            sub,
            exp: tomorrow as usize,
            user_id: user.id.clone(),
        };

        let secret = std::env::var("SECRET").expect("SECRET env not set");

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )?;

        println!("user {} logged in succesfully", user.email);
        update_last_login(user.id, conn)?;
        Ok(LoginResponse { token })
    } else {
        Err(anyhow!("Password invalid"))
    }
}

pub fn find_user_by_email(
    user_email: &str,
    conn: &PgConnection,
) -> Result<User, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let user = users.filter(email.eq(user_email)).first::<User>(conn)?;

    Ok(user)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_user(
    // prevent collision with `name` column imported inside the function
    new_user: &NewUser,
    conn: &PgConnection,
) -> Result<User, diesel::result::Error> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::users::dsl::*;

    let hashed = hash(&new_user.password, DEFAULT_COST).unwrap();

    let user = User {
        id: Uuid::new_v4().to_string(),
        username: new_user.username.clone(),
        email: new_user.email.clone(),
        password: hashed,
        created_on: std::time::SystemTime::now(),
        last_login: None,
    };

    diesel::insert_into(users).values(&user).execute(conn)?;

    Ok(user)
}
