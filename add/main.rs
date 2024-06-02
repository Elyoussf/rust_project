use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write, Read};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use sha1::{Sha1, Digest};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rgit add <file_or_directory> [...]");
        return Ok(());
    }

    if !Path::new(".rgit").exists() {
        println!("Error: Repository not initialized. Please run 'rgit init' first.");
        return Ok(());
    }

    let mut indexed_paths: HashSet<PathBuf> = HashSet::new();
    if let Ok(file) = File::open(".rgit/index") {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                let mut parts = line.split_whitespace();
                if let Some(path) = parts.next() {
                    indexed_paths.insert(PathBuf::from(path));
                }
            }
        }
    }

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

    let mut index_file = BufWriter::new(File::create(".rgit/index")?);
    for path in &indexed_paths {
        let hash = if path.is_file() {
            calculate_sha1(&path)?
        } else {
            // Use a fixed hash for directories
            calculate_directory_sha1(&path)?
        };
        index_file.write_all(path.to_str().unwrap().as_bytes())?;
        index_file.write_all(b" ")?;
        index_file.write_all(hash.as_bytes())?;
        index_file.write_all(b"\n")?;
    }

    println!("Index file updated successfully.");

    Ok(())
}

fn calculate_sha1(file_path: &Path) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0; 1024];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn calculate_directory_sha1(dir_path: &Path) -> io::Result<String> {
    // Use a hash of the directory path as a placeholder
    let mut hasher = Sha1::new();
    hasher.update(dir_path.to_str().unwrap().as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn update_index_binary(file_path: &Path, indexed_paths: &mut HashSet<PathBuf>) -> io::Result<()> {
    if indexed_paths.contains(file_path) {
        return Ok(());
    }

    println!("Adding '{}' to index.", file_path.display()); // Debugging statement

    indexed_paths.insert(file_path.to_path_buf());
    Ok(())
}

fn add(path: &Path, indexed_paths: &mut HashSet<PathBuf>) -> io::Result<()> {
    if !path.exists() {
        println!("Path '{}' does not exist", path.display());
        return Ok(());
    }

    if path.components().any(|comp| comp.as_os_str() == ".rgit") {
        return Ok(());
    }

    if path.is_file() {
        update_index_binary(path, indexed_paths)?;
    } else if path.is_dir() {
        // Add the directory itself
        update_index_binary(path, indexed_paths)?;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            add(&entry.path(), indexed_paths)?;
        }
    } else {
        println!("Path '{}' is neither a file nor a directory", path.display());
    }

    Ok(())
}
