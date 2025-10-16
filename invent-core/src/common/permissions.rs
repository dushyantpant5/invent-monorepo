use crate::middleware::auth::{Roles, UserContext};
// Product related Permissions
#[allow(dead_code)]
pub struct ProductPermission {}
#[allow(dead_code)]
impl ProductPermission{
    pub fn can_create_product(user: &UserContext) -> bool {
        matches!(user.role , Roles::Admin)
    }
}