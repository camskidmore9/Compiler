// inFile.rs

//Rules
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(dead_code)]

//imports
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::io::Read;
use std::fs::read_to_string;

//inFile Class
struct inFile{
    attatchFile: bool,
    fileName: String,
    lineCnt: i32,   
}
impl inFile {
    fn new(fileName: &str) -> inFile {
        inFile {
            fileName: fileName.to_string(),
            attatchFile: false,
            lineCnt: 4,
        }

    }
}


fn main() {
    let first = env::args().nth(1).expect("please supply an argument");
    let inFileName: String = first.parse().expect("not an integer!");
    println!("Your file: '{}'", inFileName);
    
    println!("Opening {}", inFileName);
    let mut file = File::open(&inFileName).expect("Cant open the file");
    
    let mut text = String::new();
    file.read_to_string(&mut text).expect("cant read the file");
    println!("file has {} bytes", text.len());
    println!("File Contents:\n{}", text);
    
    println!("Creating inFile structure");
    let f = inFile::new(inFileName.as_str());
    println!("inFile: {}", f.fileName);

}