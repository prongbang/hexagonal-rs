use crate::{
    api,
    application::UserServiceImpl,
    domain::{Greeter, UserRepository},
    infrastructure::{
        circuit_breaker::default_breaker,
        diesel_db::{build_pool, Db},
        diesel_repo::DieselUserRepository,
        grpc_greeter::GrpcGreeter,
        in_memory_repo::InMemoryUserRepository,
    },
};
use std::sync::Arc;

fn build_greeter() -> Arc<dyn Greeter> {
    let greeter_addr =
        std::env::var("GREETER_ADDR").unwrap_or_else(|_| "http://localhost:50051".into());
    Arc::new(
        GrpcGreeter::connect_lazy(&greeter_addr, default_breaker()).expect("invalid GREETER_ADDR"),
    )
}

pub fn build_services() -> api::Services<UserServiceImpl<InMemoryUserRepository>> {
    build_services_with(Arc::new(InMemoryUserRepository::new()))
}

pub fn build_services_with<R: UserRepository + 'static>(
    repo: Arc<R>,
) -> api::Services<UserServiceImpl<R>> {
    api::Services {
        user: Arc::new(UserServiceImpl::new(repo)),
        greeter: build_greeter(),
    }
}

pub fn build_router() -> axum::Router {
    match std::env::var("DATABASE_URL") {
        Ok(url) => {
            // one Db handle; clone it into every future Diesel repository
            let db = Db::new(build_pool(&url).expect("diesel pool init failed"));
            api::router(build_services_with(Arc::new(DieselUserRepository::new(db))))
        }
        Err(_) => api::router(build_services()),
    }
}
