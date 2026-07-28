# CompressCLI

A high-performance FFmpeg wrapper for video and image compression, written in Rust. CompressCLI streamlines encoding workflows with an interactive wizard, GPU-accelerated batch processing, and smart caching — with sensible defaults for both quick one-off conversions and large-scale pipelines.

## Features

- **Interactive Mode** — Run `compresscli` with no arguments for a guided, step-by-step wizard.
- **Auto Detection** — `compresscli auto <path>` routes files or directories to the correct pipeline automatically.
- **Performance & Caching** — Signature-based caching skips unchanged files; CPU-bound work is offloaded via `spawn_blocking`.
- **GPU Acceleration** — Hardware encoding via NVIDIA NVENC, Apple VideoToolbox, Intel QSV, AMD AMF, and Linux VAAPI (`--gpu`), with automatic fallback to software encoding.
- **Video Compression** — H.264, H.265, VP9, and AV1 codecs, with format conversion across `.mp4`, `.mkv`, and `.webm`.
- **Image Compression** — JPEG, PNG, WebP, and AVIF optimization with fine-grained quality and format control.
- **Batch Processing** — Recursive, parallelized processing across entire directories.
- **Presets** — Built-in quality tiers (`ultrafast` → `veryslow`) plus fully custom presets via config.
- **Progress Tracking** — Real-time progress bars, spinners, and compression statistics.
- **Dry Run** — Preview any operation before touching a file.

## Installation

### Cargo

```bash
cargo install compresscli
```

### Prebuilt Installers (Recommended)

