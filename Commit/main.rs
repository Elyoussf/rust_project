use std::env;
use std::fs;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::collections::HashMap;
use sha1::{Sha1, Digest};
use flate2::{Compression, write::ZlibEncoder};
use chrono::Utc;

fn main() -> io::Result<()> {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rgit commit -m <message>");
        return Ok(());
    }

    if !Path::new(".rgit").exists() {
        eprintln!("Error: Repository not initialized. Please run 'rgit init' first.");
        return Ok(());
    }


    if args.len() < 3 || args[1] != "-m" {
        eprintln!("Usage: rgit commit -m <message>");
        return Ok(());
    }


    let message = &args[2];

    commit(message)
}


fn create_file_with_content(dir_name: &PathBuf, file_name: &str, content: &str) -> io::Result<()> {
    let file_path = dir_name.join(file_name);
    let mut file = File::create(&file_path)?;
    file.write_all(content.as_bytes())?;

    return Ok(())
}





fn commit(message: &str) -> io::Result<()> {



   // 1- Check the sataging area 
   let current_dir = env::current_dir()?;
   let rgit_dir = current_dir.join(".rgit")
   let file_path = dir_name.join(file_name);






    // 2- First Thing to do is to save the message in the COMMIT_MSG

  
    create_file_with_content(&rgit_dir, "COMMIT_MSG", message)?;


    //3 - Create the tree Object 


    //4- Create the Commit Object 


  
    Ok(())
}


// To check if the file is empty (It's important for the index  )
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
















