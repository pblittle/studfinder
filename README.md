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

# Development mode (watch)
make dev
```

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

Studfinder follows a modular architecture with the following components:

### Core Components

- **StudFinder**: Main application class that coordinates the other components
- **ImageProcessor**: Trait defining the interface for image processing strategies
- **Scanner**: Color-based processor that analyzes dominant colors in images
- **Detector**: Template-matching processor that uses reference images for identification
- **ColorDetector**: Analyzes images to determine predominant colors
- **Database**: Manages the local inventory using SQLite

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

### Database

The database layer uses SQLite with a versioned schema:

- Version 1: Basic piece storage (id, part_number, color, category, quantity)
- Version 2: Added confidence scoring and indexes for performance

The database supports:
- Adding/updating pieces
- Retrieving pieces by ID
- Listing all pieces
- Updating quantities
- Deleting pieces

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

Contributions are welcome! Here are some ways you can contribute:

- Improve the image processing algorithms
- Add support for more LEGO piece types
- Enhance the color detection system
- Optimize performance
- Add new features

Please follow these guidelines:
- Write clean, maintainable code
- Include appropriate tests
- Follow Rust best practices
- Document your code

## License

MIT
