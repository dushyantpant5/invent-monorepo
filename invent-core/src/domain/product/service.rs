use async_trait::async_trait;
use db::{
    entities::Product,
    repos::{CreateProduct, ProductRepository, UpdateProduct},
    repository::{Page, Pagination, Repository},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::product::dto::{CreateProductRequest, UpdateProductRequest},
    shared::AppError,
};

// ── Repository interface (Dependency Inversion) ────────────────────────────
//
// The service declares the data access contract it needs.
// The concrete `ProductRepository` from the `db` crate is adapted to this
// interface below, keeping the service fully decoupled from infrastructure.

#[async_trait]
pub trait ProductRepo: Send + Sync + 'static {
    async fn find_by_id(&self, id: Uuid) -> Result<Product, AppError>;
    async fn find_by_user(
        &self,
        user_id: Uuid,
        pagination: Pagination,
    ) -> Result<Page<Product>, AppError>;
    async fn create(&self, payload: CreateProduct) -> Result<Product, AppError>;
    async fn update(&self, id: Uuid, payload: UpdateProduct) -> Result<Product, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

// ── Infrastructure adapter ─────────────────────────────────────────────────
//
// Bridges the `db` crate's concrete repository to our domain interface.
// Lives here (in `invent-core`) so neither the domain nor the db crate
// depend on each other beyond what is strictly necessary.

#[async_trait]
impl ProductRepo for ProductRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Product, AppError> {
        <ProductRepository as Repository<Product, CreateProduct, UpdateProduct>>::find_by_id(
            self, id,
        )
        .await
        .map_err(AppError::from)
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
        pagination: Pagination,
    ) -> Result<Page<Product>, AppError> {
        ProductRepository::find_by_user(self, user_id, pagination)
            .await
            .map_err(AppError::from)
    }

    async fn create(&self, payload: CreateProduct) -> Result<Product, AppError> {
        <ProductRepository as Repository<Product, CreateProduct, UpdateProduct>>::create(
            self, payload,
        )
        .await
        .map_err(AppError::from)
    }

    async fn update(&self, id: Uuid, payload: UpdateProduct) -> Result<Product, AppError> {
        <ProductRepository as Repository<Product, CreateProduct, UpdateProduct>>::update(
            self, id, payload,
        )
        .await
        .map_err(AppError::from)
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        <ProductRepository as Repository<Product, CreateProduct, UpdateProduct>>::delete(self, id)
            .await
            .map_err(AppError::from)
    }
}

// ── Service interface (what handlers depend on) ────────────────────────────
//
// Handlers depend only on this trait, never on the concrete implementation.
// This makes handlers trivially testable and keeps HTTP logic separated from
// business logic.

#[async_trait]
pub trait ProductService: Send + Sync + 'static {
    async fn create_product(
        &self,
        user_id: Uuid,
        req: CreateProductRequest,
    ) -> Result<Product, AppError>;

    async fn get_product(&self, id: Uuid) -> Result<Product, AppError>;

    async fn list_by_user(
        &self,
        user_id: Uuid,
        pagination: Pagination,
    ) -> Result<Page<Product>, AppError>;

    async fn update_product(
        &self,
        id: Uuid,
        req: UpdateProductRequest,
    ) -> Result<Product, AppError>;

    async fn delete_product(&self, id: Uuid) -> Result<(), AppError>;
}

// ── Concrete implementation ────────────────────────────────────────────────

pub struct ProductServiceImpl {
    repo: Arc<dyn ProductRepo>,
}

impl ProductServiceImpl {
    pub fn new(repo: Arc<dyn ProductRepo>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ProductService for ProductServiceImpl {
    async fn create_product(
        &self,
        user_id: Uuid,
        req: CreateProductRequest,
    ) -> Result<Product, AppError> {
        self.repo
            .create(CreateProduct {
                user_id,
                category_id: req.category_id,
                name: req.name.trim().to_string(),
                description: req.description,
                price: req.price,
                quantity: req.quantity,
            })
            .await
    }

    async fn get_product(&self, id: Uuid) -> Result<Product, AppError> {
        self.repo.find_by_id(id).await
    }

    async fn list_by_user(
        &self,
        user_id: Uuid,
        pagination: Pagination,
    ) -> Result<Page<Product>, AppError> {
        self.repo.find_by_user(user_id, pagination).await
    }

    async fn update_product(
        &self,
        id: Uuid,
        req: UpdateProductRequest,
    ) -> Result<Product, AppError> {
        self.repo
            .update(
                id,
                UpdateProduct {
                    category_id: req.category_id,
                    name: req.name.map(|n| n.trim().to_string()),
                    description: req.description,
                    price: req.price,
                    quantity: req.quantity,
                },
            )
            .await
    }

    async fn delete_product(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id).await
    }
}
