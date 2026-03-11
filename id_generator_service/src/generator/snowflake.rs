//! Snowflake 算法实现
//! 
//! 经典雪花算法变体：
//! - 41 位时间戳（毫秒级，可用约 69 年）
//! - 10 位机器 ID（5 位机房 ID + 5 位工作机器 ID）
//! - 12 位序列号（每毫秒最多生成 4096 个 ID）
//! 
//! 格式：[timestamp:41][datacenter_id:5][worker_id:5][sequence:12]

use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::Mutex;
use crate::error::{GeneratorError, AppError, Result};
use crate::utils::time::current_timestamp_millis;

/// Snowflake 配置常量
mod constants {
    /// 起始时间戳（2024-01-01 00:00:00 UTC）
    pub const EPOCH: u64 = 1704067200000;
    
    /// 时间戳位数
    pub const TIMESTAMP_BITS: u32 = 41;
    
    /// 机房 ID 位数
    pub const DATACENTER_ID_BITS: u32 = 5;
    
    /// 机器 ID 位数
    pub const WORKER_ID_BITS: u32 = 5;
    
    /// 序列号位数
    pub const SEQUENCE_BITS: u32 = 12;
    
    /// 机房 ID 最大值：31
    pub const MAX_DATACENTER_ID: u64 = (1 << DATACENTER_ID_BITS) - 1;
    
    /// 机器 ID 最大值：31
    pub const MAX_WORKER_ID: u64 = (1 << WORKER_ID_BITS) - 1;
    
    /// 序列号最大值：4095
    pub const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;
    
    /// 机房 ID 左移位数
    pub const DATACENTER_ID_SHIFT: u32 = WORKER_ID_BITS + SEQUENCE_BITS;
    
    /// 机器 ID 左移位数
    pub const WORKER_ID_SHIFT: u32 = SEQUENCE_BITS;
    
    /// 时间戳左移位数
    pub const TIMESTAMP_SHIFT: u32 = 0;
}

use constants::*;

/// Snowflake ID 生成器
pub struct SnowflakeGenerator {
    /// 机房 ID
    datacenter_id: u64,
    /// 机器 ID
    worker_id: u64,
    /// 序列号
    sequence: AtomicU64,
    /// 上次生成 ID 的时间戳
    last_timestamp: Mutex<u64>,
}

impl SnowflakeGenerator {
    /// 创建新的 Snowflake 生成器
    /// 
    /// # Arguments
    /// * `worker_id` - 机器 ID (0-31)
    /// * `datacenter_id` - 机房 ID (0-31)
    /// 
    /// # Errors
    /// 如果 ID 超出范围，返回错误
    pub fn new(worker_id: u64, datacenter_id: u64) -> Result<Self> {
        if worker_id > MAX_WORKER_ID {
            return Err(AppError::Generator(GeneratorError::InvalidWorkerId(worker_id)));
        }
        
        if datacenter_id > MAX_DATACENTER_ID {
            return Err(AppError::Generator(GeneratorError::InvalidDatacenterId(datacenter_id)));
        }
        
        Ok(Self {
            datacenter_id,
            worker_id,
            sequence: AtomicU64::new(0),
            last_timestamp: Mutex::new(0),
        })
    }
    
    /// 获取当前时间戳与起始时间的差值
    #[inline]
    fn timestamp(&self) -> u64 {
        current_timestamp_millis() - EPOCH
    }
    
    /// 等待下一毫秒
    fn wait_next_millis(&self, last_ts: u64) -> u64 {
        let mut ts = self.timestamp();
        while ts <= last_ts {
            ts = self.timestamp();
        }
        ts
    }
    
    /// 生成下一个 ID
    pub fn next_id(&self) -> Result<u64> {
        let mut ts = self.timestamp();
        
        // 获取并更新锁
        let mut last_ts = self.last_timestamp.lock();
        
        // 时钟回退检测
        if ts < *last_ts {
            return Err(AppError::Generator(GeneratorError::ClockMovedBackwards));
        }
        
        // 同一毫秒内，序列号递增
        if ts == *last_ts {
            let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
            
            // 序列号溢出处理
            if seq > MAX_SEQUENCE {
                // 等待下一毫秒
                ts = self.wait_next_millis(*last_ts);
                self.sequence.store(0, Ordering::SeqCst);
            }
        } else {
            // 新毫秒，重置序列号
            self.sequence.store(0, Ordering::SeqCst);
        }
        
        *last_ts = ts;
        drop(last_ts);
        
        // 组合 ID
        let id = ((ts & ((1 << TIMESTAMP_BITS) - 1)) << TIMESTAMP_SHIFT)
            | ((self.datacenter_id & MAX_DATACENTER_ID) << DATACENTER_ID_SHIFT)
            | ((self.worker_id & MAX_WORKER_ID) << WORKER_ID_SHIFT)
            | (self.sequence.load(Ordering::SeqCst) & MAX_SEQUENCE);
        
        Ok(id)
    }
}

impl super::IdGenerator for SnowflakeGenerator {
    fn generate(&self) -> Result<String> {
        let id = self.next_id()?;
        Ok(id.to_string())
    }
    
    fn name(&self) -> &str {
        "snowflake"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::IdGenerator;

    #[test]
    fn test_new_generator() {
        let gen = SnowflakeGenerator::new(1, 1);
        assert!(gen.is_ok());
    }

    #[test]
    fn test_invalid_worker_id() {
        let gen = SnowflakeGenerator::new(100, 1);
        assert!(gen.is_err());
    }

    #[test]
    fn test_generate_id() {
        let gen = SnowflakeGenerator::new(1, 1).unwrap();
        let id = gen.generate().unwrap();
        assert!(!id.is_empty());
        assert!(id.parse::<u64>().is_ok());
    }

    #[test]
    fn test_generate_unique_ids() {
        let gen = SnowflakeGenerator::new(1, 1).unwrap();
        let id1 = gen.generate().unwrap();
        let id2 = gen.generate().unwrap();
        assert_ne!(id1, id2);
    }
}
