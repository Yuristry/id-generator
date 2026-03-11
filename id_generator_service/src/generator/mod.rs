//! ID 生成器核心模块
//! 
//! 定义了统一的 IdGenerator Trait 和工厂模式，支持多种算法

pub mod snowflake;
pub mod ulid;
pub mod nanoid;

use std::sync::Arc;
use crate::error::{GeneratorError, Result};

/// ID 生成器 Trait
/// 
/// 所有 ID 生成算法都必须实现此 Trait
pub trait IdGenerator: Send + Sync {
    /// 生成单个 ID
    fn generate(&self) -> Result<String>;
    
    /// 批量生成 ID
    fn generate_batch(&self, count: usize) -> Result<Vec<String>> {
        (0..count).map(|_| self.generate()).collect()
    }
    
    /// 获取算法名称
    fn name(&self) -> &str;
}

/// 生成器类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorType {
    Snowflake,
    ULID,
    NanoID,
}

impl std::str::FromStr for GeneratorType {
    type Err = GeneratorError;
    
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "snowflake" => Ok(GeneratorType::Snowflake),
            "ulid" => Ok(GeneratorType::ULID),
            "nanoid" | "nano_id" => Ok(GeneratorType::NanoID),
            _ => Err(GeneratorError::UnsupportedAlgorithm(s.to_string())),
        }
    }
}

impl std::fmt::Display for GeneratorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneratorType::Snowflake => write!(f, "snowflake"),
            GeneratorType::ULID => write!(f, "ulid"),
            GeneratorType::NanoID => write!(f, "nanoid"),
        }
    }
}

/// ID 生成器工厂
/// 
/// 根据配置创建对应的生成器实例
pub struct GeneratorFactory;

impl GeneratorFactory {
    /// 创建 Snowflake 生成器
    pub fn create_snowflake(worker_id: u64, datacenter_id: u64) -> Result<Arc<dyn IdGenerator>> {
        let generator = snowflake::SnowflakeGenerator::new(worker_id, datacenter_id)?;
        Ok(Arc::new(generator))
    }
    
    /// 创建 ULID 生成器
    pub fn create_ulid() -> Result<Arc<dyn IdGenerator>> {
        let generator = ulid::ULIDGenerator::new();
        Ok(Arc::new(generator))
    }
    
    /// 创建 NanoID 生成器
    pub fn create_nanoid(length: usize, alphabet: &str) -> Result<Arc<dyn IdGenerator>> {
        let generator = nanoid::NanoIDGenerator::new(length, alphabet)?;
        Ok(Arc::new(generator))
    }
    
    /// 根据类型创建生成器
    pub fn create(typ: GeneratorType, config: &crate::config::Settings) -> Result<Arc<dyn IdGenerator>> {
        match typ {
            GeneratorType::Snowflake => {
                Self::create_snowflake(config.snowflake.worker_id, config.snowflake.datacenter_id)
            }
            GeneratorType::ULID => Self::create_ulid(),
            GeneratorType::NanoID => {
                Self::create_nanoid(config.nanoid.length, &config.nanoid.alphabet)
            }
        }
    }
}
