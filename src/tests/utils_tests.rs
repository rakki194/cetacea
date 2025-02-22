#[cfg(test)]
mod tests {
    use crate::format_duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_format_duration() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        // Test seconds
        assert_eq!(format_duration(now - 30), "30 seconds ago");
        
        // Test minutes
        assert_eq!(format_duration(now - 120), "2 minutes ago");
        
        // Test hours
        assert_eq!(format_duration(now - 7200), "2 hours ago");
        
        // Test days
        assert_eq!(format_duration(now - 172800), "2 days ago");
    }
} 