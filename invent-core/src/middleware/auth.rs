use axum::{
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Roles {
    Admin,
    Staff,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthClaims {
    #[serde(rename = "userId")]
    pub user_id: Uuid,
    #[serde(rename = "inventoryId")]
    pub inventory_id: Uuid,
    pub role: Roles,
    pub iat: Option<usize>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct UserContext {
    pub user_id: Uuid,
    pub inventory_id: Uuid,
    pub role: Roles,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub decoding_key: Arc<DecodingKey>,
    pub validation: Validation,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();

        AuthConfig {
            decoding_key: Arc::new(decoding_key),
            validation,
        }
    }
}

pub async fn jwt_middleware<B>(
    State(auth_config): State<AuthConfig>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, (StatusCode, &'static str)> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "missing Authorization header"))?;

    if !auth_header.starts_with("Bearer ") {
        return Err((StatusCode::UNAUTHORIZED, "No Bearer Scheme"));
    }

    let token = auth_header
        .split_whitespace()
        .nth(1)
        .ok_or((StatusCode::UNAUTHORIZED, "No Bearer token found"))?
        .trim_matches('"')
        .trim();

    let token_data =
        match decode::<AuthClaims>(token, &auth_config.decoding_key, &auth_config.validation) {
            Ok(td) => td,
            Err(_err) => {
                return Err((StatusCode::UNAUTHORIZED, "invalid token"));
            }
        };

    let auth_claim = token_data.claims;

    req.extensions_mut().insert(UserContext {
        user_id:auth_claim.user_id,
        inventory_id:auth_claim.inventory_id,
        role:auth_claim.role
    });

    Ok(next.run(req).await)
}
