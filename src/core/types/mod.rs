//! Domain types, options, parameters, and enums module

pub mod batch;
pub mod common;
pub mod image;
pub mod video;

pub use batch::*;
pub use common::*;
pub use image::*;
pub use video::*;

#[cfg(test)]
mod tests {
    use super::*;
    use clap::ValueEnum;

    #[test]
    fn test_from_str_image_format() {
        assert!(matches!(
            ImageFormat::parse_from_str("jpeg"),
            Ok(ImageFormat::Jpeg)
        ));
        assert!(matches!(
            ImageFormat::parse_from_str("JPG"),
            Ok(ImageFormat::Jpeg)
        ));
        assert!(matches!(
            ImageFormat::parse_from_str("png"),
            Ok(ImageFormat::Png)
        ));
        assert!(matches!(
            ImageFormat::parse_from_str("webp"),
            Ok(ImageFormat::Webp)
        ));
        assert!(matches!(
            ImageFormat::parse_from_str("avif"),
            Ok(ImageFormat::Avif)
        ));
        assert!(ImageFormat::parse_from_str("invalid").is_err());
    }

    #[test]
    fn test_from_str_video_codec() {
        assert!(matches!(
            VideoCodec::from_str("h264", true),
            Ok(VideoCodec::H264)
        ));
        assert!(matches!(
            VideoCodec::from_str("H265", true),
            Ok(VideoCodec::H265)
        ));
        assert!(matches!(
            VideoCodec::from_str("vp9", true),
            Ok(VideoCodec::Vp9)
        ));
        assert!(matches!(
            VideoCodec::from_str("av1", true),
            Ok(VideoCodec::Av1)
        ));
        assert!(VideoCodec::from_str("invalid", true).is_err());
    }
}
