#!/bin/bash

# Create sample images and videos for testing CompressCLI
# Requires: ImageMagick (convert command) and FFmpeg

SAMPLES_DIR="examples/samples"
mkdir -p "$SAMPLES_DIR"

echo "Creating sample files for CompressCLI testing..."

# Check if ImageMagick is available
if command -v convert &> /dev/null; then
    echo "Creating sample images with ImageMagick..."

    # Create a 4K sample image
    convert -size 3840x2160 gradient:blue-red \
        -pointsize 72 -fill white -gravity center \
        -annotate +0-200 "Sample 4K Image" \
        -annotate +0-100 "For CompressCLI Testing" \
        -annotate +0+0 "3840x2160 Resolution" \
        "$SAMPLES_DIR/sample_4k.png"
    echo "Created sample_4k.png (3840x2160)"

    # Create a 1080p sample image
    convert -size 1920x1080 gradient:green-yellow \
        -pointsize 48 -fill black -gravity center \
        -annotate +0-100 "Sample 1080p Image" \
        -annotate +0-50 "Gradient Background" \
        -annotate +0+0 "1920x1080 Resolution" \
        "$SAMPLES_DIR/sample_1080p.jpg"
    echo "Created sample_1080p.jpg (1920x1080)"

    # Create a 720p sample image
    convert -size 1280x720 plasma:fractal \
        -pointsize 36 -fill white -gravity center \
        -annotate +0-50 "Sample 720p Image" \
        -annotate +0+0 "Plasma Pattern" \
        -annotate +0+50 "1280x720 Resolution" \
        "$SAMPLES_DIR/sample_720p.webp"
    echo "Created sample_720p.webp (1280x720)"

    # Create a photo-like sample
    convert -size 2048x1536 gradient:skyblue-lightgreen \
        -pointsize 42 -fill darkblue -gravity center \
        -annotate +0-100 "Sample Photo-like Image" \
        -annotate +0-50 "Landscape Simulation" \
        -annotate +0+0 "2048x1536 Resolution" \
        -annotate +0+50 "Perfect for testing compression" \
        "$SAMPLES_DIR/sample_photo.jpg"
    echo "Created sample_photo.jpg (2048x1536)"

else
    echo "ImageMagick not found. Creating simple sample images with basic tools..."

    # Create simple colored images using printf and convert to images
    # This is a fallback method that works on most systems

    # Create a simple test pattern file
    cat > "$SAMPLES_DIR/sample_text.txt" << EOF
This is a sample text file that can be converted to an image.
CompressCLI Testing Sample
Resolution: Various sizes available
Format: Multiple formats supported
Quality: Adjustable compression levels

Use this for testing:
- Image compression with different presets
- Format conversion (PNG, JPEG, WebP)
- Resize operations
- Quality adjustments
- Batch processing

Sample created for CompressCLI project.
EOF

    echo "Created sample text file (can be used for testing)"
fi

# Check if FFmpeg is available for video creation
if command -v ffmpeg &> /dev/null; then
    echo "Creating sample video with FFmpeg..."

    # Create a 10-second test video with color bars and timer
    if ffmpeg -f lavfi -i testsrc2=duration=10:size=1280x720:rate=30 \
        -f lavfi -i sine=frequency=1000:duration=10 \
        -c:v libx264 -preset fast -crf 23 \
        -c:a aac -b:a 128k \
        -y "$SAMPLES_DIR/sample_video.mp4" 2>/dev/null; then
        echo "Created sample_video.mp4 (1280x720, 10s)"
    else
        echo "Failed to create sample video"
    fi

    # Create a shorter sample for quick testing
    if ffmpeg -f lavfi -i testsrc=duration=5:size=854x480:rate=25 \
        -f lavfi -i sine=frequency=800:duration=5 \
        -c:v libx264 -preset ultrafast -crf 28 \
        -c:a aac -b:a 96k \
        -y "$SAMPLES_DIR/sample_short.mp4" 2>/dev/null; then
        echo "Created sample_short.mp4 (854x480, 5s)"
    fi

else
    echo "FFmpeg not found. Skipping video creation."
    echo "Install FFmpeg to create sample videos."
fi

# Create a README for the samples
cat > "$SAMPLES_DIR/README.md" << EOF
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
\`\`\`bash
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
\`\`\`

### Test Video Compression
\`\`\`bash
# Test with presets
compresscli video sample_video.mp4 --preset fast
compresscli video sample_video.mp4 --preset slow

# Test with custom settings
compresscli video sample_video.mp4 --crf 20 --resolution 720p
compresscli video sample_short.mp4 --codec h265 --preset medium
\`\`\`

### Test Batch Processing
\`\`\`bash
# Process all images
compresscli batch . --images --preset web --recursive

# Process all videos
compresscli batch . --videos --preset medium --recursive

# Process both
compresscli batch . --videos --images --recursive --output-dir ../compressed
\`\`\`

## File Sizes

These sample files are designed to have different characteristics:
- Large files for testing compression efficiency
- Different formats for testing conversion
- Various resolutions for testing scaling
- Both images and videos for comprehensive testing

Use these files to verify that CompressCLI works correctly with your specific use cases.
EOF

echo ""
echo "Sample files created in $SAMPLES_DIR/"
echo "See $SAMPLES_DIR/README.md for usage examples"
echo ""
echo "To test CompressCLI with these samples:"
echo "  cd examples/samples"
echo "  compresscli image sample_photo.jpg --preset web"
echo "  compresscli video sample_video.mp4 --preset medium"