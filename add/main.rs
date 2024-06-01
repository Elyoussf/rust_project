use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::collections::HashSet;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rgit add <file_or_directory> [...]");
        return Ok(());
    }

    // Check if the repository has been initialized
    if !Path::new(".rgit").exists() {
        println!("Error: Repository not initialized. Please run 'rgit init' first.");
        return Ok(());
    }

    // Load the existing index file into a HashSet for quick lookup
    let mut indexed_paths: HashSet<PathBuf> = HashSet::new();
    if let Ok(file) = File::open(".rgit/index") {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                for path in line.split_whitespace() {
                    indexed_paths.insert(PathBuf::from(path));
                }
            }
        }
    }

    // Handle the case where the argument is "."
    if args.len() == 3 && args[2] == "." {
        let current_dir = env::current_dir()?;
        if let Err(e) = add(&current_dir, &mut indexed_paths) {
            println!("Failed to add '{}': {}", current_dir.display(), e);
        } else {
            println!("Added the whole directory to the repository.");
        }
    } else {
        for arg in &args[1..] {
            let path = Path::new(arg);
            if let Err(e) = add(path, &mut indexed_paths) {
                println!("Failed to add '{}': {}", path.display(), e);
            } else if path.exists() {
                println!("Added '{}' to the repository.", path.display());
            }
        }
    }

    // Write the updated paths back to the index file
    let mut index_file = BufWriter::new(File::create(".rgit/index")?);
    for path in indexed_paths {
        index_file.write_all(path.to_str().unwrap().as_bytes())?;
        index_file.write_all(b" ")?;
    }

    Ok(())
}

fn update_index_binary(file_path: &Path, indexed_paths: &mut HashSet<PathBuf>) -> io::Result<()> {
    // Check if the path is already indexed
    if indexed_paths.contains(file_path) {
        return Ok(());
    }

    // Insert the new file path into the indexed_paths set
    indexed_paths.insert(file_path.to_path_buf());

    // Open the index file in append mode, creating it if it doesn't exist
    let mut index_file = BufWriter::new(File::options().append(true).create(true).open(".rgit/index")?);

    // Get the absolute path of the file
    let binding = file_path.canonicalize()?;
    let path_str = binding.to_str().unwrap();

    // Write the absolute path to the index file
    index_file.write_all(path_str.as_bytes())?;
    index_file.write_all(b" ")?;  // Space to separate entries

    Ok(())
}

fn add(path: &Path, indexed_paths: &mut HashSet<PathBuf>) -> io::Result<()> {
    if !path.exists() {
        println!("Path '{}' does not exist", path.display());
        return Ok(());
    }

    // Skip processing if the path contains ".rgit" in its components
    if path.components().any(|comp| comp.as_os_str() == ".rgit") {
        return Ok(());
    }

    if path.is_file() {
        update_index_binary(path, indexed_paths)?;
    } else if path.is_dir() {
        // Add directory itself
        update_index_binary(path, indexed_paths)?;

        // Add all files and subdirectories
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            add(&entry.path(), indexed_paths)?;
        }
    } else {
        println!("Path '{}' is neither a file nor a directory", path.display());
    }

    Ok(())
}
