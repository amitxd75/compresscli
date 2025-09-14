//! Mathematical utilities for calculations

/// Calculates compression ratio as a percentage
/// Returns the percentage of size reduction achieved by compression
/// Example: 1000 bytes -> 500 bytes = 50.0% compression ratio
pub fn calculate_compression_ratio(original_size: u64, compressed_size: u64) -> f64 {
    if original_size == 0 {
        return 0.0;
    }
    
    let ratio = (original_size as f64 - compressed_size as f64) / original_size as f64;
    ratio * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_ratio() {
        assert_eq!(calculate_compression_ratio(1000, 500), 50.0);
        assert_eq!(calculate_compression_ratio(1000, 1000), 0.0);
        assert_eq!(calculate_compression_ratio(0, 500), 0.0);
    }
}
