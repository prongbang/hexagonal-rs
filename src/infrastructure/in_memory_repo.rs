use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use crate::domain::{User, UserRepository, DomainError};

#[derive(Default)]
pub struct InMemoryUserRepository {
    inner: Arc<RwLock<HashMap<String, User>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self { Self::default() }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn save(&self, user: User) -> Result<(), DomainError> {
        self.inner.write().await.insert(user.id.clone(), user);
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<User, DomainError> {
        self.inner.read().await
            .get(id)
            .cloned()
            .ok_or(DomainError::NotFound)
    }
}
