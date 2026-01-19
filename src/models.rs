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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepsSummaryResponse {
    pub daily: i64,
    pub monthly: i64,
    pub yearly: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterSummaryResponse {
    pub daily_ml: i64,
    pub monthly_ml: i64,
    pub yearly_ml: i64,
    pub daily_display: String,
    pub monthly_display: String,
    pub yearly_display: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepResponse {
    pub daily_minutes: i64,
    pub monthly_minutes: i64,
    pub yearly_minutes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

