# Sample Files for CompressCLI Testing

This directory contains sample files for testing CompressCLI functionality.

## Image Files

- **sample_4k.png** - 4K resolution image (3840x2160) for testing high-resolution compression
- **sample_1080p.jpg** - Full HD image (1920x1080) for standard compression testing
- **sample_720p.webp** - HD image (1280x720) in WebP format
- **sample_photo.jpg** - Photo-like image (2048x1536) for realistic compression testing

## Video Files

- **sample_video.mp4** - 10-second test video (1280x720) with audio
- **sample_short.mp4** - 5-second test video (854x480) for quick testing

## Usage Examples

### Test Image Compression
```bash
# Test with presets
compresscli image sample_4k.png --preset web
compresscli image sample_photo.jpg --preset high
compresscli image sample_1080p.jpg --preset lossless

# Test format conversion
compresscli image sample_4k.png --format webp --quality 80
compresscli image sample_photo.jpg --format png

# Test resizing
compresscli image sample_4k.png --resize 1920x1080 --preset web
compresscli image sample_photo.jpg --max-width 1024 --preset high
```

### Test Video Compression
```bash
# Test with presets
compresscli video sample_video.mp4 --preset fast
compresscli video sample_video.mp4 --preset slow

# Test with custom settings
compresscli video sample_video.mp4 --crf 20 --resolution 720p
compresscli video sample_short.mp4 --codec h265 --preset medium
```

### Test Batch Processing
```bash
# Process all images
compresscli batch . --images --preset web --recursive

# Process all videos
compresscli batch . --videos --preset medium --recursive

# Process both
compresscli batch . --videos --images --recursive --output-dir ../compressed
```

## File Sizes

These sample files are designed to have different characteristics:
- Large files for testing compression efficiency
- Different formats for testing conversion
- Various resolutions for testing scaling
- Both images and videos for comprehensive testing

Use these files to verify that CompressCLI works correctly with your specific use cases.
