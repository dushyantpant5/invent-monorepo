use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub quantity: i64,
}
