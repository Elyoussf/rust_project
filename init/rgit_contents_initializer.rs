use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

fn main() -> io::Result<()> {
    let current_dir = env::current_dir()?;
    let mut rgit_dir = PathBuf::from(&current_dir);
    rgit_dir.push(".rgit");
    if rgit_dir.exists() {
        println!("git already initialized  ");
    } else {
        fs::create_dir(&rgit_dir)?;
        println!("git initialized succefully!!");
    }

    let current_dir = env::current_dir()?;

    let rgit_dir = current_dir.join(".rgit");


  
    let dirs_to_create = vec![
        "hooks", 
        "info", 
        "logs", 
        "objects/info", 
        "objects/pack", 
        "refs/heads", 
        "refs/tags"
    ]; 
    for dir_name in dirs_to_create {
        let dir_path = rgit_dir.join(dir_name);
        create_directory(&dir_path)?;
    }

    
    let files_to_create = vec![
        ("COMMIT_MSG", ""),
        ("config", "[user]\n    name = HAMZA\n    email = nta.email@example.com\n"),
        ("description", "My rGit Repository\n"),
        ("HEAD", "ref: refs/heads/main\n"),
        ("index", ""),
        ("packed-refs", ""),
    ];

   
    for (file_name, content) in files_to_create {
        create_file_with_content(&rgit_dir, file_name, content)?;
    }

    Ok(())
}

fn create_directory(dir_name: &PathBuf) -> io::Result<()> {
    fs::create_dir_all(dir_name)?;
  
    Ok(())
}

fn create_file_with_content(dir_name: &PathBuf, file_name: &str, content: &str) -> io::Result<()> {
    let file_path = dir_name.join(file_name);


    let mut file = File::create(&file_path)?;


    file.write_all(content.as_bytes())?;

    

    Ok(())
}
