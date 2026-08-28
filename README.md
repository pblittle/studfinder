# Studfinder

[![Rust CI](https://github.com/pblittle/studfinder/actions/workflows/rust.yml/badge.svg)](https://github.com/pblittle/studfinder/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A LEGO piece identification and cataloging tool that scans images and records what it
finds in a local inventory.

## Project status

Early, and parts of the pipeline are still placeholders. What works today:

- Color identification, against six primary colors (red, green, blue, yellow, white, black)
- A local SQLite inventory, batch directory scanning, and JSON/CSV import and export

What does not work yet is **part identification**. Both processors return a hardcoded
part number (`3001`), so every scan reports the same piece regardless of the image, and
the category is always `Brick`. Shape analysis and template matching are stubs. See
[#21](https://github.com/pblittle/studfinder/issues/21).

Color is the only field you should trust today.

## Features

- **Image-based color identification**: Analyze images to identify LEGO pieces by color
- **Multiple processing strategies**: Choose between Scanner and Detector; both do color
  analysis today, and Detector is where template matching is intended to live ([#21])
- **Local inventory management**: Store and manage your LEGO collection in a local SQLite database
- **Batch directory processing**: Process multiple images at once
- **Export/import inventory**: Support for JSON and CSV formats
- **Color detection**: Identify six primary colors, named per BrickLink or LEGO official convention
- **Configurable scan quality**: Balance between speed and accuracy with Fast, Balanced, or Accurate modes
- **Robust error handling**: Comprehensive error types and context-rich error messages

## Installation

```bash
git clone https://github.com/pblittle/studfinder.git
cd studfinder
make install
```

## Quick Start

```bash
# Initialize database and config
studfinder init

# Scan a single piece
studfinder scan piece.jpg

# Batch process a directory
studfinder scan --batch path/to/pieces/

# List inventory
studfinder inventory list

# Export inventory
studfinder inventory export pieces.json

# Import inventory
studfinder inventory import pieces.json
```

## Development

### Prerequisites

- Rust 1.70+
- A C toolchain, used to build the bundled SQLite (`rusqlite`'s `bundled`
  feature compiles SQLite in, so no system SQLite install is required)

### Setup Development Environment

```bash
# Build project
make build

# Run tests
make test

# Run linter
make lint

# Format code
make format

# Fix linting issues automatically
make lint-fix

# Run comprehensive linting checks
make lint-all

# Development mode (watch)
make dev
```

### Code Quality Tools

The project uses several tools to ensure code quality:

1. **rustfmt** - Code formatter configured in `rustfmt.toml`
   - Ensures consistent code style
   - Run with `make format` to format code
   - Run with `make lint` to check formatting

2. **clippy** - Rust linter configured in `.clippy.toml`
   - Catches common mistakes and improves code quality
   - Enforces best practices
   - Run with `make lint` to check for issues
   - Run with `make lint-fix` to automatically fix issues

3. **GitHub Actions** - CI/CD pipeline in `.github/workflows/rust.yml`
   - Runs tests and linting on every push and pull request
   - Ensures code quality is maintained
   - Performs security audits

### Debug Logging

Enable verbose logging:

```bash
studfinder --verbose [command]
```

### Testing with Sample Data

```bash
# Run the test workflow against the sample image included in the repo
cargo run -- init
cargo run -- scan test_data/test.jpg
cargo run -- inventory list
```

## Architecture

Studfinder follows a modular architecture organized into three main modules:

### Module Structure

- **core**: Core domain types and traits

  - `piece.rs`: Defines the `Piece` struct and related types
  - `config.rs`: Configuration management

- **processing**: Image processing implementations

  - `processor.rs`: Defines the `ImageProcessor` trait
  - `scanner.rs`: Color-based processor implementation
  - `detector.rs`: Intended home for template matching; currently color-based ([#21])
  - `color.rs`: Color detection and analysis

- **storage**: Persistence layer
  - `database.rs`: SQLite database operations
  - `export.rs`: Import/export functionality

### Core Components

- **StudFinder**: Main application class that coordinates the other components
- **Piece**: Represents a LEGO piece with its properties (part number, color, category, etc.)
- **Config**: Application configuration

### Image Processing

The image processing system is built around the `ImageProcessor` trait, which defines a common interface for different image processing strategies:

```rust
pub trait ImageProcessor: Send + Sync {
    fn process_image(&self, image_path: &Path) -> Result<Vec<Piece>>;
    fn validate_image(&self, image: &DynamicImage) -> Result<()>;
    fn clone_box(&self) -> Box<dyn ImageProcessor>;
}
```

Two implementations are provided:

1. **Scanner**: A color-based processor that analyzes the average color of an image to identify LEGO pieces. Configurable with different quality levels (Fast, Balanced, Accurate), which set the color threshold and the minimum accepted confidence.

2. **Detector**: Intended to be a template-matching processor that uses reference images to identify piece shapes. Today it runs the same color analysis as Scanner and returns a fixed part number; the template map it builds is never consulted ([#21]). Uses a confidence threshold to determine matches.

The implementation can be selected via configuration:

```rust
// In code
let config = Config {
    processor_type: ProcessorType::Scanner, // or ProcessorType::Detector
    confidence_threshold: 0.8,
    // other config options...
};
```

### Color Detection

The `ColorDetector` component averages every pixel in the image and classifies that mean
RGB value against six primary colors: red, green, blue, yellow, white, and black. Anything
it cannot place returns `Unknown`. There is no segmentation, so a piece photographed against
a contrasting background is averaged together with that background.

Two naming conventions are available:

- **BrickLink**: Uses BrickLink's color naming convention
- **LEGO Official**: Uses LEGO's official color naming convention

The two standards currently differ only in the names returned; the underlying thresholds are
the same. Confidence scoring is based on color purity, that is, how far the dominant channel
sits above the others.

### Storage

The storage layer is divided into two main components:

1. **Database**: Manages the local inventory using SQLite with a versioned schema:

   - Version 1: Basic piece storage (id, part_number, color, category, quantity)
   - Version 2: Added confidence scoring and indexes for performance

   The database supports:

   - Adding/updating pieces
   - Retrieving pieces by ID
   - Listing all pieces
   - Updating quantities
   - Deleting pieces

2. **ExportManager**: Handles import/export operations with support for:
   - JSON format
   - CSV format

### Error Handling

Studfinder uses a comprehensive error handling approach:

- Custom error types defined with `thiserror`
- Context-rich error messages
- Proper error propagation with the `?` operator
- Specific error variants for different failure modes

Error types include:

- Database errors
- Image processing errors
- I/O errors
- Validation errors
- Configuration errors

## Contributing

Bug reports and pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md)
for how to build, test, and lint, and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
for the ground rules.

To report a security issue, follow [SECURITY.md](SECURITY.md) rather than opening
a public issue.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2025 P. Barrett Little

[#21]: https://github.com/pblittle/studfinder/issues/21
