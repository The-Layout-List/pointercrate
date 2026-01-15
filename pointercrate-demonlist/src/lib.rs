use pointercrate_core::permission::{Permission, PermissionsManager};
use pointercrate_user::ADMINISTRATOR;

#[macro_use]
pub mod demon;
pub mod config;
pub mod creator;
pub mod error;
pub mod nationality;
pub mod player;
pub mod record;
pub mod submitter;
mod video;

pub const RELIABLE: Permission = Permission::new("user-permissions.reliable", 0x2);
pub const LIST_HELPER: Permission = Permission::new("user-permissions.list-helper", 0x4);
pub const LIST_MODERATOR: Permission = Permission::new("user-permissions.list-moderator", 0x8);
pub const LIST_ADMINISTRATOR: Permission = Permission::new("user-permissions.list-administrator", 0x10);

pub fn default_permissions_manager() -> PermissionsManager {
    PermissionsManager::new(vec![ADMINISTRATOR, RELIABLE, LIST_HELPER, LIST_MODERATOR, LIST_ADMINISTRATOR])
        .assigns(ADMINISTRATOR, LIST_ADMINISTRATOR)
        .assigns(ADMINISTRATOR, LIST_MODERATOR)
        .assigns(ADMINISTRATOR, LIST_HELPER)
        .assigns(ADMINISTRATOR, RELIABLE)
        .assigns(LIST_ADMINISTRATOR, LIST_MODERATOR)
        .assigns(LIST_ADMINISTRATOR, LIST_HELPER)
        .implies(LIST_ADMINISTRATOR, LIST_MODERATOR)
        .implies(LIST_MODERATOR, LIST_HELPER)
        .implies(LIST_HELPER, RELIABLE)
}
