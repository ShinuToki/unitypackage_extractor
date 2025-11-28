use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use regex::Regex;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use tempfile::TempDir;
use path_clean::PathClean;

/// Displays help and correct program usage
fn print_help(program_name: &str) {
    println!("UnityPackage Extractor (Rust Version)");
    println!("---------------------------------------");
    println!("Usage: {} <file.unitypackage> [output_path]", program_name);
    println!();
    println!("Arguments:");
    println!("  <file.unitypackage>     Path to the file you want to extract.");
    println!("  [output_path]           (Optional) Folder where to extract files.");
    println!("                          Defaults to the current directory.");
    println!();
    println!("Options:");
    println!("  -h, --help              Show this help message.");
}

fn extract_package(package_path: &Path, output_path: Option<&Path>) -> Result<()> {
    // Determine output path (cwd by default)
    let cwd = env::current_dir()?;
    let output_path = output_path.unwrap_or(&cwd);
    
    // Resolve absolute path for security checks
    let output_path_abs = output_path.canonicalize().unwrap_or(output_path.to_path_buf());

    // Create temporary directory
    let tmp_dir = TempDir::new()?;
    let tmp_path = tmp_dir.path();

    println!("Unpacking file temporarily...");

    // Open .unitypackage (tar.gz)
    let file = File::open(package_path).context("Could not open .unitypackage file")?;
    let tar = GzDecoder::new(file);
    let mut archive = tar::Archive::new(tar);

    // Unpack everything to temp
    archive.unpack(tmp_path).context("Error unpacking to temporary directory")?;

    // Regex compiled once
    // > : " | ? * are forbidden characters in Windows filenames
    let windows_bad_chars = Regex::new(r#"[>:"|?*]"#).expect("Invalid Regex");

    // Iterate through directories in temp
    for entry in fs::read_dir(tmp_path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if !entry_path.is_dir() {
            continue;
        }

        let pathname_file = entry_path.join("pathname");
        let asset_file = entry_path.join("asset");

        if !pathname_file.exists() || !asset_file.exists() {
            continue;
        }

        // Read the 'pathname' file containing the real asset path
        let file = File::open(&pathname_file)?;
        let mut reader = BufReader::new(file);
        let mut pathname = String::new();
        reader.read_line(&mut pathname)?;
        
        let mut pathname = pathname.trim_end().to_string();

        // Sanitization for Windows
        if cfg!(windows) {
            pathname = windows_bad_chars.replace_all(&pathname, "_").to_string();
        }

        // Construct final path
        let asset_out_path = output_path.join(&pathname);
        
        // Security Check: Prevent Path Traversal (Zip Slip vulnerability logic)
        let resolved_out_path = output_path_abs.join(&pathname).clean();
        
        if !resolved_out_path.starts_with(&output_path_abs) {
            println!("WARNING: Skipping '{}' as '{}' is outside the destination path '{}'.", 
                entry.file_name().to_string_lossy(), 
                asset_out_path.display(), 
                output_path.display()
            );
            continue;
        }

        println!("Extracting '{}' as '{}'", entry.file_name().to_string_lossy(), pathname);

        if let Some(parent) = asset_out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        move_file(&asset_file, &asset_out_path)?;
    }

    Ok(())
}

fn move_file(src: &Path, dst: &Path) -> Result<()> {
    // Attempt rename (fast, but fails across different drives/partitions)
    if fs::rename(src, dst).is_err() {
        // Fallback: Copy and delete
        fs::copy(src, dst)?;
        fs::remove_file(src)?;
    }
    Ok(())
}

fn cli() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let program_name = args.first().map(|s| s.as_str()).unwrap_or("unitypackage_extractor");

    // Check help flags
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_help(program_name);
        return Ok(());
    }

    // Check argument count
    if args.len() < 2 {
        print_help(program_name);
        println!(); // Extra space
        bail!("Error: You must specify at least the .unitypackage file.");
    }

    let package_path = Path::new(&args[1]);

    // 3. Check input file existence
    if !package_path.exists() {
        bail!("Error: The file '{}' does not exist.", package_path.display());
    }

    let output_path = if args.len() > 2 {
        Some(Path::new(&args[2]))
    } else {
        None
    };

    let start_time = Instant::now();
    extract_package(package_path, output_path)?;
    let duration = start_time.elapsed();

    println!("--- Finished in {:.4} seconds ---", duration.as_secs_f64());
    Ok(())
}

fn main() {
    if let Err(e) = cli() {
        // Use eprintln for errors (stderr output)
        eprintln!("{}", e);
        std::process::exit(1);
    }
}