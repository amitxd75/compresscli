use crate::cli::args::ImageFormat;
use crate::core::{CompressError, Config, DEFAULT_IMAGE_QUALITY, Result};
use crate::ui::progress::print_success;
use crate::utils::{
    calculate_compression_ratio, check_output_overwrite, ensure_parent_dir, generate_output_path,
    get_extension_lowercase, get_file_size, validate_input_file, validate_safe_path,
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
        validate_safe_path(&options.input)?;

        // Apply preset configuration if specified
        self.apply_preset_config(&mut options)?;

        // Get original file size
        let original_size = get_file_size(&options.input)?;

        // Determine output format and path
        let output_format = self.determine_output_format(&options)?;
        let output_path = self.generate_output_path(&options, &output_format)?;

        // Ensure parent directory exists
        ensure_parent_dir(&output_path)?;

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
        let mut img = image::open(&options.input).map_err(CompressError::Image)?;

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

    /// Applies preset configuration to options
    fn apply_preset_config(&self, options: &mut ImageCompressionOptions) -> Result<()> {
        if let Some(preset_name) = &options.preset {
            if let Some(preset) = self.config.get_image_preset(preset_name) {
                // Only apply preset values if the option wasn't explicitly set by the user
                // We need a better way to detect user-set vs default values
                // For now, we'll apply preset values and let CLI overrides take precedence

                // Apply preset quality only if it's still the default and not explicitly set
                if options.quality == DEFAULT_IMAGE_QUALITY {
                    options.quality = preset.quality;
                }

                // Apply other preset options if they weren't explicitly enabled
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
        Ok(())
    }

    /// Determines output format from options or input file extension
    fn determine_output_format(&self, options: &ImageCompressionOptions) -> Result<ImageFormat> {
        if let Some(format) = &options.format {
            Ok(format.clone())
        } else {
            // Try to determine from input extension
            if let Some(extension) = get_extension_lowercase(&options.input) {
                match extension.as_str() {
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

    /// Generates output path with proper naming and validation
    fn generate_output_path(
        &self,
        options: &ImageCompressionOptions,
        format: &ImageFormat,
    ) -> Result<PathBuf> {
        if let Some(output) = &options.output {
            validate_safe_path(output)?;
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

    /// Applies image transformations (resize, constraints)
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

        // Check max width constraint
        if let Some(max_width) = options.max_width
            && current_width > max_width
        {
            new_width = max_width;
            new_height = (current_height as f32 * max_width as f32 / current_width as f32) as u32;
        }

        // Check max height constraint (may override width constraint)
        if let Some(max_height) = options.max_height
            && new_height > max_height
        {
            new_height = max_height;
            new_width = (new_width as f32 * max_height as f32 / new_height as f32) as u32;
        }

        // Apply resize if dimensions changed
        if new_width != current_width || new_height != current_height {
            img = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
            debug!(
                "Resized image to fit constraints: {}x{}",
                new_width, new_height
            );
        }

        Ok(img)
    }

    /// Saves image with format-specific options
    fn save_image(
        &self,
        img: &DynamicImage,
        output_path: &Path,
        format: &ImageFormat,
        options: &ImageCompressionOptions,
    ) -> Result<()> {
        match format {
            ImageFormat::Jpeg => {
                // For JPEG, we could use more advanced encoding options
                // but the image crate has limited JPEG encoder options
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
                    "AVIF encoding not yet supported by the image crate",
                ));
            }
        }

        if self.verbose {
            debug!(
                "Saved image with quality: {}, optimize: {}, progressive: {}, lossless: {}",
                options.quality, options.optimize, options.progressive, options.lossless
            );
        }

        Ok(())
    }

    /// Parses resize dimensions from string format
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

        if width == 0 || height == 0 {
            return Err(CompressError::invalid_parameter(
                "resize",
                "Width and height must be greater than 0",
            ));
        }

        Ok((width, height))
    }

    /// Prints dry run information
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

// Make ImageCompressor cloneable for async processing
impl Clone for ImageCompressor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            dry_run: self.dry_run,
            verbose: self.verbose,
        }
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
        assert!(compressor.parse_resize_dimensions("0x600").is_err());
        assert!(compressor.parse_resize_dimensions("800x0").is_err());
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

        // Test with PNG input
        let options_png = ImageCompressionOptions {
            input: PathBuf::from("test.PNG"),
            ..options
        };
        let format_png = compressor.determine_output_format(&options_png).unwrap();
        assert!(matches!(format_png, ImageFormat::Png));
    }

    #[test]
    fn test_preset_application() {
        let config = Config::default();
        let compressor = ImageCompressor::new(config, false, false);

        let mut options = ImageCompressionOptions {
            input: PathBuf::from("test.jpg"),
            output: None,
            quality: DEFAULT_IMAGE_QUALITY, // Default quality
            format: None,
            resize: None,
            max_width: None,
            max_height: None,
            optimize: false,
            progressive: false,
            lossless: false,
            preset: Some("high".to_string()),
            output_dir: None,
            overwrite: false,
        };

        compressor.apply_preset_config(&mut options).unwrap();

        // Should have applied the "high" preset quality (95)
        assert_eq!(options.quality, 95);
        assert!(options.optimize); // Should be enabled by preset
    }
}
