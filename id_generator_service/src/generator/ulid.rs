//! ULID 算法实现
//! 
//! ULID (Universally Unique Lexicographically Sortable Identifier)
//! - 128 位兼容性，与 UUID 互转
//! - 按时间排序
//! - Crockford's Base32 编码（32 字符集：0-9, A-Z 去除 I,L,O,U）
//! 
//! 格式：[timestamp:48bits][randomness:80bits]
//! 编码后：26 个字符

use ulid::Ulid;
use crate::error::Result;

/// ULID ID 生成器
pub struct ULIDGenerator;

impl ULIDGenerator {
    /// 创建新的 ULID 生成器
    pub fn new() -> Self {
        Self
    }
    
    /// 生成下一个 ULID
    pub fn next_id(&self) -> Ulid {
        Ulid::new()
    }
}

impl Default for ULIDGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl super::IdGenerator for ULIDGenerator {
    fn generate(&self) -> Result<String> {
        let ulid = self.next_id();
        Ok(ulid.to_string())
    }
    
    fn name(&self) -> &str {
        "ulid"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::IdGenerator;

    #[test]
    fn test_generate_ulid() {
        let gen = ULIDGenerator::new();
        let id = gen.generate().unwrap();
        assert_eq!(id.len(), 26); // ULID 固定 26 字符
    }

    #[test]
    fn test_generate_unique_ids() {
        let gen = ULIDGenerator::new();
        let id1 = gen.generate().unwrap();
        let id2 = gen.generate().unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_ulid_format() {
        let gen = ULIDGenerator::new();
        let id = gen.generate().unwrap();
        // ULID 使用 Crockford's Base32，只包含特定字符
        assert!(id.chars().all(|c| c.is_ascii_digit() || ('A'..='Z').contains(&c)));
        // 不应包含 I, L, O, U
        assert!(!id.contains('I'));
        assert!(!id.contains('L'));
        assert!(!id.contains('O'));
        assert!(!id.contains('U'));
    }
}
