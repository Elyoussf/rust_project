use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use sha1::{Sha1, Digest};
use flate2::{Compression, write::ZlibEncoder};
use std::collections::HashSet;
use std::collections::HashMap;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rgit add <file_or_directory> [...]");
        return Ok(());
    }

    // Check if the repository has been initialized
    if !Path::new(".rgit").exists() {
        eprintln!("Error: Repository not initialized. Please run 'rgit init' first.");
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
            eprintln!("Failed to add '{}': {}", current_dir.display(), e);
        } else {
            println!("Added the whole directory to the repository.");
        }
    } else {
        for arg in &args[1..] {
            let path = Path::new(arg);
            if path.ends_with(".rgit") {
                println!("Skipping '{}': .rgit directory should not be processed.", path.display());
                continue;
            }

            if let Err(e) = add(path, &mut indexed_paths) {
                eprintln!("Failed to add '{}': {}", path.display(), e);
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

fn calculate_sha1(content: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn write_object(content: &[u8], sha1: &str) -> io::Result<()> {
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

fn create_tree(dir_path: &Path, indexed_paths: &mut HashSet<PathBuf>) -> io::Result<Option<String>> {
    let mut tree_content = Vec::new();
    let mut entries: HashMap<String, String> = HashMap::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name().into_string().unwrap();

        if file_name == "." || file_name == ".." || path.ends_with(".rgit") {
            continue;
        }

        if path.is_file() {
            let content = fs::read(&path)?;
            let sha1 = calculate_sha1(&content);
            entries.insert(file_name.clone(), sha1.clone());
            // Add file to index
            update_index_binary(&path, &sha1, indexed_paths)?;
        } else if path.is_dir() {
            if let Some(sha1) = create_tree(&path, indexed_paths)? {
                entries.insert(file_name.clone(), sha1);
                // Add directory to index
                update_index_binary(&path, &file_name, indexed_paths)?;
            }
        }
    }

    if entries.is_empty() {
        return Ok(None);
    }

    for (name, sha1) in entries {
        let entry = format!("100644 blob {}\t{}\n", sha1, name);
        tree_content.push(entry);
    }

    let tree_content = tree_content.concat();
    let sha1 = calculate_sha1(tree_content.as_bytes());
    write_object(tree_content.as_bytes(), &sha1)?;
    Ok(Some(sha1))
}

// fn update_index_binary(file_path: &Path, sha1: &str, indexed_paths: &mut HashSet<PathBuf>) -> io::Result<()> {
//     if indexed_paths.contains(file_path) {
//         // File already indexed, skip adding it again
//         return Ok(());
//     }

//     indexed_paths.insert(file_path.to_path_buf());

//     let mut index_file = BufWriter::new(File::options().append(true).create(true).open(".rgit/index")?);
//     let path_str = file_path.to_str().unwrap();

//     index_file.write_all(path_str.as_bytes())?;
//    // Write path
//     index_file.write_all(b" \n")?;  // Separator
   
//     Ok(())
// }
fn update_index_binary(file_path: &Path, sha1: &str, indexed_paths: &mut HashSet<PathBuf>) -> io::Result<()> {
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

    // Write the absolute path and SHA-1 hash to the index file, separated by a space
    index_file.write_all(path_str.as_bytes())?;
    // index_file.write_all(b" ")?;
    index_file.write_all(sha1.as_bytes())?;
    index_file.write_all(b"\n")?;  // Newline to separate entries

    Ok(())
}

fn sha1_to_bytes(sha1: &str) -> [u8; 20] {
    let mut bytes = [0u8; 20];
    for (i, byte) in hex::decode(sha1).unwrap().iter().enumerate() {
        bytes[i] = *byte;
    }
    bytes
}

fn add(path: &Path, indexed_paths: &mut HashSet<PathBuf>) -> io::Result<()> {
    if !path.exists() {
        eprintln!("Path '{}' does not exist", path.display());
        return Ok(());
    }

    // Skip processing if the path is the .rgit directory
    if path == Path::new(".rgit") {
        println!("Skipping .rgit directory.");
        return Ok(());
    }

    if path.is_file() {
        let content = fs::read(path)?;
        let sha1 = calculate_sha1(&content);
        write_object(&content, &sha1)?;
        update_index_binary(path, &sha1, indexed_paths)?;
    } else if path.is_dir() {
        if let Some(sha1) = create_tree(path, indexed_paths)? {
            update_index_binary(path, &sha1, indexed_paths)?;
        }
    } else {
        eprintln!("Path '{}' is neither a file nor a directory", path.display());
    }

    Ok(())
}
