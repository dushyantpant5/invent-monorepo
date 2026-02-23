#![allow(dead_code)]

use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use db::{
    entities::Product,
    repos::{CreateProduct, ProductRepository, UpdateProduct},
    repository::{Page, Pagination, Repository},
    DbError, DbResult,
};

#[derive(Clone)]
pub struct ProductService {
    repo: Arc<ProductRepository>,
}

impl ProductService {
    pub fn new(repo: Arc<ProductRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_product(
        &self,
        user_id: Uuid,
        category_id: Option<Uuid>,
        name: String,
        description: Option<String>,
        price: Decimal,
        quantity: i32,
    ) -> DbResult<Product> {
        if name.trim().is_empty() {
            return Err(DbError::InvalidInput("name is required".into()));
        }
        if price < Decimal::ZERO {
            return Err(DbError::InvalidInput("price must be >= 0".into()));
        }
        if quantity < 0 {
            return Err(DbError::InvalidInput("quantity must be >= 0".into()));
        }

        self.repo
            .create(CreateProduct {
                user_id,
                category_id,
                name: name.trim().to_string(),
                description,
                price,
                quantity,
            })
            .await
    }

    pub async fn get_product(&self, id: Uuid) -> DbResult<Product> {
        self.repo.find_by_id(id).await
    }

    pub async fn list_products(&self, pagination: Pagination) -> DbResult<Page<Product>> {
        self.repo.find_all(pagination).await
    }

    pub async fn list_by_user(
        &self,
        user_id: Uuid,
        pagination: Pagination,
    ) -> DbResult<Page<Product>> {
        self.repo.find_by_user(user_id, pagination).await
    }

    pub async fn update_product(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        price: Option<Decimal>,
        quantity: Option<i32>,
        category_id: Option<Uuid>,
    ) -> DbResult<Product> {
        if let Some(ref n) = name {
            if n.trim().is_empty() {
                return Err(DbError::InvalidInput("name cannot be empty".into()));
            }
        }

        if let Some(p) = price {
            if p < Decimal::ZERO {
                return Err(DbError::InvalidInput("price must be >= 0".into()));
            }
        }

        self.repo
            .update(
                id,
                UpdateProduct {
                    category_id,
                    name: name.map(|n| n.trim().to_string()),
                    description,
                    price,
                    quantity,
                },
            )
            .await
    }

    pub async fn delete_product(&self, id: Uuid) -> DbResult<()> {
        self.repo.delete(id).await
    }
}
