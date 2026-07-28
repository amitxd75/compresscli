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

        if height == 0 {
            return Err(CompressError::invalid_parameter("resolution", resolution));
        }

        // Map common resolution heights or compute 16:9 width (rounded to even number for FFmpeg)
        let width = match height {
            240 => 320,   // QVGA
            360 => 640,   // nHD (16:9)
            480 => 854,   // FWVGA (16:9)
            540 => 960,   // qHD
            576 => 1024,  // 576p PAL
            720 => 1280,  // HD
            1080 => 1920, // Full HD
            1440 => 2560, // QHD
            2160 => 3840, // 4K UHD
            4320 => 7680, // 8K UHD
            _ => (height * 16 / 9) & !1,
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

        if width == 0 || height == 0 {
            return Err(CompressError::invalid_parameter("resolution", resolution));
        }

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

                if minutes < 0.0 || !(0.0..60.0).contains(&seconds) {
                    return Err(CompressError::invalid_parameter("time", time_str));
                }

                let total = minutes * 60.0 + seconds;
                if !total.is_finite() {
                    return Err(CompressError::invalid_parameter("time", time_str));
                }
                Ok(total)
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

                if hours < 0.0 || !(0.0..60.0).contains(&minutes) || !(0.0..60.0).contains(&seconds)
                {
                    return Err(CompressError::invalid_parameter("time", time_str));
                }

                let total = hours * 3600.0 + minutes * 60.0 + seconds;
                if !total.is_finite() {
                    return Err(CompressError::invalid_parameter("time", time_str));
                }
                Ok(total)
            }
            _ => Err(CompressError::invalid_parameter("time", time_str)),
        }
    } else {
        // Just seconds as a number
        let total: f64 = time_str
            .parse()
            .map_err(|_| CompressError::invalid_parameter("time", time_str))?;
        if !total.is_finite() || total < 0.0 {
            return Err(CompressError::invalid_parameter("time", time_str));
        }
        Ok(total)
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
        assert_eq!(parse_resolution("540p").unwrap(), (960, 540));
        assert!(parse_resolution("invalid").is_err());
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("90").unwrap(), 90.0);
        assert_eq!(parse_time("1:30").unwrap(), 90.0);
        assert_eq!(parse_time("01:01:30").unwrap(), 3690.0);
        assert!(parse_time("invalid").is_err());
        assert!(parse_time("1:99").is_err());
        assert!(parse_time("1:-1:30").is_err());
    }
}
