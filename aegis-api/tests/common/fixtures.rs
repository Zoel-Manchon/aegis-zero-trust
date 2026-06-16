pub const PASSWORD: &str = "StrongPassword123!";

pub fn unique_email(prefix: &str) -> String {
    format!("{}-{}@example.com", prefix, uuid::Uuid::new_v4())
}
