use std::fmt::Display;

use serde::Deserialize;

#[derive(thiserror::Error, Debug, Deserialize)]
#[serde(untagged)]
pub enum TastyApiResponse<T> {
    Success(Response<T>),
    Error { error: ApiError },
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub data: T,
    pub context: String,
}

#[derive(Debug, Deserialize)]
pub struct Items<T> {
    pub items: Vec<T>,
}

#[derive(thiserror::Error, Debug, Deserialize)]
pub struct ApiError {
    pub code: Option<String>,
    pub message: String,
    pub errors: Option<Vec<InnerApiError>>,
}

#[derive(Debug, Deserialize)]
pub struct InnerApiError {
    pub code: Option<String>,
    pub message: String,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {:?}: {}", self.code, self.message)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TastyError {
    #[error("Tastyworks API error")]
    Api(#[from] ApiError),
    #[error("HTTP Error")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON Error")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TastyError>;
