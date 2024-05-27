// hash.rs


use std::fs::File;
use std::io::{self, Read};
use sha1::{Sha1, Digest};
fn main(){

}
pub fn calculate_sha1(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut hasher = Sha1::new();
    hasher.update(&buffer);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}
