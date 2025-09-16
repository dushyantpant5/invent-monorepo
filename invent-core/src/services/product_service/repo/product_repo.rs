use axum::async_trait;
use axum::extract::{Extension, FromRequestParts};
use axum::http::request::Parts;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::services::product_service::model::product::Product;

#[derive(Clone)]
pub struct ProductRepository {
    pool: Arc<PgPool>,
}

impl ProductRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_product(
        &self,
        user_id: Uuid,
        category_id: Option<Uuid>,
        name: &str,
        description: Option<&str>,
        price: f64,
        quantity: i32,
    ) -> Result<Product, sqlx::Error> {
        let rec = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (user_id, category_id, name, description, price, quantity, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, now())
            RETURNING id, user_id, category_id, name, description, price, quantity, created_at
            "#,
        )
        .bind(user_id)
        .bind(category_id)
        .bind(name)
        .bind(description)
        .bind(price)
        .bind(quantity)
        .fetch_one(&*self.pool)
        .await?;

        Ok(rec)
    }
}

// region: Repository Extractor for Axum
pub struct RepoExtractor(pub ProductRepository);

#[async_trait]
impl<S> FromRequestParts<S> for RepoExtractor
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract Arc<PgPool> from request extensions (set in main.rs)
        let Extension(pool): Extension<Arc<PgPool>> = Extension::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "missing db pool",
                )
            })?;
        Ok(RepoExtractor(ProductRepository::new(pool)))
    }
}
