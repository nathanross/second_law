use std::process::exit;
use std::io::Write;

fn failure(reason: String) {
    writeln!(&mut std::io::stderr(), "failure: {}", reason).expect("failure: stderr write");
    exit(1);
}

fn main() {
    let args : Vec<String> = std::env::args().skip(1).collect();
    let mut summed : u64 = 0;
    for arg in args.iter() {
        match arg.parse::<u32>() {
            Ok(num) => {
                summed += num as u64;
            },
            _ => {
                failure(format!("could not parse argument '{0}'", arg));
            }
        }
    }
    println!("{}", summed.to_string());
    exit(0);
}
