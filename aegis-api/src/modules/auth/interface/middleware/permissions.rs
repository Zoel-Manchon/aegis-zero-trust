
// HARDENING NOTE:
// This is currently path-prefix RBAC. It is simple and testable, but not enough
// for a mature Zero Trust system. Next step: move route -> permission mapping to
// a typed policy registry, require method-aware checks, deny-by-default unknown
// admin routes, and add ABAC conditions such as tenant_id/resource_owner_id.
use crate::modules::auth::models::user_model::UserRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    SelfLogout,
    SelfLogoutAll,

    AdminAccess,
    AuditRead,
    RiskRead,
    UserRead,
    UserDelete,
    SessionRevoke,
}

pub fn role_has_permission(role: &UserRole, permission: Permission) -> bool {
    match role {
        UserRole::Admin => true,

        UserRole::User => matches!(
            permission,
            Permission::SelfLogout | Permission::SelfLogoutAll
        ),
    }
}

pub fn required_permission_for_path(path: &str) -> Option<Permission> {
    if path == "/logout" {
        return Some(Permission::SelfLogout);
    }

    if path == "/logout-all" || path == "/logout_all" {
        return Some(Permission::SelfLogoutAll);
    }

    if path.starts_with("/admin") {
        return Some(Permission::AdminAccess);
    }

    if path.starts_with("/audit") {
        return Some(Permission::AuditRead);
    }

    if path.starts_with("/risk") {
        return Some(Permission::RiskRead);
    }

    None
}
