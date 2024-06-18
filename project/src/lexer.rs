// args1.rs
#![allow(non_snake_case)]
use std::env;

fn main() {
    let first = env::args().nth(1).expect("please supply an argument");
    let inFileName: String = first.parse().expect("not an integer!");
    println!("Your file: '{}'", inFileName);
    // do your magic
}