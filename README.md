# CompressCLI

A powerful FFmpeg wrapper CLI built in Rust. Simplifies complex video and image compression and provides batch processing capabilities.

## Features

- 🎥 **Video Compression**: Support for H.264, H.265, VP9, and AV1 codecs
- 🖼️ **Image Compression**: JPEG, PNG, WebP optimization with quality control
- 📦 **Batch Processing**: Process entire directories with parallel execution
- ⚡ **Performance**: Multi-threaded processing with progress tracking
- 🎛️ **Presets**: Built-in quality presets (ultrafast, fast, medium, slow, veryslow)
- 🔧 **Customizable**: Extensive command-line options and configuration files
- 📊 **Progress Tracking**: Real-time progress bars and compression statistics
- 🔍 **Dry Run**: Preview operations without modifying files

## Installation

### Install from Cargo

```bash
cargo install compresscli
```

### Prebuilt Binaries (Recommended)

Download from [GitHub Releases](https://github.com/amitxd75/compresscli/releases):

- **Windows**: Download and run the MSI installer
- **Linux**: Download tarball, extract, and run `./install.sh`
- **macOS**: Download tarball, extract, and run `./install.sh`

### Build from Source

```bash
git clone https://github.com/amitxd75/compresscli.git
cd compresscli
cargo build --release
```

### Prerequisites

- **FFmpeg**: Required for video compression
  - Ubuntu/Debian: `sudo apt install ffmpeg`
  - macOS: `brew install ffmpeg`
  - Windows: Download from [ffmpeg.org](https://ffmpeg.org/download.html)

## Shell Autocompletion

Only for linux
Tab completion for Bash, Zsh, Fish, and PowerShell:

```bash
./scripts/setup-completions.sh
```

## Quick Start

```bash
# Video compression
compresscli video input.mp4 --preset medium
compresscli video input.mp4 --codec h265 --crf 20

# Image compression
compresscli image photo.jpg --preset web
compresscli image photo.png --format webp --quality 80

# Batch processing
compresscli batch ./videos --videos --preset medium --recursive
compresscli batch ./photos --images --preset web
```

## Command Reference

### Video Options

| Option | Description | Example |
|--------|-------------|---------|
| `--preset` | Compression preset | `fast`, `medium`, `slow` |
| `--codec` | Video codec | `h264`, `h265`, `vp9`, `av1` |
| `--crf` | Constant Rate Factor (0-51) | `--crf 23` |
| `--bitrate` | Target bitrate | `--bitrate 2M` |
| `--resolution` | Target resolution | `--resolution 1920x1080` |
| `--fps` | Target framerate | `--fps 30` |
| `--audio-codec` | Audio codec | `aac`, `mp3`, `opus` |
| `--no-audio` | Remove audio track | |
| `--start` | Start time for trimming | `--start 00:01:30` |
| `--end` | End time for trimming | `--end 00:05:00` |
| `--two-pass` | Enable two-pass encoding | |

### Image Options

| Option | Description | Example |
|--------|-------------|---------|
| `--preset` | Image preset | `web`, `high`, `lossless` |
| `--quality` | Image quality (1-100) | `--quality 85` |
| `--format` | Output format | `jpeg`, `png`, `webp` |
| `--resize` | Resize to dimensions | `--resize 1920x1080` |
| `--max-width` | Maximum width | `--max-width 1920` |
| `--max-height` | Maximum height | `--max-height 1080` |
| `--optimize` | Enable optimization | |
| `--progressive` | Progressive JPEG | |
| `--lossless` | Lossless compression | |

### Global Options

| Option | Description |
|--------|-------------|
| `--output-dir` | Output directory |
| `--overwrite` | Overwrite existing files |
| `--dry-run` | Preview without executing |
| `--verbose` | Verbose output |
| `--jobs` | Parallel jobs (batch mode) |

### Other Commands

| Command | Description |
|---------|-------------|
| `info` | Show system information and dependencies |
| `presets list` | List all available presets |
| `presets show <name>` | Show details of a specific preset |
| `completions <shell>` | Generate shell completion scripts |

## Configuration

CompressCLI uses YAML configuration files located at:
- Linux: `~/.config/compresscli/config.yaml`
- macOS: `~/Library/Application Support/compresscli/config.yaml`
- Windows: `%APPDATA%\compresscli\config.yaml`

### Example Configuration

```yaml
video_presets:
  custom_high:
    codec: H265
    crf: 18
    audio_codec: Aac
    audio_bitrate: "256k"
    preset: "slow"
    two_pass: true
    extra_args: []

image_presets:
  web_optimized:
    quality: 85
    optimize: true
    progressive: true
    lossless: false

default_settings:
  parallel_jobs: 4
  preserve_metadata: true
  backup_originals: false
```

## Presets

### Video Presets

- **ultrafast**: Fastest compression, larger files (CRF 28)
- **fast**: Fast compression, good for quick processing (CRF 25)
- **medium**: Balanced quality and speed (CRF 23) - Default
- **slow**: Better compression, smaller files (CRF 20)
- **veryslow**: Best compression, H.265 codec (CRF 18)

### Image Presets

- **web**: Optimized for web use (Quality 85)
- **high**: High quality (Quality 95)
- **lossless**: Maximum quality (Quality 100)

## Usage Tips

- Use `--preset` for quick compression: `fast`, `medium`, `slow`
- Use `--dry-run` to preview operations
- Use `--jobs N` for parallel processing
- Use `compresscli info` to check FFmpeg installation

## Troubleshooting

**FFmpeg not found**: Install with `sudo apt install ffmpeg` (Linux) or `brew install ffmpeg` (macOS)

**Permission errors**: Use `--overwrite` or `--output-dir ./output`

**Memory issues**: Reduce `--jobs` or use faster presets

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run tests: `cargo test`
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [FFmpeg](https://ffmpeg.org/) for video processing
- CLI powered by [clap](https://github.com/clap-rs/clap)
- Progress bars by [indicatif](https://github.com/console-rs/indicatif)
