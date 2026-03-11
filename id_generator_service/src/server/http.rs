//! HTTP 路由和 Handler 实现
//! 
//! 使用 Axum 框架构建 RESTful API

use std::sync::Arc;
use std::time::Instant;
use axum::{
    Json, Router, routing::{get, post},
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::server::state::AppState;

/// ID 生成请求体（可选，用于批量生成）
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GenerateRequest {
    /// 生成数量，默认 1
    pub count: usize,
}

impl Default for GenerateRequest {
    fn default() -> Self {
        Self { count: 1 }
    }
}

/// 单个 ID 响应
#[derive(Debug, Serialize)]
pub struct IdResponse {
    pub id: String,
    pub algorithm: String,
    pub timestamp: u64,
}

/// 批量 ID 响应
#[derive(Debug, Serialize)]
pub struct BatchIdResponse {
    pub ids: Vec<String>,
    pub algorithm: String,
    pub count: usize,
}

/// 健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u64,
}

/// 错误响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: bool,
    pub message: String,
}

/// 构建应用路由
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // 健康检查端点
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        // 指标端点
        .route("/metrics", get(metrics_handler))
        // ID 生成端点
        .route("/api/v1/id/snowflake", post(generate_snowflake))
        .route("/api/v1/id/ulid", post(generate_ulid))
        .route("/api/v1/id/nanoid", post(generate_nanoid))
        // 批量生成端点
        .route("/api/v1/id/batch/:algorithm/:count", get(batch_generate))
        // 应用状态
        .with_state(state)
}

/// 健康检查 Handler
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: crate::utils::time::current_timestamp_millis(),
    })
}

/// 就绪检查 Handler
async fn readiness_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    // 检查所有生成器是否可用
    let is_ready = state.snowflake.generate().is_ok()
        && state.ulid.generate().is_ok()
        && state.nanoid.generate().is_ok();
    
    if !is_ready {
        warn!("Service not ready: one or more generators failed");
    }
    
    Json(HealthResponse {
        status: if is_ready { "ready" } else { "not_ready" }.to_string(),
        timestamp: crate::utils::time::current_timestamp_millis(),
    })
}

/// Prometheus 指标 Handler
async fn metrics_handler(State(state): State<Arc<AppState>>) -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = state.metrics_registry.gather();
    let mut buffer = Vec::new();
    
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => String::from_utf8_lossy(&buffer).to_string(),
        Err(e) => {
            error!("Failed to encode metrics: {}", e);
            "Error encoding metrics".to_string()
        }
    }
}

/// 生成 Snowflake ID
async fn generate_snowflake(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<BatchIdResponse>, (StatusCode, Json<ErrorResponse>)> {
    generate_ids(&state, "snowflake", req.count)
}

/// 生成 ULID
async fn generate_ulid(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<BatchIdResponse>, (StatusCode, Json<ErrorResponse>)> {
    generate_ids(&state, "ulid", req.count)
}

/// 生成 NanoID
async fn generate_nanoid(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<BatchIdResponse>, (StatusCode, Json<ErrorResponse>)> {
    generate_ids(&state, "nanoid", req.count)
}

/// 批量生成 ID
async fn batch_generate(
    State(state): State<Arc<AppState>>,
    Path((algorithm, count)): Path<(String, usize)>,
) -> Result<Json<BatchIdResponse>, (StatusCode, Json<ErrorResponse>)> {
    if count == 0 || count > 1000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: true,
                message: "count 必须在 1-1000 范围内".to_string(),
            }),
        ));
    }
    
    generate_ids(&state, &algorithm, count)
}

/// 通用的 ID 生成逻辑
fn generate_ids(
    state: &AppState,
    algorithm: &str,
    count: usize,
) -> Result<Json<BatchIdResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 限制批量大小
    let count = count.min(100).max(1);
    
    let start = Instant::now();
    
    let generator = state.get_generator(algorithm).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: true,
                message: format!("不支持的算法：{}", algorithm),
            }),
        )
    })?;
    
    // 生成 ID
    let result = if count == 1 {
        generator.generate().map(|id| vec![id])
    } else {
        generator.generate_batch(count)
    };
    
    let duration = start.elapsed();
    
    // 记录指标
    let status = if result.is_ok() { "success" } else { "error" };
    state.request_counter
        .with_label_values(&[algorithm, status])
        .inc();
    
    state.generation_histogram
        .with_label_values(&[algorithm])
        .observe(duration.as_secs_f64());
    
    // 处理结果
    match result {
        Ok(ids) => {
            info!(
                algorithm = %algorithm,
                count = ids.len(),
                duration_us = duration.as_micros(),
                "Generated IDs"
            );
            
            Ok(Json(BatchIdResponse {
                ids,
                algorithm: algorithm.to_string(),
                count,
            }))
        }
        Err(e) => {
            error!(
                algorithm = %algorithm,
                error = %e,
                "Failed to generate ID"
            );
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: true,
                    message: e.to_string(),
                }),
            ))
        }
    }
}
