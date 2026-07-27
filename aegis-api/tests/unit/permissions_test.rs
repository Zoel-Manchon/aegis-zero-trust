use aegis::modules::auth::{
    interface::middleware::permissions::{
        Permission, required_permission_for_path, role_has_permission,
    },
    models::user_model::UserRole,
};

#[test]
fn user_can_logout_self() {
    assert!(role_has_permission(&UserRole::User, Permission::SelfLogout,));
}

#[test]
fn user_cannot_access_admin() {
    assert!(!role_has_permission(
        &UserRole::User,
        Permission::AdminAccess,
    ));
}

#[test]
fn admin_can_access_admin() {
    assert!(role_has_permission(
        &UserRole::Admin,
        Permission::AdminAccess,
    ));
}

#[test]
fn admin_dashboard_requires_admin_access() {
    assert_eq!(
        required_permission_for_path("/admin/dashboard"),
        Some(Permission::AdminAccess),
    );
}

#[test]
fn logout_requires_self_logout() {
    assert_eq!(
        required_permission_for_path("/logout"),
        Some(Permission::SelfLogout),
    );
}
