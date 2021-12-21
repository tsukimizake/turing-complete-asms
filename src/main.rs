use std::env;
use std::fs;

fn translate_tokens(input: String) -> String {
    "".to_string()
}

fn label_to_linum(input: String) -> String {
    "".to_string()
}

fn main() {
    if let Some(filename) = env::args().nth(1) {
        println!("f:{}", filename);
        let res: Result<String, std::io::Error> = fs::read_to_string(filename)
            .map(|s| translate_tokens(s))
            .map(|s| label_to_linum(s));
        match res {
            Err(e) => println!("{}", e),
            Ok(r) => println!("{}", r),
        };

        return ();
    } else {
        println!("give me filename!");
        return ();
    }
}
