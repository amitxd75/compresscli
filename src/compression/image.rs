use crate::cli::args::ImageFormat;
use crate::core::{CompressError, Config, Result};
use crate::ui::progress::print_success;
use crate::utils::{
    calculate_compression_ratio, check_output_overwrite, generate_output_path, get_file_size,
    validate_input_file,
};
use image::{DynamicImage, ImageFormat as ImageLibFormat};
use log::{debug, info};
use std::path::{Path, PathBuf};

pub struct ImageCompressor {
    pub config: Config,
    pub dry_run: bool,
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub struct ImageCompressionOptions {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub quality: u8,
    pub format: Option<ImageFormat>,
    pub resize: Option<String>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub optimize: bool,
    pub progressive: bool,
    pub lossless: bool,
    pub preset: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
}

impl ImageCompressor {
    /// Creates a new ImageCompressor instance
    /// Initializes with configuration, dry-run mode, and verbosity settings
    pub fn new(config: Config, dry_run: bool, verbose: bool) -> Self {
        Self {
            config,
            dry_run,
            verbose,
        }
    }

    /// Compresses an image file using the specified options
    /// Handles preset application, format conversion, resizing, and optimization
    /// Returns the path to the compressed output file
    pub async fn compress(&self, mut options: ImageCompressionOptions) -> Result<PathBuf> {
        // Validate input file exists and is accessible
        validate_input_file(&options.input)?;

        // Apply preset configuration if specified
        if let Some(preset_name) = &options.preset {
            if let Some(preset) = self.config.get_image_preset(preset_name) {
                // Override options with preset values if not explicitly set
                if options.quality == 85 {
                    // Default quality, use preset
                    options.quality = preset.quality;
                }
                if !options.optimize {
                    options.optimize = preset.optimize;
                }
                if !options.progressive {
                    options.progressive = preset.progressive;
                }
                if !options.lossless {
                    options.lossless = preset.lossless;
                }
            } else {
                return Err(CompressError::config(format!(
                    "Image preset '{}' not found",
                    preset_name
                )));
            }
        }

        // Get original file size
        let original_size = get_file_size(&options.input)?;

        // Determine output format and path
        let output_format = self.determine_output_format(&options)?;
        let output_path = self.generate_output_path(&options, &output_format)?;

        // Check overwrite
        check_output_overwrite(&output_path, options.overwrite)?;

        info!(
            "Compressing image: {} -> {}",
            options.input.display(),
            output_path.display()
        );

        if self.dry_run {
            self.print_dry_run_info(&options, &output_format, &output_path);
            return Ok(output_path);
        }

        // Load image
        info!("Loading image...");
        let mut img = image::open(&options.input)?;

        // Apply transformations
        img = self.apply_transformations(img, &options)?;

        // Compress and save
        info!("Compressing and saving...");
        self.save_image(&img, &output_path, &output_format, &options)?;

        // Calculate compression ratio
        let compressed_size = get_file_size(&output_path)?;
        let compression_ratio =
            calculate_compression_ratio(original_size.as_u64(), compressed_size.as_u64());

        print_success(&format!(
            "Image compressed successfully: {} -> {} ({:.1}% reduction)",
            original_size, compressed_size, compression_ratio
        ));

        Ok(output_path)
    }

    fn determine_output_format(&self, options: &ImageCompressionOptions) -> Result<ImageFormat> {
        if let Some(format) = &options.format {
            Ok(format.clone())
        } else {
            // Try to determine from input extension
            if let Some(extension) = options.input.extension() {
                match extension.to_str().unwrap_or("").to_lowercase().as_str() {
                    "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
                    "png" => Ok(ImageFormat::Png),
                    "webp" => Ok(ImageFormat::Webp),
                    "avif" => Ok(ImageFormat::Avif),
                    _ => Ok(ImageFormat::Jpeg), // Default to JPEG
                }
            } else {
                Ok(ImageFormat::Jpeg)
            }
        }
    }

    fn generate_output_path(
        &self,
        options: &ImageCompressionOptions,
        format: &ImageFormat,
    ) -> Result<PathBuf> {
        if let Some(output) = &options.output {
            Ok(output.clone())
        } else {
            let suffix = "_compressed";
            let extension = format.to_string();
            let output_path = generate_output_path(
                &options.input,
                options.output_dir.as_deref(),
                Some(suffix),
                Some(&extension),
            );
            Ok(output_path)
        }
    }

