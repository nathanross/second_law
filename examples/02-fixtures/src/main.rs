use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use std::io::{Read, Write};

fn failure(reason: String) {
    writeln!(&mut std::io::stderr(), "failure: {}", reason).expect("failure: stderr write");
    exit(1);
}

fn main() {
    let args : Vec<String> = std::env::args().skip(1).collect();
    let mut summed : u64 = 0;
    let pathb = PathBuf::from(args.get(0).unwrap());
    let mut contents = String::new();
    File::open(pathb).unwrap().read_to_string(&mut contents);
    let mut num_lines : u64 = 0;
    for (line_num, line_text) in contents.split("\n").enumerate() {
        if line_text != "" {
            match line_text.parse::<u32>() {
                Ok(num) => {
                    summed += num as u64;
                    num_lines += 1;
                },
                _ => {
                    failure(format!("could not parse the text '{0}' as an integer at line {1}", line_text, line_num));
                }
            }
        }
    }
    let average = summed as f64 / num_lines as f64;
    let rounded = average.round() as u64;
    println!("{}", rounded.to_string());
    exit(0);
}
