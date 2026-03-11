//! 应用状态管理
//! 
//! 持有所有生成器实例和指标注册表

use std::sync::Arc;
use axum::extract::State;
use prometheus::Registry;

use crate::generator::IdGenerator;

/// 应用全局状态
/// 
/// 通过 Axum 的 State extractor 在 Handler 中访问
#[derive(Clone)]
pub struct AppState {
    /// Snowflake 生成器
    pub snowflake: Arc<dyn IdGenerator>,
    /// ULID 生成器
    pub ulid: Arc<dyn IdGenerator>,
    /// NanoID 生成器
    pub nanoid: Arc<dyn IdGenerator>,
    /// Prometheus 指标注册表
    pub metrics_registry: Arc<Registry>,
    /// 请求计数器
    pub request_counter: Arc<prometheus::CounterVec>,
    /// 生成耗时直方图
    pub generation_histogram: Arc<prometheus::HistogramVec>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(
        snowflake: Arc<dyn IdGenerator>,
        ulid: Arc<dyn IdGenerator>,
        nanoid: Arc<dyn IdGenerator>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        
        // 注册请求计数器
        let request_counter = prometheus::CounterVec::new(
            prometheus::Opts::new(
                "idgen_requests_total",
                "Total number of ID generation requests",
            ),
            &["algorithm", "status"],
        )?;
        registry.register(Box::new(request_counter.clone()))?;
        
        // 注册生成耗时直方图
        let generation_histogram = prometheus::HistogramVec::new(
            prometheus::HistogramOpts::new(
                "idgen_generation_duration_seconds",
                "ID generation duration in seconds",
            )
            .buckets(vec![0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01]),
            &["algorithm"],
        )?;
        registry.register(Box::new(generation_histogram.clone()))?;
        
        Ok(Self {
            snowflake,
            ulid,
            nanoid,
            metrics_registry: Arc::new(registry),
            request_counter: Arc::new(request_counter),
            generation_histogram: Arc::new(generation_histogram),
        })
    }
    
    /// 根据算法名称获取生成器
    pub fn get_generator(&self, algorithm: &str) -> Option<&Arc<dyn IdGenerator>> {
        match algorithm.to_lowercase().as_str() {
            "snowflake" => Some(&self.snowflake),
            "ulid" => Some(&self.ulid),
            "nanoid" => Some(&self.nanoid),
            _ => None,
        }
    }
}

/// Axum State 提取器别名
pub type AppRef<'a> = State<Arc<AppState>>;
