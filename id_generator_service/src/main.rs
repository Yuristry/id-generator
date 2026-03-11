//! 分布式 ID 生成服务
//! 
//! 支持 Snowflake、ULID、NanoID 三种主流算法
//! 提供 HTTP REST API 接口

mod config;
mod error;
mod generator;
mod server;
mod utils;

use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use config::Settings;
use error::Result;
use generator::GeneratorFactory;
use server::{AppState, build_router};

/// 应用版本号
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 应用名称
const APP_NAME: &str = "id-generator-service";

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logging();
    
    info!("{} v{} starting...", APP_NAME, VERSION);
    
    // 加载配置
    let settings = load_config()?;
    
    // 验证配置
    if let Err(e) = settings.validate() {
        error!("Configuration validation failed: {}", e);
        return Err(error::ConfigError::InvalidValue(e).into());
    }
    
    // 创建生成器实例
    let snowflake = GeneratorFactory::create_snowflake(
        settings.snowflake.worker_id,
        settings.snowflake.datacenter_id,
    )?;
    
    let ulid = GeneratorFactory::create_ulid()?;
    
    let nanoid = GeneratorFactory::create_nanoid(
        settings.nanoid.length,
        &settings.nanoid.alphabet,
    )?;
    
    info!(
        worker_id = settings.snowflake.worker_id,
        datacenter_id = settings.snowflake.datacenter_id,
        "Snowflake generator initialized"
    );
    info!("ULID generator initialized");
    info!(
        length = settings.nanoid.length,
        "NanoID generator initialized"
    );
    
    // 创建应用状态
    let app_state = AppState::new(snowflake, ulid, nanoid)
        .map_err(|e| error::ServerError::StartupFailed(e.to_string()))?;
    
    let app_state = Arc::new(app_state);
    
    // 构建路由
    let app = build_router(app_state.clone())
        // 添加追踪中间件
        .layer(TraceLayer::new_for_http())
        // 添加 CORS 支持
        .layer(CorsLayer::new().allow_origin(Any));
    
    // 启动服务器
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| error::ServerError::BindFailed(e.to_string()))?;
    
    info!("Server listening on http://{}", addr);
    info!("Health check: http://{}/health", addr);
    info!("Metrics: http://{}/metrics", addr);
    
    // 启动服务
    axum::serve(listener, app)
        .await
        .map_err(|e| error::ServerError::StartupFailed(e.to_string()))?;
    
    Ok(())
}

/// 初始化日志系统
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new("info,id_generator_service=debug")
        });
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// 加载配置
fn load_config() -> Result<Settings> {
    // 尝试从配置文件加载
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config.yaml".to_string());
    
    info!("Loading configuration from: {}", config_path);
    
    match Settings::new(&config_path) {
        Ok(settings) => {
            info!("Configuration loaded successfully");
            Ok(settings)
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            // 如果配置文件不存在，使用默认配置
            info!("Using default configuration");
            Ok(Settings::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
