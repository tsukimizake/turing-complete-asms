use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::exit;
use std::str;

fn strip_instr<'a>(line: &'a str, opcode: &'a str) -> Option<&'a str> {
    line.strip_prefix(opcode).map(|s| s.trim())
}

fn throw_parse_error<A>(line: &str) -> A {
    println!("parse error on {}", line);
    exit(1);
}
fn concat_lines(lines: Vec<String>) -> String {
    return lines.join("\n");
}

fn gen_goto(line: String) -> String {
    if let Some(l) = strip_instr(line.as_str(), "goto") {
        return concat_lines(vec![l.to_string(), "goto".to_string()]);
    } else {
        line.to_string()
    }
}

fn assert_num(token: String) -> String {
    if let Ok(_n) = token.parse::<i32>() {
        return token;
    } else {
        throw_parse_error(&token)
    }
}
fn assert_reg(token: String) -> String {
    if token == "reg4" || token == "reg5" {
        return token;
    } else {
        throw_parse_error(&token)
    }
}

// jneq reg4 3 $label 的なreg4,reg5とinputの比較のみ対応
fn gen_jneq(line: &str) -> String {
    if let Some(l) = strip_instr(line, "jneq") {
        let args = l.split(" ").collect::<Vec<&str>>();
        let reg = *args.get(0).unwrap();
        let val = *args.get(1).unwrap();
        let label = *args.get(2).unwrap();
        return concat_lines(vec![
            (assert_reg(reg.to_string()) + "_to_reg1"),
            assert_num(val.to_string()),
            "reg0_to_reg2".to_string(),
            "sub".to_string(),
            label.to_string(),
            "jnz".to_string(),
        ]);
    } else {
        line.to_string() + "\n"
    }
}

fn translate_tokens(input: String) -> String {
    input
        .lines()
        .map(|line| gen_goto(line.to_string()))
        .map(|line| gen_jneq(line.as_str()))
        .collect::<String>()
}

fn is_empty_or_comment_or_label(line: &str) -> bool {
    line.is_empty() || line.starts_with("#") || line.starts_with("@")
}

fn get_if_label(lines_to_linum: &HashMap<&str, i32>, line: &str) -> Option<i32> {
    if line.starts_with("@") {
        let res = lines_to_linum.get(line).map(|x| *x);
        assert!(res.is_some(), "label not found");
        res
    } else {
        None
    }
}

fn make_label_map(input: &String) -> HashMap<String, i32> {
    let mut idx = 0;
    let lines_to_linum = input
        .lines()
        .map(|l| {
            let res = (l, idx);
            if !is_empty_or_comment_or_label(l) {
                idx = idx + 1;
            }
            res
        })
        .collect::<HashMap<&str, i32>>();

    input
        .lines()
        .map(|l| {
            get_if_label(&lines_to_linum, l)
                .into_iter()
                .flat_map(|linum| {
                    l.to_string()
                        .strip_prefix("@")
                        .map(|x| (x.to_string(), linum))
                })
                .next()
        })
        .filter_map(|x| x)
        .collect::<HashMap<String, i32>>()
}

fn label_to_linum(input: String) -> String {
    let label_map: HashMap<String, i32> = make_label_map(&input);
    // println!("{:?}", label_map);
    let r: Vec<String> = input
        .lines()
        .map(|line| {
            if line.starts_with("$") {
                label_map
                    .get(line.strip_prefix("$").unwrap_or(""))
                    .map(|n| (*n).to_string() + " # " + line)
                    .unwrap_or(line.to_string())
            } else {
                line.to_string()
            }
        })
        .collect();

    concat_lines(r)
}

fn labels_to_comments(input: String) -> String {
    concat_lines(
        input
            .lines()
            .map(|line| {
                line.strip_prefix("@")
                    .map(|label| "#".to_string() + label)
                    .unwrap_or(line.to_string())
            })
            .collect::<Vec<String>>(),
    )
}

fn main() {
    if let Some(filename) = env::args().nth(1) {
        let res: Result<String, std::io::Error> = fs::read_to_string(filename)
            .map(|s| translate_tokens(s))
            .map(|s| label_to_linum(s))
            .map(|s| labels_to_comments(s));
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
