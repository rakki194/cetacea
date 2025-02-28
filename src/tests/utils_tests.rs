#[cfg(test)]
mod tests {
    use cetacea::utils::format_duration;

    #[test]
    fn test_format_duration() {
        // Test cases with different timestamps
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert_eq!(format_duration(now), "0 seconds ago");
        assert_eq!(format_duration(now - 30), "30 seconds ago");
        assert_eq!(format_duration(now - 90), "1 minutes ago");
        assert_eq!(format_duration(now - 3600), "1 hours ago");
        assert_eq!(format_duration(now - 86400), "1 days ago");
    }
} 