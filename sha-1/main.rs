use std::fs::File;
use std::io::{self, Read};
use sha1::{Sha1, Digest};

fn main() -> io::Result<()> {
   
    let file_path = "./test";
    

    let mut file = File::open(file_path)?;
    
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
   
    let mut hasher = Sha1::new();
    
    hasher.update(&buffer);
    

    let result = hasher.finalize();
    
    
    println!("SHA-1 hash: {:x}", result);
    
    Ok(())
}
