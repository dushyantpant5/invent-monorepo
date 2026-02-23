use async_trait::async_trait;
use uuid::Uuid;
use crate::error::DbResult;

#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: u64,
    pub offset: u64,
}

impl Pagination {
    pub fn new(page: u64, page_size: u64) -> Self {
        Self { limit: page_size, offset: (page - 1) * page_size }
    }
}

impl Default for Pagination {
    fn default() -> Self { Self { limit: 20, offset: 0 } }
}

#[derive(Debug)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

#[async_trait]
pub trait Repository<Model, Create, Update>: Send + Sync
where
    Model: Send + Sync,
    Create: Send + Sync,
    Update: Send + Sync,
{
    async fn find_by_id(&self, id: Uuid) -> DbResult<Model>;
    async fn find_all(&self, pagination: Pagination) -> DbResult<Page<Model>>;
    async fn create(&self, payload: Create) -> DbResult<Model>;
    async fn update(&self, id: Uuid, payload: Update) -> DbResult<Model>;
    async fn delete(&self, id: Uuid) -> DbResult<()>;
    async fn exists(&self, id: Uuid) -> DbResult<bool>;
}
