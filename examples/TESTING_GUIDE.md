# CompressCLI Testing Guide

This guide provides comprehensive examples for testing all features of CompressCLI using the provided sample files.

## Prerequisites

1. **Build CompressCLI**: `cargo build --release`
2. **Create Sample Files**: `bash examples/create_samples.sh`
3. **Install FFmpeg**: Required for video compression

## Quick Start Tests

### Test System Information
```bash
# Check if everything is working
compresscli info

# List available presets
compresscli presets list
```

## Image Compression Tests

### Basic Image Compression
```bash
cd examples/samples

# Test with default settings
compresscli image sample_photo.jpg

# Test with different quality levels
compresscli image sample_photo.jpg --quality 50
compresscli image sample_photo.jpg --quality 95
```

### Image Preset Tests
```bash
# Test all built-in presets
compresscli image sample_4k.png --preset web
compresscli image sample_4k.png --preset high
compresscli image sample_4k.png --preset lossless

# Compare file sizes
ls -lh sample_4k*
```

### Format Conversion Tests
```bash
# Convert PNG to JPEG
compresscli image sample_4k.png --format jpeg --quality 85

# Convert JPEG to WebP
compresscli image sample_photo.jpg --format webp --quality 80

# Convert to PNG (lossless)
compresscli image sample_1080p.jpg --format png
```

### Resize Tests
```bash
# Resize to specific dimensions
compresscli image sample_4k.png --resize 1920x1080 --preset web

# Resize with max width constraint
compresscli image sample_4k.png --max-width 1920 --preset high

# Resize with max height constraint
compresscli image sample_photo.jpg --max-height 1080 --preset web
```

### Advanced Image Tests
```bash
# Combine preset with custom options
compresscli image sample_4k.png --preset web --resize 1920x1080

# Progressive JPEG
compresscli image sample_photo.jpg --progressive --quality 90

# Optimized compression
compresscli image sample_1080p.jpg --optimize --quality 85
```

## Video Compression Tests

### Basic Video Compression
```bash
# Test with default settings
compresscli video sample_video.mp4

# Test with different presets
compresscli video sample_video.mp4 --preset fast
compresscli video sample_video.mp4 --preset slow
```

### Video Quality Tests
```bash
# Test CRF values
compresscli video sample_video.mp4 --crf 18  # High quality
compresscli video sample_video.mp4 --crf 28  # Lower quality

# Test bitrate targeting
compresscli video sample_video.mp4 --bitrate 1M
compresscli video sample_video.mp4 --bitrate 500K
```

### Video Codec Tests
```bash
# Test different codecs
compresscli video sample_video.mp4 --codec h264 --preset medium
compresscli video sample_video.mp4 --codec h265 --preset slow

# Note: VP9 and AV1 require specific FFmpeg builds
# compresscli video sample_video.mp4 --codec vp9 --preset medium
```

### Video Resolution Tests
```bash
# Resize video
compresscli video sample_video.mp4 --resolution 720p
compresscli video sample_video.mp4 --resolution 480p
compresscli video sample_video.mp4 --resolution 1920x1080
```

### Video Trimming Tests
```bash
# Trim video segments
compresscli video sample_video.mp4 --start 00:00:02 --end 00:00:05
compresscli video sample_video.mp4 --start 2 --end 8  # Using seconds
```

### Audio Processing Tests
```bash
# Different audio codecs
compresscli video sample_video.mp4 --audio-codec aac --audio-bitrate 128k
compresscli video sample_video.mp4 --audio-codec mp3 --audio-bitrate 192k

# Remove audio
compresscli video sample_video.mp4 --no-audio
```

### Advanced Video Tests
```bash
# Two-pass encoding
compresscli video sample_video.mp4 --bitrate 1M --two-pass

# Complex combination
compresscli video sample_video.mp4 \\
    --preset slow \\
    --crf 20 \\
    --resolution 720p \\
    --audio-bitrate 128k
```

## Batch Processing Tests

### Image Batch Processing
```bash
# Process all images with web preset
compresscli batch . --images --preset web

# Process with pattern matching
compresscli batch . --images --pattern "*.jpg" --preset high

# Process with output directory
compresscli batch . --images --preset web --output-dir ../compressed_images
```

