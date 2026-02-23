use async_trait::async_trait;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    entities::{
        product::{ActiveModel, Column, Entity},
        Product,
    },
    error::{DbError, DbResult},
    repository::{Page, Pagination, Repository},
};

// ── DTOs ──────────────────────────────────────────────────────────────────────

pub struct CreateProduct {
    pub user_id: Uuid,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub quantity: i32,
}

pub struct UpdateProduct {
    pub category_id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub quantity: Option<i32>,
}

// ── Repository ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ProductRepository {
    db: DatabaseConnection,
}

impl ProductRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ── Domain-specific queries ───────────────────────────────────────────

    /// All products belonging to a specific user (inventory owner).
    pub async fn find_by_user(
        &self,
        user_id: Uuid,
        pagination: Pagination,
    ) -> DbResult<Page<Product>> {
        let total = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .count(&self.db)
            .await
            .map_err(DbError::from_sea)?;

        let items = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .offset(Some(pagination.offset))
            .limit(Some(pagination.limit))
            .all(&self.db)
            .await
            .map_err(DbError::from_sea)?;

        Ok(Page {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Products within a specific category.
    pub async fn find_by_category(
        &self,
        category_id: Uuid,
        pagination: Pagination,
    ) -> DbResult<Page<Product>> {
        let total = Entity::find()
            .filter(Column::CategoryId.eq(category_id))
            .count(&self.db)
            .await
            .map_err(DbError::from_sea)?;

        let items = Entity::find()
            .filter(Column::CategoryId.eq(category_id))
            .order_by_desc(Column::CreatedAt)
            .offset(Some(pagination.offset))
            .limit(Some(pagination.limit))
            .all(&self.db)
            .await
            .map_err(DbError::from_sea)?;

        Ok(Page {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Products at or below a stock threshold — used for inventory alerts.
    pub async fn find_low_stock(&self, threshold: i32) -> DbResult<Vec<Product>> {
        Entity::find()
            .filter(Column::Quantity.lte(threshold))
            .order_by_asc(Column::Quantity)
            .all(&self.db)
            .await
            .map_err(DbError::from_sea)
    }

    /// Case-insensitive name search.
    pub async fn search_by_name(
        &self,
        query: &str,
        pagination: Pagination,
    ) -> DbResult<Page<Product>> {
        use sea_orm::Condition;
        let pattern = format!("%{}%", query.to_lowercase());

        let condition = Condition::any().add(Column::Name.contains(&pattern));

        let total = Entity::find()
            .filter(condition.clone())
            .count(&self.db)
            .await
            .map_err(DbError::from_sea)?;

        let items = Entity::find()
            .filter(condition)
            .order_by_asc(Column::Name)
            .offset(Some(pagination.offset))
            .limit(Some(pagination.limit))
            .all(&self.db)
            .await
            .map_err(DbError::from_sea)?;

        Ok(Page {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }
}

// ── Base Repository trait implementation ──────────────────────────────────────

#[async_trait]
impl Repository<Product, CreateProduct, UpdateProduct> for ProductRepository {
    async fn find_by_id(&self, id: Uuid) -> DbResult<Product> {
        Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(DbError::from_sea)?
            .ok_or(DbError::NotFound)
    }

    async fn find_all(&self, pagination: Pagination) -> DbResult<Page<Product>> {
        let total = Entity::find()
            .count(&self.db)
            .await
            .map_err(DbError::from_sea)?;

        let items = Entity::find()
            .order_by_desc(Column::CreatedAt)
            .offset(Some(pagination.offset))
            .limit(Some(pagination.limit))
            .all(&self.db)
            .await
            .map_err(DbError::from_sea)?;

        Ok(Page {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    async fn create(&self, payload: CreateProduct) -> DbResult<Product> {
        let model = ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(payload.user_id),
            category_id: Set(payload.category_id),
            name: Set(payload.name),
            description: Set(payload.description),
            price: Set(payload.price),
            quantity: Set(payload.quantity),
            created_at: Set(chrono::Utc::now()),
        };

        model.insert(&self.db).await.map_err(DbError::from_sea)
    }

    async fn update(&self, id: Uuid, payload: UpdateProduct) -> DbResult<Product> {
        // Fetch first, then apply only the fields that were provided
        let existing = Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(DbError::from_sea)?
            .ok_or(DbError::NotFound)?;

        let mut model: ActiveModel = existing.into();

        if let Some(v) = payload.category_id {
            model.category_id = Set(Some(v));
        }
        if let Some(v) = payload.name {
            model.name = Set(v);
        }
        if let Some(v) = payload.description {
            model.description = Set(Some(v));
        }
        if let Some(v) = payload.price {
            model.price = Set(v);
        }
        if let Some(v) = payload.quantity {
            model.quantity = Set(v);
        }

        model.update(&self.db).await.map_err(DbError::from_sea)
    }

    async fn delete(&self, id: Uuid) -> DbResult<()> {
        let result = Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(DbError::from_sea)?;

        if result.rows_affected == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    async fn exists(&self, id: Uuid) -> DbResult<bool> {
        let count = Entity::find_by_id(id)
            .count(&self.db)
            .await
            .map_err(DbError::from_sea)?;
        Ok(count > 0)
    }
}
