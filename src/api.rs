use serde::{Deserialize, Serialize};

// Standardized API response structure
#[derive(Serialize, Deserialize)]
pub struct Resp<T> {
    code: u32,
    msg: String,
    data: Option<T>,
}

impl<T> Resp<T> {
    // Create success response
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            msg: "Success".to_string(),
            data: Some(data),
        }
    }

    // Create error response
    pub fn error(code: u32, msg: String) -> Self {
        Self {
            code,
            msg,
            data: None,
        }
    }
}