### Video Batch Processing
```bash
# Process all videos
compresscli batch . --videos --preset medium

# Process with specific pattern
compresscli batch . --videos --pattern "*.mp4" --preset fast

# Process with parallel jobs
compresscli batch . --videos --preset medium --jobs 2
```

### Mixed Batch Processing
```bash
# Process both images and videos
compresscli batch . --videos --images --preset medium

# Recursive processing
compresscli batch ../samples --videos --images --recursive --output-dir ../all_compressed
```

## Preset Management Tests

### List and Show Presets
```bash
# List all presets
compresscli presets list

# Show specific preset details
compresscli presets show web
compresscli presets show medium
```

### Custom Preset Tests
```bash
# Create custom image preset
compresscli presets create social_media examples/custom_presets.yaml

# Create custom video preset
compresscli presets create streaming examples/custom_video_preset.yaml

# Use custom preset
compresscli image sample_photo.jpg --preset social_media

# Delete custom preset
compresscli presets delete social_media
```

## Dry Run Tests

### Preview Operations
```bash
# Preview image compression
compresscli image sample_4k.png --preset web --dry-run

# Preview video compression
compresscli video sample_video.mp4 --preset slow --dry-run

# Preview batch processing
compresscli batch . --images --preset web --dry-run
```

## Error Handling Tests

### Test Error Conditions
```bash
# Non-existent file
compresscli image nonexistent.jpg

# Invalid preset
compresscli image sample_photo.jpg --preset invalid

# Invalid parameters
compresscli image sample_photo.jpg --quality 150
compresscli video sample_video.mp4 --crf 60
```

## Performance Tests

### Measure Compression Efficiency
```bash
# Compare original and compressed sizes
ls -lh sample_4k.png
compresscli image sample_4k.png --preset web
ls -lh sample_4k_compressed_web.jpg

# Time compression operations
time compresscli video sample_video.mp4 --preset fast
time compresscli video sample_video.mp4 --preset slow
```

### Parallel Processing Tests
```bash
# Test different job counts
compresscli batch . --images --preset web --jobs 1
compresscli batch . --images --preset web --jobs 4
compresscli batch . --images --preset web --jobs 8
```

## Integration Tests

### Complete Workflow Test
```bash
#!/bin/bash
# Complete workflow test script

echo "=== CompressCLI Integration Test ==="

# 1. Check system
echo "1. Checking system..."
compresscli info

# 2. Test image compression
echo "2. Testing image compression..."
compresscli image sample_photo.jpg --preset web --output-dir test_output

# 3. Test video compression
echo "3. Testing video compression..."
compresscli video sample_video.mp4 --preset medium --output-dir test_output

# 4. Test batch processing
echo "4. Testing batch processing..."
compresscli batch . --images --preset high --output-dir test_output --pattern "sample_*"

# 5. Verify outputs
echo "5. Verifying outputs..."
ls -la test_output/

echo "=== Integration Test Complete ==="
```

## Troubleshooting Tests

### Common Issues
```bash
# Test with verbose output
compresscli video sample_video.mp4 --preset medium --verbose

# Test overwrite behavior
compresscli image sample_photo.jpg --preset web
compresscli image sample_photo.jpg --preset web --overwrite

# Test with custom output directory
mkdir -p test_output
compresscli image sample_photo.jpg --preset web --output-dir test_output
```

## Cleanup

### Remove Test Files
```bash
# Remove generated files
rm -f sample_*_compressed_*
rm -f sample_*_web.*
rm -f sample_*_high.*
rm -f sample_*_lossless.*
rm -rf test_output/

# Or use a cleanup script
find . -name "*_compressed_*" -delete
find . -name "*_web.*" -delete
find . -name "*_high.*" -delete
```

## Expected Results

### File Size Reductions
- **Web preset**: 60-80% size reduction for images
- **High preset**: 40-60% size reduction for images
- **Video compression**: 30-70% depending on preset and content

### Quality Expectations
- **Web preset**: Good quality for web use
- **High preset**: Excellent quality for professional use
- **Lossless preset**: No quality loss, minimal size reduction

### Performance Expectations
- **Image compression**: Near-instant for small images, seconds for 4K
- **Video compression**: Real-time or faster for most presets
- **Batch processing**: Scales with number of CPU cores

Use this guide to thoroughly test CompressCLI and verify all functionality works as expected in your environment.