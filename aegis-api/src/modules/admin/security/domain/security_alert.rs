use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SecurityAlert {
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub count: i64,
}