    fn apply_transformations(
        &self,
        mut img: DynamicImage,
        options: &ImageCompressionOptions,
    ) -> Result<DynamicImage> {
        // Resize if specified
        if let Some(resize_str) = &options.resize {
            let (width, height) = self.parse_resize_dimensions(resize_str)?;
            img = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
            debug!("Resized image to {}x{}", width, height);
        }

        // Apply max width/height constraints
        let (current_width, current_height) = (img.width(), img.height());
        let mut new_width = current_width;
        let mut new_height = current_height;

        if let Some(max_width) = options.max_width
            && current_width > max_width
        {
            new_width = max_width;
            new_height = (current_height as f32 * max_width as f32 / current_width as f32) as u32;
        }

        if let Some(max_height) = options.max_height
            && new_height > max_height
        {
            new_height = max_height;
            new_width = (new_width as f32 * max_height as f32 / new_height as f32) as u32;
        }

        if new_width != current_width || new_height != current_height {
            img = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
            debug!(
                "Resized image to fit constraints: {}x{}",
                new_width, new_height
            );
        }

        Ok(img)
    }

    fn save_image(
        &self,
        img: &DynamicImage,
        output_path: &Path,
        format: &ImageFormat,
        _options: &ImageCompressionOptions,
    ) -> Result<()> {
        match format {
            ImageFormat::Jpeg => {
                img.save_with_format(output_path, ImageLibFormat::Jpeg)?;
            }
            ImageFormat::Png => {
                img.save_with_format(output_path, ImageLibFormat::Png)?;
            }
            ImageFormat::Webp => {
                img.save_with_format(output_path, ImageLibFormat::WebP)?;
            }
            ImageFormat::Avif => {
                return Err(CompressError::unsupported_format(
                    "AVIF encoding not yet supported",
                ));
            }
        }

        Ok(())
    }

    fn parse_resize_dimensions(&self, resize_str: &str) -> Result<(u32, u32)> {
        let parts: Vec<&str> = resize_str.split('x').collect();
        if parts.len() != 2 {
            return Err(CompressError::invalid_parameter("resize", resize_str));
        }

        let width: u32 = parts[0]
            .parse()
            .map_err(|_| CompressError::invalid_parameter("resize", resize_str))?;
        let height: u32 = parts[1]
            .parse()
            .map_err(|_| CompressError::invalid_parameter("resize", resize_str))?;

        Ok((width, height))
    }

    fn print_dry_run_info(
        &self,
        options: &ImageCompressionOptions,
        format: &ImageFormat,
        output_path: &Path,
    ) {
        println!(
            "\n{}",
            console::style("DRY RUN - No files will be modified")
                .yellow()
                .bold()
        );
        println!("Input:   {}", options.input.display());
        println!("Output:  {}", output_path.display());
        println!("Format:  {}", format);
        println!("Quality: {}", options.quality);

        if let Some(resize) = &options.resize {
            println!("Resize:  {}", resize);
        }
        if let Some(max_width) = options.max_width {
            println!("Max width: {}", max_width);
        }
        if let Some(max_height) = options.max_height {
            println!("Max height: {}", max_height);
        }

        println!("Optimize: {}", options.optimize);
        println!("Progressive: {}", options.progressive);
        println!("Lossless: {}", options.lossless);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resize_dimensions() {
        let config = Config::default();
        let compressor = ImageCompressor::new(config, false, false);

        assert_eq!(
            compressor.parse_resize_dimensions("800x600").unwrap(),
            (800, 600)
        );
        assert_eq!(
            compressor.parse_resize_dimensions("1920x1080").unwrap(),
            (1920, 1080)
        );
        assert!(compressor.parse_resize_dimensions("invalid").is_err());
    }

    #[test]
    fn test_determine_output_format() {
        let config = Config::default();
        let compressor = ImageCompressor::new(config, false, false);

        let options = ImageCompressionOptions {
            input: PathBuf::from("test.jpg"),
            output: None,
            quality: 85,
            format: None,
            resize: None,
            max_width: None,
            max_height: None,
            optimize: false,
            progressive: false,
            lossless: false,
            preset: None,
            output_dir: None,
            overwrite: false,
        };

        let format = compressor.determine_output_format(&options).unwrap();
        assert!(matches!(format, ImageFormat::Jpeg));
    }
}
