# UnityPackage Extractor (Rust Version)

A blazing fast, standalone tool to extract `.unitypackage` files without needing the Unity Editor.

This project is a high-performance port of the original Python script [unitypackage_extractor by Cobertos](https://github.com/Cobertos/unitypackage_extractor). By rewriting it in **Rust**, this tool offers significant speed improvements and requires no external runtime dependencies (like Python) once compiled.

## Why Rust?

While the original Python script is excellent, porting it to Rust provides several advantages:

* **Performance**: Rust handles file I/O and decompression much faster, making extraction of large packages significantly quicker.
* **Standalone Binary**: No need to install Python, `pip`, or manage virtual environments. You just need the single executable file.
* **Safety**: Includes built-in protection against path traversal attacks (Zip Slip vulnerability) and safely handles filename sanitization for Windows.

## Installation

### Option 1: Build from Source

You need to have the [Rust toolchain](https://www.rust-lang.org/tools/install) installed.

1. Clone this repository:

    ```bash
    git clone https://github.com/ShinuToki/unitypackage_extractor.git
    cd unitypackage_extractor
    ```

2. Build the release binary:

    ```bash
    cargo build --release
    ```

3. The executable will be located at:
    * **Windows**: `target\release\unitypackage_extractor.exe`
    * **Linux/macOS**: `target/release/unitypackage_extractor`

## Usage

Run the tool from the command line (Terminal or CMD/PowerShell).

### Basic Syntax

```bash
unitypackage_extractor <file.unitypackage> [output_path]
```

### Examples

**Extract to the current directory:**

```bash
./unitypackage_extractor MyAssets.unitypackage
```

**Extract to a specific folder:**

```bash
./unitypackage_extractor MyAssets.unitypackage ./ExtractedAssets
```

### Options

* `-h, --help`: Displays the help message and usage instructions.

## How it Works

1. The tool creates a temporary directory.
2. It decompresses the `.unitypackage` (which is essentially a `tar.gz` archive) into the temp folder.
3. It iterates through the unique hashed directories inside the archive.
4. It reads the `pathname` file to determine where the file should go (e.g., `Assets/Scripts/Player.cs`).
5. It moves the actual `asset` file to the correct destination, creating necessary subdirectories.
6. It cleans up the temporary files automatically.

## Requirements

* **To Run**: None! (Just the executable).
* **To Build**: Rust (Cargo).

## License

This project is open-source. Feel free to modify and distribute it.
