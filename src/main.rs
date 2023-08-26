use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args.to_owned());
    let primary_arg = args.get(1);
    let file = args.get(2);

    if let Some(value) = primary_arg {
        match value.as_str() {
            "l" => println!("Listing folders"),
            "o" if file.is_some() => println!("{:?}",String::from("opening: ") + file.unwrap()),
            "o" => println!("Error opening project, no file specified"),
            _ => println!("Unknow arg"),
        }
    }
}

