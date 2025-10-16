use crate::middleware::auth::{Roles, UserContext};
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};

#[derive(Clone, Debug)]

pub struct AuthUser(pub UserContext);

#[allow(dead_code)]
impl AuthUser {
    pub fn is_admin(&self) -> bool {
        matches!(self.0.role, Roles::Admin)
    }

    pub fn is_staff(&self) -> bool {
        matches!(self.0.role, Roles::Staff)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<UserContext>()
            .cloned()
            .map(AuthUser)
            .ok_or((StatusCode::UNAUTHORIZED, "User context missing"))
    }
}
