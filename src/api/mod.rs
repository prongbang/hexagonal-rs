use crate::{
    application::UserService,
    domain::{DomainError, User},
};
use axum::{extract::FromRef, http::StatusCode};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, sync::Arc};

#[derive(Debug)]
pub struct Services<U> {
    pub user: Arc<U>,
}

#[derive(Debug)]
pub struct AppState<S> {
    pub svc: Arc<S>,
}

#[derive(Deserialize)]
pub struct CreateUserReq {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct ErrorRes {
    pub error: String,
}

// Manual clone
impl<U> Clone for Services<U> {
    fn clone(&self) -> Self {
        Self {
            user: self.user.clone(),
        }
    }
}

impl<S> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            svc: self.svc.clone(),
        }
    }
}

#[derive(Clone)]
pub struct UserSvc<U>(pub Arc<U>);

impl<U> Deref for UserSvc<U> {
    type Target = U;
    fn deref(&self) -> &U {
        &self.0
    }
}

impl<U> FromRef<AppState<Services<U>>> for UserSvc<U> {
    fn from_ref(s: &AppState<Services<U>>) -> Self {
        UserSvc(s.svc.user.clone())
    }
}

#[allow(dead_code)]
pub fn router_without_state<U>() -> axum::Router<AppState<Services<U>>>
where
    U: UserService + Sync + Send + 'static,
{
    axum::Router::new()
        .route("/health", get(health))
        .route("/users/{id}", get(get_user::<U>))
        .route("/users", post(create_user::<U>))
}

pub fn router<U>(services: Services<U>) -> axum::Router
where
    U: UserService + Sync + Send + 'static,
{
    let state = AppState {
        svc: Arc::new(services),
    };

    axum::Router::new()
        .route("/health", get(health))
        .route("/users/{id}", get(get_user::<U>))
        .route("/users", post(create_user::<U>))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

pub async fn create_user<U>(
    State(user_svc): State<UserSvc<U>>,
    Json(req): Json<CreateUserReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorRes>)>
where
    U: UserService + Send + Sync + 'static,
{
    user_svc
        .create_user(req.id, req.name)
        .await
        .map_err(to_http_err)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_user<U>(
    State(user_svc): State<UserSvc<U>>,
    Path(id): Path<String>,
) -> Result<Json<User>, (StatusCode, Json<ErrorRes>)>
where
    U: UserService + Send + Sync + 'static,
{
    let user = user_svc.get_user(id).await.map_err(to_http_err)?;
    Ok(Json(user))
}

fn to_http_err(e: DomainError) -> (StatusCode, Json<ErrorRes>) {
    let (code, msg) = match e {
        DomainError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
        DomainError::Validation(m) => (StatusCode::BAD_REQUEST, m),
        DomainError::Other(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into()),
    };
    (code, Json(ErrorRes { error: msg }))
}
