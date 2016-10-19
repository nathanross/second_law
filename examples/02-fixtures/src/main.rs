use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use std::io::{Read, Write};

fn failure(reason: String) {
    writeln!(&mut std::io::stderr(), "failure: {}", reason).expect("failure: stderr write");
    exit(1);
}

fn average(sample : &[String]) -> f64 {
    let mut summed : u64 = 0;
    let mut datapoints : u64 = 0;
    for (point_num, point_text) in sample.iter().enumerate() {        
        match point_text.parse::<u32>() {
            Ok(num) => {
                summed += num as u64;
                datapoints += 1;
            },
            _ => {
                failure(format!("could not parse the text '{0}' as an integer at line {1}", point_text, point_num));
            }
        }
    }
    return summed as f64 / datapoints as f64;
}

fn sample_from_file(filename: &str) -> Vec<String> {
    let pathb = PathBuf::from(filename);
    let mut contents = String::new();
    File::open(pathb).unwrap().read_to_string(&mut contents).unwrap();
    contents.split("\n").filter(|x| *x != "").map(|x| x.to_owned()).collect()
}

fn main() {
    let args : Vec<String> = std::env::args().skip(1).collect();
    let sample = if args.len() == 2 && *args.get(0).unwrap() == "-f" {
        sample_from_file(args.get(1).unwrap())
    } else {
        args
    };
    let result = average(&sample).round() as u64;
    println!("{}", result.to_string());
    exit(0);
}
