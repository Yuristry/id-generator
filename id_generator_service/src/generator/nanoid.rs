//! NanoID 算法实现
//! 
//! NanoID 是一个小巧、安全、不依赖环境的 ID 生成库
//! - 可配置长度和字符集
//! - 使用加密安全的随机数生成器
//! - URL 友好

use rand::Rng;
use crate::error::{GeneratorError, AppError, Result};

/// NanoID 生成器
pub struct NanoIDGenerator {
    /// ID 长度
    length: usize,
    /// 字符集
    alphabet: Vec<char>,
}

impl NanoIDGenerator {
    /// 创建新的 NanoID 生成器
    /// 
    /// # Arguments
    /// * `length` - 生成的 ID 长度
    /// * `alphabet` - 使用的字符集
    /// 
    /// # Errors
    /// 如果字符集为空，返回错误
    pub fn new(length: usize, alphabet: &str) -> Result<Self> {
        if alphabet.is_empty() {
            return Err(AppError::Generator(GeneratorError::GenerationFailed(
                "字符集不能为空".to_string()
            )));
        }
        
        let alphabet_chars: Vec<char> = alphabet.chars().collect();
        
        // 检查字符集是否有重复
        let unique_count = alphabet_chars.iter().collect::<std::collections::HashSet<_>>().len();
        if unique_count != alphabet_chars.len() {
            return Err(AppError::Generator(GeneratorError::GenerationFailed(
                "字符集包含重复字符".to_string()
            )));
        }
        
        Ok(Self {
            length,
            alphabet: alphabet_chars,
        })
    }
    
    /// 使用默认 URL-safe 字符集创建生成器
    pub fn with_default_alphabet(length: usize) -> Result<Self> {
        Self::new(length, "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_")
    }
    
    /// 生成下一个 NanoID
    pub fn next_id(&self) -> String {
        let mut rng = rand::thread_rng();
        (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..self.alphabet.len());
                self.alphabet[idx]
            })
            .collect()
    }
}

impl super::IdGenerator for NanoIDGenerator {
    fn generate(&self) -> Result<String> {
        Ok(self.next_id())
    }
    
    fn name(&self) -> &str {
        "nanoid"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::IdGenerator;

    #[test]
    fn test_new_generator() {
        let gen = NanoIDGenerator::with_default_alphabet(21);
        assert!(gen.is_ok());
    }

    #[test]
    fn test_empty_alphabet() {
        let gen = NanoIDGenerator::new(21, "");
        assert!(gen.is_err());
    }

    #[test]
    fn test_generate_id() {
        let gen = NanoIDGenerator::with_default_alphabet(21).unwrap();
        let id = gen.generate().unwrap();
        assert_eq!(id.len(), 21);
    }

    #[test]
    fn test_generate_unique_ids() {
        let gen = NanoIDGenerator::with_default_alphabet(21).unwrap();
        let id1 = gen.generate().unwrap();
        let id2 = gen.generate().unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_custom_length() {
        let gen = NanoIDGenerator::with_default_alphabet(32).unwrap();
        let id = gen.generate().unwrap();
        assert_eq!(id.len(), 32);
    }
}