Available from [GitHub Releases](https://github.com/amitxd75/compresscli/releases):

| Platform | Method |
|---|---|
| Windows | `.msi` installer with FFmpeg bundled — no separate install required |
| Linux | Download tarball, extract, run `./install.sh` |
| macOS | Download tarball, extract, run `./install.sh` |

### Build from Source

```bash
git clone https://github.com/amitxd75/compresscli.git
cd compresscli
cargo build --release
```

### Prerequisites (Linux / macOS / Source Builds)

FFmpeg is required (bundled automatically in Windows MSI releases):

- Ubuntu/Debian: `sudo apt install ffmpeg`
- macOS: `brew install ffmpeg`
- Windows: `winget install -e --id Gyan.FFmpeg.Essentials`, or download from [ffmpeg.org](https://ffmpeg.org/download.html)

## Shell Autocompletion

Interactive path filtering and tab completion are supported on all major shells.

### Option A: Using the Automated Setup Script (Recommended for Repository Clones)

```bash
# Linux / macOS (Bash, Zsh, Fish)
./scripts/setup-completions.sh

# Windows (PowerShell)
.\scripts\setup-completions.ps1
```

### Option B: Direct CLI Generation (Recommended for `cargo install`)

If you installed via `cargo install`, generate autocompletions directly using the built-in `completions` subcommand:

```bash
# Bash
mkdir -p ~/.local/share/bash-completion/completions
compresscli completions bash > ~/.local/share/bash-completion/completions/compresscli

# Zsh
mkdir -p ~/.local/share/zsh/site-functions
compresscli completions zsh > ~/.local/share/zsh/site-functions/_compresscli
# Ensure ~/.local/share/zsh/site-functions is in your fpath:
echo 'fpath=(~/.local/share/zsh/site-functions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc

# Fish
mkdir -p ~/.config/fish/completions
compresscli completions fish > ~/.config/fish/completions/compresscli.fish

# PowerShell
compresscli completions powershell >> $PROFILE
```

## Quick Start

```bash
# Interactive wizard
compresscli

# Auto-detect file type
compresscli auto photo.png
compresscli auto video.mp4
compresscli auto ./media_folder

# Format conversion
compresscli image photo.png --format webp --quality 80
compresscli image photo.png --format jpeg --quality 85

# Video compression with GPU acceleration
compresscli video input.mp4 --preset medium --gpu
compresscli video input.mp4 --codec h265 --crf 20

# Batch processing
compresscli batch ./videos --videos --video-preset medium --recursive
```

## Command Reference

### Video Options

| Option | Description | Example |
|---|---|---|
| `--preset` | Compression preset | `fast`, `medium`, `slow` |
| `--codec` | Video codec | `h264`, `h265`, `vp9`, `av1` |
| `--crf` | Constant Rate Factor (0–51) | `--crf 23` |
| `--bitrate` | Target bitrate | `--bitrate 2M` |
| `--resolution` | Target resolution | `--resolution 1920x1080` |
| `--fps` | Target frame rate | `--fps 30` |
| `--audio-codec` | Audio codec | `aac`, `mp3`, `opus` |
| `--no-audio` | Remove audio track | |
| `--start` | Trim start time | `--start 00:01:30` |
| `--end` | Trim end time | `--end 00:05:00` |
| `--two-pass` | Enable two-pass encoding | |

### Image Options

| Option | Description | Example |
|---|---|---|
| `--preset` | Image preset | `web`, `high`, `lossless` |
| `--quality` | Image quality (1–100) | `--quality 85` |
| `--format` | Output format | `jpeg`, `png`, `webp`, `avif` |
| `--resize` | Resize to dimensions | `--resize 1920x1080` |
| `--max-width` | Maximum width | `--max-width 1920` |
| `--max-height` | Maximum height | `--max-height 1080` |
| `--optimize` | Enable optimization | |
| `--progressive` | Progressive JPEG | |
| `--lossless` | Lossless compression | |

### Batch Options

| Option | Description | Example |
|---|---|---|
| `--videos` | Process video files | |
| `--images` | Process image files | |
| `--recursive` | Recurse into subdirectories | |
| `--video-preset` | Video compression preset | `fast`, `medium`, `slow` |
| `--image-quality` | Image quality (1–100) | `--image-quality 85` |
| `--jobs` | Parallel job count (min. 1) | `--jobs 4` |
| `--pattern` | Filename pattern match | `--pattern "*.mp4"` |

### Global Options

| Option | Description |
|---|---|
| `--output-dir` | Output directory |
| `--overwrite` | Overwrite existing files |
| `--gpu` | Enable GPU acceleration (auto-detect) |
| `--hwaccel <mode>` | Specific acceleration mode: `auto`, `nvidia`, `apple`, `intel`, `amd`, `vaapi`, `disabled` |
| `--no-cache` | Disable file signature caching |
| `--dry-run` | Preview without executing |
| `--verbose` | Verbose logging |

### Other Commands

| Command | Description |
|---|---|
| `interactive` | Launch the step-by-step wizard |
| `info` | Show system info and dependency status |
| `presets list` | List all available presets |
| `presets show <name>` | Show configuration for a specific preset |
| `completions <shell>` | Generate shell completion scripts |

## Configuration

CompressCLI reads a YAML configuration file from:

| Platform | Path |
|---|---|
| Linux | `~/.config/compresscli/config.yaml` |
| macOS | `~/Library/Application Support/compresscli/config.yaml` |
| Windows | `%APPDATA%\compresscli\config.yaml` |

### Example

```yaml
video_presets:
  custom_high:
    codec: H265
    crf: 18
    bitrate: null          # required field; null means CRF-driven encoding
    audio_codec: Aac
    audio_bitrate: "256k"
    preset: "slow"
    two_pass: true
    extra_args: []         # avoid -i, -vf, and URLs — see examples/ for security notes

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
  max_fps: 240.0           # upper bound enforced during validation
  gpu_hwaccel: null        # default GPU mode (auto, nvidia, apple, intel, amd, vaapi, disabled, null)
```

## Presets

### Video

| Preset | CRF | Notes |
|---|---|---|
| `ultrafast` | 28 | Fastest, largest output |
| `fast` | 25 | Good for quick turnaround |
| `medium` | 23 | Balanced — default |
| `slow` | 20 | Smaller files, slower encode |
| `veryslow` | 18 | Best compression, H.265 |
| `custom` | — | User-defined, via `config.yaml` |

### Image

| Preset | Quality | Notes |
|---|---|---|
| `web` | 85 | Optimized for web delivery |
| `high` | 95 | High fidelity |
| `lossless` | 100 | Maximum quality |

> **Note:** Animated GIFs are not supported. Multi-frame GIFs are rejected outright to prevent silent frame loss — convert to a video format (WebM/MP4) first.

## Usage Tips

- Run `compresscli` with no arguments for guided prompts.
- Use `--dry-run` before any destructive batch operation.
- Use `--jobs N` to tune parallelism for available CPU/GPU resources.
- Run `compresscli info` to verify FFmpeg and GPU acceleration availability.

## Troubleshooting

| Issue | Resolution |
|---|---|
| FFmpeg not found | Linux/macOS: `sudo apt install ffmpeg` or `brew install ffmpeg`. Windows: use the `.msi` installer, which bundles FFmpeg. |
| Permission errors | Use `--overwrite`, or redirect output with `--output-dir ./output`. |
| Memory pressure | Reduce `--jobs` or switch to a faster preset. |
| FPS value rejected | Default `max_fps` is `240.0`. Raise it in `config.yaml` under `default_settings` if needed. |
| Output file larger than input | Expected for already-compressed sources. CompressCLI reports the increase explicitly (e.g. "file increased by 3%") rather than a misleading reduction figure. |

## License

Licensed under the MIT License — see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Rust](https://www.rust-lang.org/) — high-performance systems programming language
- [FFmpeg](https://ffmpeg.org/) — video/audio processing
- [clap](https://github.com/clap-rs/clap) — CLI parsing
- [indicatif](https://github.com/console-rs/indicatif) — progress bars and spinners
- [Tokio](https://tokio.rs/) — async runtime
- [thiserror](https://github.com/dtolnay/thiserror) — error handling
- [image](https://github.com/image-rs/image) — image processing
