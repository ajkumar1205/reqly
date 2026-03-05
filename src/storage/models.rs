use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlRecord {
    pub id: i64,
    pub url: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRecord {
    pub id: i64,
    pub url_id: i64,
    pub protocol: String,
    pub method: String,
    pub headers: String,
    pub request_body: String,
    pub response_body: String,
    pub status_code: i64,
    pub duration_ms: i64,
    pub created_at: i64,
}
