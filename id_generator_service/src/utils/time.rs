//! 时间相关工具函数

use time::OffsetDateTime;

/// 获取当前时间的毫秒级时间戳（自 Unix 纪元）
/// 
/// # Returns
/// 返回从 1970-01-01 00:00:00 UTC 到现在的毫秒数
#[inline]
pub fn current_timestamp_millis() -> u64 {
    OffsetDateTime::now_utc().unix_timestamp() as u64 * 1_000
        + OffsetDateTime::now_utc().millisecond() as u64
}

/// 获取当前时间的微秒级时间戳（自 Unix 纪元）
/// 
/// # Returns
/// 返回从 1970-01-01 00:00:00 UTC 到现在的微秒数
#[inline]
pub fn current_timestamp_micros() -> u64 {
    OffsetDateTime::now_utc().unix_timestamp() as u64 * 1_000_000
        + OffsetDateTime::now_utc().microsecond() as u64
}

/// 获取当前时间的纳秒级时间戳（自 Unix 纪元）
/// 
/// # Returns
/// 返回从 1970-01-01 00:00:00 UTC 到现在的纳秒数
#[inline]
pub fn current_timestamp_nanos() -> u128 {
    let now = OffsetDateTime::now_utc();
    (now.unix_timestamp() as u128 * 1_000_000_000)
        + now.nanosecond() as u128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_millis() {
        let ts1 = current_timestamp_millis();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = current_timestamp_millis();
        assert!(ts2 >= ts1);
    }

    #[test]
    fn test_timestamp_micros() {
        let ts = current_timestamp_micros();
        assert!(ts > 0);
    }
}
