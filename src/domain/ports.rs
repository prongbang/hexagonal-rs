use async_trait::async_trait;
use super::{errors::DomainError, model::User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: User) -> Result<(), DomainError>;
    async fn get(&self, id: &str) -> Result<User, DomainError>;
}
