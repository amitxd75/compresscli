/// Calculates compression ratio as a percentage
/// Returns the percentage of size reduction achieved by compression
/// Example: 1000 bytes -> 500 bytes = 50.0% reduction, 500 -> 1000 = -100.0% (growth)
pub fn calculate_compression_ratio(original_size: u64, compressed_size: u64) -> f64 {
    if original_size == 0 {
        return 0.0;
    }

    let ratio = (original_size as f64 - compressed_size as f64) / original_size as f64;
    ratio * 100.0
}

/// Formats compression ratio with contextual description (e.g. "50.0% reduction" or "10.0% size increase")
pub fn format_compression_ratio(original_size: u64, compressed_size: u64) -> String {
    let ratio = calculate_compression_ratio(original_size, compressed_size);
    if ratio >= 0.0 {
        format!("{:.1}% reduction", ratio)
    } else {
        format!("{:.1}% size increase", ratio.abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_ratio() {
        assert_eq!(calculate_compression_ratio(1000, 500), 50.0);
        assert_eq!(calculate_compression_ratio(1000, 1000), 0.0);
        assert_eq!(calculate_compression_ratio(0, 500), 0.0);
        assert_eq!(format_compression_ratio(1000, 500), "50.0% reduction");
        assert_eq!(format_compression_ratio(500, 600), "20.0% size increase");
    }
}
