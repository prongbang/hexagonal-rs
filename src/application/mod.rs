use std::sync::Arc;
use async_trait::async_trait;
use crate::domain::{User, UserRepository, DomainError};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn create_user(&self, id: String, name: String) -> Result<(), DomainError>;
    async fn get_user(&self, id: String) -> Result<User, DomainError>;
}

pub struct UserServiceImpl<R: UserRepository> {
    repo: Arc<R>,
}

impl<R: UserRepository> UserServiceImpl<R> {
    pub fn new(repo: Arc<R>) -> Self { Self { repo } }
}

#[async_trait]
impl<R: UserRepository> UserService for UserServiceImpl<R> {
    async fn create_user(&self, id: String, name: String) -> Result<(), DomainError> {
        let user = User::new(id, name)?;
        self.repo.save(user).await
    }

    async fn get_user(&self, id: String) -> Result<User, DomainError> {
        self.repo.get(&id).await
    }
}
