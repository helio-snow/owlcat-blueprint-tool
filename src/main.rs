use clap::Parser;
use serde_json::{Value};
use serde_json_path::JsonPath;
use walkdir::WalkDir;
use core::fmt;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;


#[derive(Parser, Debug)]
#[command(name = "ocbpt", version = "0.1", about ="Queries Owlcat blueprint files", long_about = None)]
struct Args {
    /// Path to the blueprint file
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    type_query: String,

}

#[derive(Debug)]
struct BlueprintError{
    error: String
}

impl fmt::Display for BlueprintError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blueprint Error: {}", self.error)
    }
}

impl Error for BlueprintError {}

fn search(args: &Args) -> Result<Vec<String>, Box<dyn Error>> {
    let guid_path = JsonPath::parse("$.AssetId")?;
    let type_path = JsonPath::parse("$.Data.$type")?;
    let mut guids: Vec<String> = Vec::new();
    for entry in WalkDir::new(&args.input){
        let entry = entry?;
        let path = entry.path();
        let extension = path.extension().map(|x| x.display().to_string());
        if extension == Some(".jbp".to_owned()) {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            let filename: String = path.file_name().map(|p| p.display().to_string()).ok_or(BlueprintError{error: "No filename found".to_string()})?;   
            let type_with_hash_string = type_path.query(&json).exactly_one()?.as_str().ok_or(BlueprintError{error: format!("No type found in file {}", filename)})?;
            let type_string = type_with_hash_string.split(", ").collect::<Vec<_>>()[1];
            if type_string == args.type_query {
                let guid = guid_path.query(&json).exactly_one()?.as_str().ok_or(BlueprintError{error: format!("No GUID found in file {}", filename)})?;
                guids.push(guid.to_string());
            }
        }
    }
    Ok(guids.clone())
}

fn main() {
    let args = Args::parse();
    let guids = search(&args).unwrap();
    println!("GUIDs with type {}: {}", args.type_query, guids.join(", "));
}
