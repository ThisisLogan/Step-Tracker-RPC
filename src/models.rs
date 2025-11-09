use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepsRequest {
    pub steps: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepsResponse {
    pub steps: i64,
    pub date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepsSummaryResponse {
    pub daily: i64,
    pub monthly: i64,
    pub yearly: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

