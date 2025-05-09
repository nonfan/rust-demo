use crate::models::{User, NewUser};
use crate::schema::users::dsl::*;
use actix_web::{
    HttpResponse, Responder, Result, delete, error, get, post, put, web,
};
use serde_json::json;
use diesel::prelude::*;
use crate::DbPool;


#[get("/users")]
pub async fn list_users(pool: web::Data<DbPool>) -> Result<impl Responder> {
    let mut conn = pool.get().map_err(|e| error::ErrorInternalServerError(e))?;

    let results = web::block(move || users.load::<User>(&mut conn))
    .await?
    .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(results))
}

#[get("/users/{id}")]
pub async fn get_user(pool: web::Data<DbPool>, path: web::Path<i32>) -> Result<impl Responder> {
    let user_id = path.into_inner();

    let mut conn = pool.get().map_err(|e| error::ErrorInternalServerError(e))?;

    let result = web::block(move || users.filter(id.eq(user_id)).first::<User>(&mut conn))
    .await?;

    match result {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(diesel::result::Error::NotFound) => Err(error::ErrorNotFound("用户不存在")),
        Err(e) => Err(error::ErrorInternalServerError(e)),
    }}

#[post("/users")]
pub async fn create_user(pool: web::Data<DbPool>, body: web::Json<NewUser>) -> Result<impl Responder> {
    let new_user = body.into_inner();

    if new_user.remark.is_empty() || new_user.username.is_empty() {
        return Err(error::ErrorBadRequest("参数异常"))
    }

    let mut conn = pool.get().map_err(|e| error::ErrorInternalServerError(e))?;

    let result = web::block(move || {
        diesel::insert_into(users)
        .values(new_user)
        .returning(User::as_returning())
        .get_result(&mut conn)
    })
    .await?
    .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(result))
}

#[put("/users/{id}")]
pub async fn update_user(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    body: web::Json<NewUser>,
) -> Result<impl Responder> {
    let user_id = path.into_inner();
    let new_user = body.into_inner();

    let mut conn = pool.get().map_err(|e| error::ErrorInternalServerError(e))?;

    let result = web::block(move || {
        diesel::update(users)
        .filter(id.eq(user_id))
        .set((username.eq(new_user.username), remark.eq(new_user.remark)))
        .returning(User::as_returning())
        .get_result(&mut conn)
    })
    .await
    .map_err(|e| error::ErrorInternalServerError(e))?;

    match result {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(diesel::NotFound) => Ok(HttpResponse::NotFound().json(json!({ "error": "用户不存在" }))),
        Err(e) => Err(error::ErrorInternalServerError(e)),
    }
}

#[delete("/users/{id}")]
pub async fn delete_user(pool: web::Data<DbPool>, path: web::Path<i32>) -> Result<impl Responder> {
    let user_id = path.into_inner();
    let mut conn = pool.get().map_err(error::ErrorInternalServerError)?;

    let result = web::block(move || {
        diesel::delete(users.filter(id.eq(user_id)))
        .returning(User::as_returning())
        .get_result(&mut conn)
    }).await
    .map_err(|e| error::ErrorInternalServerError(e))?;

    match result {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(diesel::NotFound) => Ok(HttpResponse::NotFound().json(json!({ "error": "用户不存在" }))),
        Err(e) => Err(error::ErrorInternalServerError(e)),
    }
}