#!/usr/bin/env python3
"""
Create sample images and videos for testing CompressCLI
Requires: pip install pillow opencv-python numpy
"""

import os
from PIL import Image, ImageDraw, ImageFont
import numpy as np

def create_sample_images():
    """Create sample images for testing"""
    samples_dir = "examples/samples"
    os.makedirs(samples_dir, exist_ok=True)

    # Create a large sample image (4K)
    img_4k = Image.new('RGB', (3840, 2160), color='skyblue')
    draw = ImageDraw.Draw(img_4k)

    # Add some text and shapes
    try:
        # Try to use a default font, fallback to basic if not available
        font = ImageFont.load_default()
    except:
        font = None

    # Draw some shapes and text
    draw.rectangle([100, 100, 1000, 600], fill='red', outline='black', width=5)
    draw.ellipse([2000, 500, 3500, 1500], fill='green', outline='blue', width=10)
    draw.text((200, 200), "Sample 4K Image", fill='white', font=font)
    draw.text((200, 300), "For CompressCLI Testing", fill='white', font=font)

    img_4k.save(f"{samples_dir}/sample_4k.png")
    print("Created sample_4k.png (3840x2160)")

    # Create a medium sample image (1080p)
    img_1080p = Image.new('RGB', (1920, 1080), color='lightgreen')
    draw = ImageDraw.Draw(img_1080p)

    # Add gradient effect
    for i in range(1920):
        color_val = int(255 * (i / 1920))
        draw.line([(i, 0), (i, 1080)], fill=(color_val, 100, 255 - color_val))

    draw.text((100, 100), "Sample 1080p Image", fill='white', font=font)
    draw.text((100, 150), "Gradient Background", fill='white', font=font)

    img_1080p.save(f"{samples_dir}/sample_1080p.jpg", quality=95)
    print("Created sample_1080p.jpg (1920x1080)")

    # Create a small sample image (720p)
    img_720p = Image.new('RGB', (1280, 720), color='orange')
    draw = ImageDraw.Draw(img_720p)

    # Create a pattern
    for x in range(0, 1280, 50):
        for y in range(0, 720, 50):
            color = (x % 255, y % 255, (x + y) % 255)
            draw.rectangle([x, y, x+40, y+40], fill=color)

    draw.text((50, 50), "Sample 720p Image", fill='white', font=font)
    draw.text((50, 100), "Pattern Background", fill='white', font=font)

    img_720p.save(f"{samples_dir}/sample_720p.webp", quality=90)
    print("Created sample_720p.webp (1280x720)")

    # Create a photo-like sample
    img_photo = Image.new('RGB', (2048, 1536), color='lightblue')
    draw = ImageDraw.Draw(img_photo)

    # Simulate a landscape
    # Sky
    for y in range(500):
        color_val = int(135 + (y / 500) * 120)
        draw.line([(0, y), (2048, y)], fill=(color_val, color_val + 20, 255))

    # Ground
    for y in range(500, 1536):
        color_val = int(34 + ((y - 500) / 1036) * 100)
        draw.line([(0, y), (2048, y)], fill=(color_val, color_val + 50, color_val))

    # Add some "mountains"
    points = [(0, 500), (300, 200), (600, 350), (900, 150), (1200, 300), (1500, 100), (1800, 250), (2048, 180), (2048, 500)]
    draw.polygon(points, fill='gray')

    draw.text((100, 1400), "Sample Photo-like Image", fill='white', font=font)
    draw.text((100, 1450), "Landscape Simulation", fill='white', font=font)

    img_photo.save(f"{samples_dir}/sample_photo.jpg", quality=85)
    print("Created sample_photo.jpg (2048x1536)")

def create_sample_video():
    """Create a sample video for testing"""
    try:
        import cv2

        samples_dir = "examples/samples"

        # Video properties
        width, height = 1280, 720
        fps = 30
        duration = 10  # seconds
        total_frames = fps * duration

        # Create video writer
        fourcc = cv2.VideoWriter_fourcc(*'mp4v')
        out = cv2.VideoWriter(f'{samples_dir}/sample_video.mp4', fourcc, fps, (width, height))

        for frame_num in range(total_frames):
            # Create a frame with changing colors
            frame = np.zeros((height, width, 3), dtype=np.uint8)

            # Create a moving gradient
            for y in range(height):
                for x in range(width):
                    r = int(128 + 127 * np.sin(2 * np.pi * frame_num / 60 + x / 100))
                    g = int(128 + 127 * np.sin(2 * np.pi * frame_num / 60 + y / 100))
                    b = int(128 + 127 * np.sin(2 * np.pi * frame_num / 60 + (x + y) / 200))
                    frame[y, x] = [b, g, r]  # OpenCV uses BGR

            # Add frame counter text
            cv2.putText(frame, f'Frame {frame_num + 1}/{total_frames}',
                       (50, 50), cv2.FONT_HERSHEY_SIMPLEX, 1, (255, 255, 255), 2)
            cv2.putText(frame, f'Time: {frame_num/fps:.1f}s',
                       (50, 100), cv2.FONT_HERSHEY_SIMPLEX, 1, (255, 255, 255), 2)
            cv2.putText(frame, 'Sample Video for CompressCLI',
                       (50, height - 50), cv2.FONT_HERSHEY_SIMPLEX, 1, (255, 255, 255), 2)

            out.write(frame)

        out.release()
        print(f"Created sample_video.mp4 ({width}x{height}, {duration}s)")

    except ImportError:
        print("OpenCV not available, skipping video creation")
        print("To create sample video, install: pip install opencv-python numpy")

if __name__ == "__main__":
    print("Creating sample files for CompressCLI testing...")
    create_sample_images()
    create_sample_video()
    print("\nSample files created in examples/samples/")
    print("You can now test CompressCLI with these files!")