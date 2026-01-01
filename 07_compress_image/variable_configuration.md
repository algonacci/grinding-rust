# Image Compressor Configuration & Edge Cases

This document outlines the variable configurations and potential edge cases for the image compression tool. It serves as a blueprint for extending the current functional implementation into a robust, general-purpose utility.

## 1. Core Configuration Variables

### Input Handling
- **Supported Formats**:
  - `PNG` (Lossless, Transparency)
  - `JPG` / `JPEG` (Lossy, Standard)
  - `WEBP` (Modern, High efficiency)
  - `HEIC` / `AVIF` (Next-gen, often requires specific system libraries)
- **Source Type**:
  - `Single File`: Specific path to one image.
  - `Multiple Files`: List of specific paths.
  - `Directory`: Path to a folder containing images. (Need `recursive` flag?)

### Output Handling
- **Target Format**:
  - `Same as Input`: Maintain the original extension.
  - `Convert To...`: Explicitly convert all inputs to a specific format (e.g., Force all to `WEBP`).
- **Destination**:
  - `In-place`: Overwrite original files (Dangerous, needs backup/confirmation).
  - `New Directory`: Mirror structure in a specific output folder.
  - `Suffix/Prefix`: e.g., `image_compressed.jpg`.

### Dimension & Quality Control
- **Resizing Logic**:
  - `Original`: Keep input dimensions ($W \times H$).
  - `Fixed Width`: Resize to width $W$, calculate $H$ to maintain aspect ratio.
  - `Fixed Height`: Resize to height $H$, calculate $W$ to maintain aspect ratio.
  - `Max Boundary`: Ensure image fits within $W_{max} \times H_{max}$ (e.g., thumbnail generation).
- **Quality & Compression Control**:
  - `Absolute Value`: Direct scale (usually `0-100`).
  - `Presets`: User-friendly aliases.
    - `Low` (e.g., 40-50, optimized for thumb)
    - `Medium` (e.g., 70-75, web standard)
    - `High` (e.g., 85-90, print/archive)
  - **Target File Size** (Adaptive):
    - "Compress until file is < 500KB". Requires iterative processing (binary search quality).
  - **Format Specifics**:
    - **JPG**: Quality `1-100`.
    - **PNG**: Compression Level `0-9` (Speed vs Size) OR Color Quantization (Lossy PNG, e.g., Reduce colors to 256).
    - **WEBP**: Lossy `0-100` OR Lossless mode.

## 2. Processing Modes

| Mode | Description | Complexity |
| :--- | :--- | :--- |
| **Single** | Process one file. | Low. Current implementation. |
| **Bulk (Flat)** | Process all images in a folder (no subfolders). | Medium. Needs loop. |
| **Bulk (Recursive)** | Process folder and all sub-directories. | High. Needs tree traversal and output path mirroring. |
| **Parallel** | Use multi-threading to process X images at once. | High. Great for CPU utilization on large batches. |

## 3. Edge Cases & Error Handling

### Input Anomalies
1.  **Unsupported Formats**: User provides a `.txt` or `.bmp` file.
    - *Action*: Skip and log warning.
2.  **Corrupted Files**: File exists but header is broken.
    - *Action*: Graceful failure, do not crash application, log error.
3.  **Zero-byte Files**: Empty files.
    - *Action*: Skip.
4.  **Permissions**: Read-only files or directories.
    - *Action*: Check permissions before write, fail fast for output creation.

### format-Specific Quirks
1.  **Transparency (Alpha Channel)**:
    - *Scenario*: Converting transparent PNG to JPG.
    - *Issue*: JPG doesn't support transparency. Background becomes black/white artifacts.
    - *Solution*: Detect alpha channel -> Flatten background (e.g., to white) OR force error/warning.
2.  **Animated Images** (GIF/APNG):
    - *Scenario*: Processing an animated GIF.
    - *Issue*: Most simple compressors only grab the first frame.
    - *Solution*: Explicitly skip animated formats or support multi-frame processing.
3.  **Color Profiles (CMYK)**:
    - *Scenario*: Print-ready images.
    - *Issue*: Colors look washed out when converted to RGB for web.
    - *Solution*: Convert color profile to `sRGB` during processing.

### Size Dimensions
1.  **Upscaling**:
    - *Scenario*: Requesting resize width 2000px on a 500px image.
    - *Action*: Usually generally avoided. Add flag `allow_upscale: false` by default.
2.  **Tiny Images**:
    - *Scenario*: Compressing a 1KB icon.
    - *Issue*: Headers might make output larger than input.
    - *Solution*: If `output_size > input_size`, keep original file (optional logic).

## 4. Proposed Configuration Structure (Rust Struct)

```rust
struct Config {
    // I/O
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>, // If None, maybe overwrite or use default suffix?
    pub recursive: bool,
    
    // Format
    pub target_format: Option<ImageFormat>, // Enum: Png, Jpg, WebP, Original
    
    // Dimensions
    pub resize_mode: ResizeMode, // Enum: None, Fit(w, h), Exact(w, h)
    
    // Quality
    pub quality: f32, // 0-100
    
    // Safety
    pub dry_run: bool, // Simulate without writing
    pub overwrite: bool,
}
```
