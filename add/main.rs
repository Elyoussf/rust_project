use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write, BufWriter};
use std::path::{Path, PathBuf};
use sha1::{Sha1, Digest};
use flate2::{Compression, write::ZlibEncoder};

fn calculate_sha1(content: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn write_object(content: &[u8], sha1: &str) -> io::Result<()> {
    // Split SHA-1 into directory and file name
    let (dir_name, file_name) = sha1.split_at(2);
    let object_dir = format!(".rgit/objects/{}", dir_name);
    let object_path = format!("{}/{}", object_dir, file_name);

    fs::create_dir_all(&object_dir)?;
    let file = File::create(object_path)?;
    let mut encoder = ZlibEncoder::new(file, Compression::default());
    encoder.write_all(content)?;
    encoder.finish()?;

    Ok(())
}

fn update_index(file_path: &Path, sha1: &str) -> io::Result<()> {
    let mut index_file = BufWriter::new(File::options().append(true).create(true).open(".rgit/index")?);
    writeln!(index_file, "{} {}", sha1, file_path.display())?;
    Ok(())
}

fn process_file(file_path: &Path) -> io::Result<()> {
    let content = fs::read(file_path)?;
    let sha1 = calculate_sha1(&content);
    write_object(&content, &sha1)?;
    update_index(file_path, &sha1)?;
    Ok(())
}

fn process_directory(dir_path: &Path) -> io::Result<()> {
    let mut is_empty = true;

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            is_empty = false;
            process_file(&path)?;
        } else if path.is_dir() {
            is_empty = false;
            process_directory(&path)?;
        }
    }

    // If the directory is empty, add a .keep file
    if is_empty {
        let keep_file_path = dir_path.join(".keep");
        File::create(&keep_file_path)?;
        process_file(&keep_file_path)?;
    }

    Ok(())
}

fn add(path: &Path) -> io::Result<()> {
    if !path.exists() {
        eprintln!("Path '{}' does not exist", path.display());
        return Ok(());
    }

    if path.is_file() {
        process_file(path)?;
    } else if path.is_dir() {
        process_directory(path)?;
    } else {
        eprintln!("Path '{}' is neither a file nor a directory", path.display());
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rgit add <file_or_directory> [...]");
        return Ok(());
    }

    // Handle the case where the argument is "."
    if args.len() == 2 && args[1] == "." {
        let current_dir = env::current_dir()?;
        if let Err(e) = add(&current_dir) {
            eprintln!("Failed to add '{}': {}", current_dir.display(), e);
        } else {
            println!("Added the whole directory to the repository booyah");
        }
    } else {
        for arg in &args[1..] {
            let path = Path::new(arg);
            if let Err(e) = add(&path) {
                eprintln!("Failed to add '{}': {}", path.display(), e);
            } else if path.exists() {
                println!("Added '{}' to the repository.", path.display());
            }
        }
    }

    Ok(())
}
