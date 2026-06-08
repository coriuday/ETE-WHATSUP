use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl PaginationQuery {
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(20).min(100)
    }

    pub fn offset(&self) -> i64 {
        ((self.page() - 1) * self.limit()) as i64
    }

    pub fn limit_i64(&self) -> i64 {
        self.limit() as i64
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, query: &PaginationQuery) -> Self {
        let page = query.page();
        let limit = query.limit();
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

        Self {
            data,
            pagination: PaginationMeta {
                total,
                page,
                limit,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
        }
    }
}

/// Standard API success response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data, message: None }
    }

    pub fn with_message(data: T, message: impl Into<String>) -> Self {
        Self { success: true, data, message: Some(message.into()) }
    }
}
