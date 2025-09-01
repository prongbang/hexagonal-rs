use crate::{
    api, application::UserServiceImpl, infrastructure::in_memory_repo::InMemoryUserRepository,
};
use std::sync::Arc;

pub fn build_services() -> api::Services<UserServiceImpl<InMemoryUserRepository>> {
    let repo = Arc::new(InMemoryUserRepository::new());
    let user_svc = Arc::new(UserServiceImpl::new(repo));
    api::Services { user: user_svc }
}

pub fn build_router() -> axum::Router {
    let services = build_services();
    api::router(services)
}

#[allow(dead_code)]
pub fn build_router_with_state() -> axum::Router {
    let services = build_services();
    api::router_without_state().with_state(api::AppState {
        svc: Arc::new(services),
    })
}
