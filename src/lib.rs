use std::error::Error;
use std::fs;
use std::env;
use std::path::Path;

pub enum InputType {
    File,
    Directory
}

pub struct Config {
    pub query: String,
    pub name: String,
    pub input_type: InputType,
    pub case_sensitive: bool
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();
        let name = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file or folder name")
        };
        let query = args.collect::<Vec<String>>().join(" ");
        if query == "" {
            return Err("Didn't get a query")
        }
        let input_type = match fs::metadata(&name).unwrap().is_file() {
            true => InputType::File,
            false => InputType::Directory
        };
        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();
        Ok(Config{query, name, input_type, case_sensitive})
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let path = Path::new(&config.name);
    let _res = match config.input_type {
        InputType::File => search_single_file(&path, &config).unwrap(),
        InputType::Directory => search_directory(&path, &config).unwrap()
    };
    Ok(())
}

pub fn search_directory(dir: &Path, config: &Config) -> Result<(), Box<dyn Error>> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                search_directory(&path, &config)?;
            }
            else {
                search_single_file(&path, &config)?;
                println!("");
            }
        }
    }
    Ok(())
}

pub fn search_single_file(path: &Path, config: &Config) -> Result<(), Box<dyn Error>>{
    let file_contents = fs::read_to_string(path)?;
    let matched_lines = if config.case_sensitive {
        search(&config.query, &file_contents)
    } else {
        search_case_insensitive(&config.query, &file_contents)
    };
    if matched_lines.len() > 0 {
        println!("File: {}", path.to_str().unwrap());
        for line in matched_lines {
            let index = &file_contents.lines().position(|r| r == line.to_string()).unwrap();
            println!("{0}: {1}", index, line);
        }
    }
    Ok(())
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines()
        .filter(|line| line.contains(query))
        .collect()
    // Old, bad way to do this
    // ------------------------
    // let mut results = Vec::new();
    // for line in contents.lines() {
    //     if line.contains(query) {
    //         results.push(line.trim());
    //     }
    // }
    // results
}
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    contents.lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .map(|line| line.trim())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
            Rust:
            safe, fast, productive.
            Pick three.
            Duct tape";
        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }
    #[test]
    fn no_results() {
        let empty_vector: Vec<&str> = Vec::new();
        let query = "foo";
        let contents = "";
        assert_eq!(empty_vector, search(query, contents));
    }
    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
            Rust:
            safe, fast, productive.
            Pick three.
            Trust me.";
        assert_eq!(vec!["Rust:", "Trust me."], search_case_insensitive(query, contents));
    }
}
