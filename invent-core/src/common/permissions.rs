use crate::middleware::auth::{Roles, UserContext};

pub struct ProductPermission;

impl ProductPermission {
    pub fn can_create_product(user: &UserContext) -> bool {
        matches!(user.role, Roles::Admin)
    }
}
