use crate::middleware::auth::{Roles, UserContext};

/// Permission guards for the product domain.
///
/// Each method encodes one business rule. Add new methods here as access
/// control requirements grow — handlers simply call into this module.
pub struct ProductPermission;

impl ProductPermission {
    pub fn can_create_product(user: &UserContext) -> bool {
        matches!(user.role, Roles::Admin)
    }

    pub fn can_update_product(user: &UserContext) -> bool {
        matches!(user.role, Roles::Admin | Roles::Staff)
    }

    pub fn can_delete_product(user: &UserContext) -> bool {
        matches!(user.role, Roles::Admin)
    }
}
