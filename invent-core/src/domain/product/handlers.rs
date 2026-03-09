use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use db::repository::Pagination;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::product::{
        dto::{CreateProductRequest, ProductResponse, UpdateProductRequest},
        service::ProductService,
    },
    extractors::AuthUser,
    shared::{
        errors::AppError,
        permissions::ProductPermission,
        response::{created, no_content, ok, ApiResponse, PagedData},
    },
};

// ── Pagination query params ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page() -> u64 {
    1
}
fn default_page_size() -> u64 {
    20
}

impl From<PaginationQuery> for Pagination {
    fn from(q: PaginationQuery) -> Self {
        Pagination::new(q.page, q.page_size)
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────
//
// Each handler follows the same three-step pattern:
//   1. Authorise   — check the caller has permission
//   2. Validate    — call dto.validate() before touching the service
//   3. Delegate    — call the service, map the result to a response
//
// Handlers never contain business logic. All decisions live in the service.

/// POST /api/v1/product
pub async fn create_product(
    State(service): State<Arc<dyn ProductService>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ProductResponse>>), AppError> {
    if !ProductPermission::can_create_product(&user) {
        return Err(AppError::Forbidden(
            "you are not allowed to create products",
        ));
    }
    payload.validate()?;
    let product = service.create_product(user.user_id, payload).await?;
    Ok(created(ProductResponse::from(product)))
}

/// GET /api/v1/product/:id
pub async fn get_product(
    State(service): State<Arc<dyn ProductService>>,
    AuthUser(_): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProductResponse>>, AppError> {
    let product = service.get_product(id).await?;
    Ok(ok(ProductResponse::from(product)))
}

/// GET /api/v1/product/my
pub async fn list_my_products(
    State(service): State<Arc<dyn ProductService>>,
    AuthUser(user): AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<PagedData<ProductResponse>>>, AppError> {
    let page = service
        .list_by_user(user.user_id, pagination.into())
        .await?;
    let data = PagedData::new(
        page.items.into_iter().map(ProductResponse::from).collect(),
        page.total,
        page.limit,
        page.offset,
    );
    Ok(ok(data))
}

/// PUT /api/v1/product/:id
pub async fn update_product(
    State(service): State<Arc<dyn ProductService>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Json<ApiResponse<ProductResponse>>, AppError> {
    if !ProductPermission::can_update_product(&user) {
        return Err(AppError::Forbidden(
            "you are not allowed to update products",
        ));
    }
    payload.validate()?;
    let product = service.update_product(id, payload).await?;
    Ok(ok(ProductResponse::from(product)))
}

/// DELETE /api/v1/product/:id
pub async fn delete_product(
    State(service): State<Arc<dyn ProductService>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if !ProductPermission::can_delete_product(&user) {
        return Err(AppError::Forbidden(
            "you are not allowed to delete products",
        ));
    }
    service.delete_product(id).await?;
    Ok(no_content())
}
