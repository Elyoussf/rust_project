use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use sha1::{Sha1, Digest};
use flate2::{Compression, write::ZlibEncoder};
use chrono::Utc;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "-m" {
        eprintln!("Usage: rgit commit -m <message>");
        return Ok(());
    }

    if !Path::new(".rgit").exists() {
        eprintln!("Error: Repository not initialized. Please run 'rgit init' first.");
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
    // 1- Check the staging area
    let current_dir = env::current_dir()?;
    let rgit_dir = current_dir.join(".rgit");

    // Assuming index file is used as a staging area
    let index_path = rgit_dir.join("index");

    // If the index file does not exist or is empty, return an error
    if !index_path.exists() || is_file_empty(index_path.to_str().unwrap())? {
        eprintln!("Nothing to commit. Staging area is empty.");
        return Ok(());
    }

    // 2- Save the message in the COMMIT_MSG
    create_file_with_content(&rgit_dir, "COMMIT_MSG", message)?;

    // 3- Hash the content of the index and save it as a tree object
    let index_sha1 = hash_and_save_index()?;

    // 4- Create the commit object
    let author_name = "Hamoudi";
    let author_email = "hamoudi@sbitar.com";
    let timestamp = Utc::now().to_rfc3339();

    let commit_content = format!(
        "tree {}\nauthor {} <{}>\ndate {}\n\n{}\n",
        index_sha1, author_name, author_email, timestamp, message
    );

    let commit_sha1 = calculate_sha1(commit_content.as_bytes());
    write_object(commit_content.as_bytes(), &commit_sha1)?;

    println!("Committed as {}", commit_sha1);
    Ok(())
}

// To check if the file is empty (It's important for the index)
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

fn hash_and_save_index() -> io::Result<String> {
    let index_content = read_index_content()?;
    let sha1 = calculate_sha1(&index_content);
    write_object(&index_content, &sha1)?;
    Ok(sha1)
}
