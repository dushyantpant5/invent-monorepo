use axum::{http::StatusCode, Json};
use serde::Serialize;

/// Standard envelope for every API response.
///
/// Success:  `{ "success": true,  "data": <T> }`
/// Error:    handled by `AppError::into_response` (no envelope needed)
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

/// Paginated data payload — used as the `data` field inside `ApiResponse`.
#[derive(Serialize)]
pub struct PagedData<T: Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

impl<T: Serialize> PagedData<T> {
    pub fn new(items: Vec<T>, total: u64, limit: u64, offset: u64) -> Self {
        Self {
            items,
            total,
            limit,
            offset,
        }
    }
}

/// 200 OK with a JSON envelope.
pub fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data,
    })
}

/// 201 Created with a JSON envelope.
pub fn created<T: Serialize>(data: T) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data,
        }),
    )
}

/// 204 No Content.
pub fn no_content() -> StatusCode {
    StatusCode::NO_CONTENT
}
