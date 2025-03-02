# Studfinder

A vision-based LEGO piece identification and cataloging tool that scans and identifies LEGO pieces using computer vision.

## Features

- **Image-based LEGO piece identification**: Analyze images to identify LEGO pieces by color and shape
- **Multiple processing strategies**: Choose between Scanner (color-based) and Detector (template matching) approaches
- **Local inventory management**: Store and manage your LEGO collection in a local SQLite database
- **Batch directory processing**: Process multiple images at once
- **Export/import inventory**: Support for JSON and CSV formats
- **Color detection**: Identify LEGO colors with configurable standards (BrickLink or LEGO official)
- **Configurable scan quality**: Balance between speed and accuracy with Fast, Balanced, or Accurate modes
- **Robust error handling**: Comprehensive error types and context-rich error messages

## Installation

```bash
git clone https://github.com/yourusername/studfinder.git
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
- SQLite3
- ImageMagick (for tests)

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
# Create test directory
mkdir test_data

# Create test image
magick convert -size 200x200 xc:red test_data/test.jpg

# Run test workflow
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
  - `detector.rs`: Template-matching processor implementation
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

1. **Scanner**: A color-based processor that analyzes the dominant colors in an image to identify LEGO pieces. Configurable with different quality levels (Fast, Balanced, Accurate).

2. **Detector**: A template-matching processor that uses reference images to identify specific LEGO piece shapes. Uses a confidence threshold to determine matches.

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

The `ColorDetector` component provides color analysis with support for different color standards:

- **BrickLink**: Uses BrickLink's color naming convention
- **LEGO Official**: Uses LEGO's official color naming convention

Color detection includes confidence scoring based on color purity and matching against known LEGO colors.

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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2025 P. Barrett Little
