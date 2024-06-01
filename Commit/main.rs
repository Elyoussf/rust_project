use std::env;
use std::fs::{self, File};
use sha1::{Sha1, Digest};
use flate2::{Compression, write::ZlibEncoder};
use chrono::Utc;
use std::io;
use std::path::Path;
use std::io::Write;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "-m" {
        println!("Usage: rgit commit -m <message>");
        return Ok(());
    }

    if !Path::new(".rgit").exists() {
        println!("Error: Repository not initialized. Please run 'rgit init' first.");
        return Ok(());
    }

    let message = &args[2];
    commit(message)
}

fn create_file_with_content(dir_name: &Path, file_name: &str, content: &str) -> io::Result<()> {
    let file_path = dir_name.join(file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn commit(message: &str) -> io::Result<()> {
    let current_dir = env::current_dir()?;
    let rgit_dir = current_dir.join(".rgit");
    let index_path = rgit_dir.join("index");

    if !index_path.exists() || is_file_empty(index_path.to_str().unwrap())? {
        println!("Nothing to commit. Staging area is empty.");
        return Ok(());
    }

    let index_content = read_index_content()?;
    let index_string = String::from_utf8(index_content).unwrap();
    let indexed_paths: Vec<&str> = index_string.split_whitespace().collect();

    if indexed_paths.is_empty() {
        println!("Nothing to commit. Staging area is empty.");
        return Ok(());
    }

    let tree_sha1 = create_tree_object(&indexed_paths)?;

    let author_name = "Hamoudi";
    let author_email = "hamoudi@sbitar.com";
    let timestamp = Utc::now().to_rfc3339();

    let commit_content = format!(
        "tree {}\nauthor {} <{}>\ndate {}\n\n{}\n",
        tree_sha1, author_name, author_email, timestamp, message
    );

    let commit_sha1 = calculate_sha1(commit_content.as_bytes());
    write_object(commit_content.as_bytes(), &commit_sha1)?;

    println!("Committed as {}", commit_sha1);

    // Truncate the index file to empty it after committing
    File::create(&index_path)?;

    Ok(())
}

fn is_file_empty(path: &str) -> Result<bool, std::io::Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len() == 0)
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

fn read_index_content() -> io::Result<Vec<u8>> {
    let index_path = Path::new(".rgit/index");
    if !index_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Index file not found"));
    }
    fs::read(index_path)
}

fn create_tree_object(paths: &[&str]) -> io::Result<String> {
    let mut tree_content = Vec::new();

    for path_str in paths {
        let path = Path::new(path_str);
        if path.is_file() {
            let content = fs::read(path)?;
            let sha1 = calculate_sha1(&content);
            write_object(&content, &sha1)?;

            let file_name = path.file_name().unwrap().to_str().unwrap();
            let entry = format!("100644 blob {}\t{}\n", sha1, file_name);
            tree_content.push(entry);
        }
    }

    let tree_content = tree_content.concat();
    let sha1 = calculate_sha1(tree_content.as_bytes());
    write_object(tree_content.as_bytes(), &sha1)?;

    Ok(sha1)
}
