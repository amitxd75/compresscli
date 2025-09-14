//! Parsing utilities for handling various input formats

use crate::core::error::{CompressError, Result};

/// Parses resolution string into width and height values
/// Supports formats like "1920x1080", "720p", "1080p", "4K"
/// Returns tuple of (width, height) in pixels
pub fn parse_resolution(resolution: &str) -> Result<(u32, u32)> {
    if let Some(height_str) = resolution.strip_suffix('p') {
        let height: u32 = height_str
            .parse()
            .map_err(|_| CompressError::invalid_parameter("resolution", resolution))?;
        
        // Map common resolution heights to their standard widths
        let width = match height {
            240 => 320,   // QVGA
            360 => 480,   // nHD
            480 => 640,   // VGA
            720 => 1280,  // HD
            1080 => 1920, // Full HD
            1440 => 2560, // QHD
            2160 => 3840, // 4K UHD
            _ => return Err(CompressError::invalid_parameter("resolution", resolution)),
        };
        
        Ok((width, height))
    } else if resolution.contains('x') {
        // Parse "WIDTHxHEIGHT" format
        let parts: Vec<&str> = resolution.split('x').collect();
        if parts.len() != 2 {
            return Err(CompressError::invalid_parameter("resolution", resolution));
        }
        
        let width: u32 = parts[0]
            .parse()
            .map_err(|_| CompressError::invalid_parameter("resolution", resolution))?;
        let height: u32 = parts[1]
            .parse()
            .map_err(|_| CompressError::invalid_parameter("resolution", resolution))?;
        
        Ok((width, height))
    } else {
        Err(CompressError::invalid_parameter("resolution", resolution))
    }
}

/// Parses time string into seconds as floating point
/// Supports formats: "90" (seconds), "1:30" (MM:SS), "01:30:45" (HH:MM:SS)
/// Used for video trimming start/end times
pub fn parse_time(time_str: &str) -> Result<f64> {
    if time_str.contains(':') {
        let parts: Vec<&str> = time_str.split(':').collect();
        match parts.len() {
            2 => {
                // MM:SS format
                let minutes: f64 = parts[0]
                    .parse()
                    .map_err(|_| CompressError::invalid_parameter("time", time_str))?;
                let seconds: f64 = parts[1]
                    .parse()
                    .map_err(|_| CompressError::invalid_parameter("time", time_str))?;
                Ok(minutes * 60.0 + seconds)
            }
            3 => {
                // HH:MM:SS format
                let hours: f64 = parts[0]
                    .parse()
                    .map_err(|_| CompressError::invalid_parameter("time", time_str))?;
                let minutes: f64 = parts[1]
                    .parse()
                    .map_err(|_| CompressError::invalid_parameter("time", time_str))?;
                let seconds: f64 = parts[2]
                    .parse()
                    .map_err(|_| CompressError::invalid_parameter("time", time_str))?;
                Ok(hours * 3600.0 + minutes * 60.0 + seconds)
            }
            _ => Err(CompressError::invalid_parameter("time", time_str)),
        }
    } else {
        // Just seconds as a number
        time_str
            .parse()
            .map_err(|_| CompressError::invalid_parameter("time", time_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resolution() {
        assert_eq!(parse_resolution("1920x1080").unwrap(), (1920, 1080));
        assert_eq!(parse_resolution("720p").unwrap(), (1280, 720));
        assert_eq!(parse_resolution("1080p").unwrap(), (1920, 1080));
        assert!(parse_resolution("invalid").is_err());
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("90").unwrap(), 90.0);
        assert_eq!(parse_time("1:30").unwrap(), 90.0);
        assert_eq!(parse_time("01:01:30").unwrap(), 3690.0);
        assert!(parse_time("invalid").is_err());
    }
}
