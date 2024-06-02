use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use sha1::{Sha1, Digest};

fn main() -> io::Result<()> {
    let current_dir = Path::new(".");
    let rgit_dir = current_dir.join(".rgit");
    let index_path = rgit_dir.join("index");
    let object_dir = rgit_dir.join("objects");

    // Read the content of the index file
    let index_content = match fs::read_to_string(&index_path) {
        Ok(content) => content,
        Err(_) => {
            println!("Index file not found or empty. There are no staged changes.");
            return Ok(());
        }
    };

    let indexed_files: HashSet<PathBuf> = index_content
        .lines()
        .map(|line| {
            let mut parts = line.split_whitespace();
            let _ = parts.next(); // Skip SHA1 hash
            let path_str = parts.next().unwrap();
            let abs_path = current_dir.join(path_str); // Convert to absolute path
            abs_path
        })
        .collect();

    let mut untracked_files = HashSet::new();
    let mut uncommitted_files = HashSet::new();

    traverse_directory(
        current_dir,
        &rgit_dir,
        &indexed_files,
        &object_dir,
        &mut untracked_files,
        &mut uncommitted_files,
    )?;

    if untracked_files.is_empty() && uncommitted_files.is_empty() {
        println!("Up to date");
    } else {
        if !untracked_files.is_empty() {
            println!("Untracked files:");
            for file in untracked_files {
                println!("   {}", file.display()); // Use display() to print path
            }
        }
        if !uncommitted_files.is_empty() {
            println!("Uncommitted changes:");
            for file in uncommitted_files {
                println!("   {}", file.display()); // Use display() to print path
            }
        }
    }

    Ok(())
}

fn traverse_directory(
    current_dir: &Path,
    rgit_dir: &Path,
    indexed_files: &HashSet<PathBuf>,
    object_dir: &Path,
    untracked_files: &mut HashSet<PathBuf>,
    uncommitted_files: &mut HashSet<PathBuf>,
) -> io::Result<()> {
    // Iterate over files and directories in the current directory
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip the .rgit directory itself
        if path == *rgit_dir {
            continue;
        }

        // If it's a file, compute its SHA1 hash
        if path.is_file() {
            let content = fs::read(&path)?;
            let file_sha1 = calculate_sha1(&content);
            let object_path = object_dir.join(&file_sha1[..2]).join(&file_sha1[2..]);
            if !indexed_files.contains(&path) && !object_path.exists() {
                untracked_files.insert(path);
            } else if indexed_files.contains(&path) && !object_path.exists() {
                uncommitted_files.insert(path);
            }
        }
        // If it's a directory, recursively traverse it
        else if path.is_dir() {
            traverse_directory(
                &path,
                rgit_dir,
                indexed_files,
                object_dir,
                untracked_files,
                uncommitted_files,
            )?;
        }
    }

    Ok(())
}

fn calculate_sha1(content: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}
