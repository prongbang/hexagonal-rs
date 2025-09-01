use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
}

impl User {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Result<Self, crate::domain::errors::DomainError> {
        let id = id.into();
        let name = name.into();
        if name.trim().is_empty() {
            return Err(crate::domain::errors::DomainError::Validation("name is empty".into()));
        }
        Ok(Self { id, name })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_user_rejects_empty_name() {
        let res = User::new("u1", "");
        assert!(res.is_err());
    }
}
