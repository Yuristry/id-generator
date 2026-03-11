//! 全局错误定义模块
//! 
//! 使用 thiserror 定义统一的错误类型，便于错误处理和 API 响应

use thiserror::Error;

/// ID 生成器相关错误
#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("时钟回退检测：服务器时间似乎倒流")]
    ClockMovedBackwards,

    #[error("序列号溢出：当前毫秒内已生成过多 ID")]
    SequenceOverflow,

    #[error("无效的机器 ID: {0}")]
    InvalidWorkerId(u64),

    #[error("无效的机房 ID: {0}")]
    InvalidDatacenterId(u64),

    #[error("ID 生成失败：{0}")]
    GenerationFailed(String),

    #[error("不支持的算法：{0}")]
    UnsupportedAlgorithm(String),
}

/// 配置相关错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("加载配置文件失败：{0}")]
    LoadFailed(#[from] ::config::ConfigError),

    #[error("缺少必要配置：{0}")]
    MissingField(String),

    #[error("配置值无效：{0}")]
    InvalidValue(String),
}

/// HTTP 服务相关错误
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("服务启动失败：{0}")]
    StartupFailed(String),

    #[error("绑定端口失败：{0}")]
    BindFailed(String),

    #[error("内部服务器错误：{0}")]
    InternalError(String),
}

/// 统一的应用程序错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("生成器错误：{0}")]
    Generator(#[from] GeneratorError),

    #[error("配置错误：{0}")]
    Config(#[from] ConfigError),

    #[error("服务器错误：{0}")]
    Server(#[from] ServerError),

    #[error("IO 错误：{0}")]
    Io(#[from] std::io::Error),
}

/// Axum 响应转换器
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Generator(e) => match e {
                GeneratorError::ClockMovedBackwards => {
                    (axum::http::StatusCode::SERVICE_UNAVAILABLE, e.to_string())
                }
                GeneratorError::SequenceOverflow => {
                    (axum::http::StatusCode::TOO_MANY_REQUESTS, e.to_string())
                }
                _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            },
            AppError::Config(e) => {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            AppError::Server(e) => {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            AppError::Io(e) => {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        };

        // 返回 JSON 格式错误响应
        let body = serde_json::json!({
            "error": true,
            "message": message,
        });

        (status, axum::Json(body)).into_response()
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, AppError>;
