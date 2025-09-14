#!/bin/bash

# CompressCLI Batch Processing Examples
# Make this file executable: chmod +x batch_script.sh

echo "CompressCLI Batch Processing Examples"
echo "====================================="

# Example 1: Process all videos in a directory with medium quality
echo "1. Processing videos with medium preset..."
compresscli batch ./input_videos \
    --videos \
    --preset medium \
    --recursive \
    --output-dir ./compressed_videos \
    --jobs 4

# Example 2: Optimize all images for web using presets
echo "2. Optimizing images for web..."
compresscli batch ./input_images \
    --images \
    --preset web \
    --max-width 1920 \
    --recursive \
    --output-dir ./web_images \
    --jobs 8

# Example 3: Process both videos and images with presets
echo "3. Processing mixed media..."
compresscli batch ./mixed_media \
    --videos \
    --images \
    --video-preset fast \
    --preset high \
    --recursive \
    --output-dir ./compressed_media \
    --jobs 6

# Example 4: High-quality video compression for archival
echo "4. High-quality archival compression..."
compresscli batch ./raw_footage \
    --videos \
    --preset veryslow \
    --pattern "*.mov" \
    --output-dir ./archived_videos \
    --jobs 2

# Example 5: Quick image thumbnails with web preset
echo "5. Creating thumbnails..."
compresscli batch ./photos \
    --images \
    --resize 400x300 \
    --preset web \
    --pattern "*.jpg" \
    --output-dir ./thumbnails \
    --jobs 12

# Example 6: Convert videos to web format
echo "6. Converting to web format..."
compresscli batch ./source_videos \
    --videos \
    --preset medium \
    --resolution 720p \
    --audio-bitrate 128k \
    --output-dir ./web_videos \
    --jobs 4

echo "Batch processing complete!"