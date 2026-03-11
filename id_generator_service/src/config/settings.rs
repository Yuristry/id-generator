//! 配置管理模块
//! 
//! 支持从 YAML 文件和环境变量加载配置
//! 环境变量优先级高于配置文件

use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, File, Environment};

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
        }
    }
}

/// Snowflake 算法配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SnowflakeConfig {
    /// 机器 ID (0-1023)
    pub worker_id: u64,
    /// 机房 ID (0-31)
    pub datacenter_id: u64,
}

impl Default for SnowflakeConfig {
    fn default() -> Self {
        Self {
            worker_id: 1,
            datacenter_id: 1,
        }
    }
}

/// NanoID 算法配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NanoIDConfig {
    /// ID 长度
    pub length: usize,
    /// 字符集（默认 URL-safe）
    pub alphabet: String,
}

impl Default for NanoIDConfig {
    fn default() -> Self {
        Self {
            length: 21,
            alphabet: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_".to_string(),
        }
    }
}

/// ULID 算法配置（当前无需特殊配置）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ULIDConfig {
    /// 占位字段，保留扩展性
    pub _placeholder: bool,
}

impl Default for ULIDConfig {
    fn default() -> Self {
        Self {
            _placeholder: false,
        }
    }
}

/// 完整应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// 服务器配置
    pub server: ServerConfig,
    /// Snowflake 配置
    pub snowflake: SnowflakeConfig,
    /// ULID 配置
    pub ulid: ULIDConfig,
    /// NanoID 配置
    pub nanoid: NanoIDConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            snowflake: SnowflakeConfig::default(),
            ulid: ULIDConfig::default(),
            nanoid: NanoIDConfig::default(),
        }
    }
}

impl Settings {
    /// 从配置文件和环境变量加载配置
    /// 
    /// # Arguments
    /// * `config_path` - YAML 配置文件路径
    /// 
    /// # Returns
    /// 返回加载的配置对象
    /// 
    /// # Errors
    /// 如果配置文件不存在或格式错误，返回 ConfigError
    pub fn new(config_path: &str) -> Result<Self, ConfigError> {
        let builder = Config::builder();
        
        // 1. 加载默认配置
        let builder = builder
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("snowflake.worker_id", 1)?
            .set_default("snowflake.datacenter_id", 1)?
            .set_default("nanoid.length", 21)?
            .set_default("nanoid.alphabet", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_")?;
        
        // 2. 从 YAML 文件加载配置
        let builder = builder
            .add_source(File::with_name(config_path).required(false));
        
        // 3. 从环境变量加载配置（优先级最高）
        // 环境变量格式：SERVER_HOST, SERVER_PORT, SNOWFLAKE_WORKER_ID 等
        let builder = builder
            .add_source(
                Environment::with_prefix("IDGEN")
                    .separator("__")
                    .try_parsing(true),
            );
        
        // 构建并解析配置
        let config = builder.build()?;
        let settings = config.try_deserialize()?;
        
        Ok(settings)
    }
    
    /// 验证配置的有效性
    /// 
    /// # Errors
    /// 如果配置值超出允许范围，返回错误
    pub fn validate(&self) -> Result<(), String> {
        // 验证 Snowflake 配置
        if self.snowflake.worker_id > 1023 {
            return Err("worker_id 必须在 0-1023 范围内".to_string());
        }
        
        if self.snowflake.datacenter_id > 31 {
            return Err("datacenter_id 必须在 0-31 范围内".to_string());
        }
        
        // 验证 NanoID 配置
        if self.nanoid.length == 0 {
            return Err("nanoid.length 必须大于 0".to_string());
        }
        
        if self.nanoid.alphabet.is_empty() {
            return Err("nanoid.alphabet 不能为空".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.server.port, 3000);
        assert_eq!(settings.snowflake.worker_id, 1);
        assert_eq!(settings.nanoid.length, 21);
    }

    #[test]
    fn test_validate_default() {
        let settings = Settings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_worker_id() {
        let mut settings = Settings::default();
        settings.snowflake.worker_id = 2000;
        assert!(settings.validate().is_err());
    }
}
