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
    if token.starts_with("reg") {
        return token;
    } else {
        throw_parse_error(&token)
    }
}

fn assert_label_callsite(token: &str) -> String {
    if token.starts_with("$") {
        return token.to_string();
    } else {
        throw_parse_error(&token)
    }
}
// jneq reg4 3 $label 的なreg4,reg5とinputの比較のみ対応
// のつもりだったけどreg0でも使いたくて使えるようになった
// 使う時は注意して
fn gen_jneq(line: &str) -> String {
    gen_cond("jneq", "jnz", line)
}

fn gen_jeq(line: &str) -> String {
    gen_cond("jeq", "jz", line)
}

// jneq reg4 0 $label 的な0との比較の場合subを省略できる
fn gen_zero_cond(asm_opcode: &str, reg: &str, label: &str) -> String {
    return concat_lines(vec![
        (assert_reg(reg.to_string()) + "_to_reg3"),
        label.to_string(),
        asm_opcode.to_string(),
        "\n".to_string(),
    ]);
}

fn gen_cond(instr: &str, asm_opcode: &str, line: &str) -> String {
    if let Some(l) = strip_instr(line, instr) {
        let args = l.split(" ").collect::<Vec<&str>>();
        let reg = *args.get(0).unwrap();
        let val = *args.get(1).unwrap();
        let label = *args.get(2).unwrap();
        assert_label_callsite(label);

        if let Ok(0) = val.parse() {
            return gen_zero_cond(asm_opcode, reg, label);
        } else {
            return concat_lines(vec![
                (assert_reg(reg.to_string()) + "_to_reg1"),
                assert_num(val.to_string()),
                "reg0_to_reg2".to_string(),
                "sub".to_string(),
                label.to_string(),
                asm_opcode.to_string(),
                "\n".to_string(),
            ]);
        }
    } else {
        line.to_string() + "\n"
    }
}

fn gen_load_register_or_immediate(reg_or_imm: &str, dest_reg: &str) -> Vec<String> {
    if reg_or_imm == "in" {
        vec!["in_to_".to_string() + dest_reg]
    } else if reg_or_imm.starts_with("reg") {
        vec![reg_or_imm.to_string() + "_to_" + dest_reg]
    } else {
        vec![
            assert_num(reg_or_imm.to_string()),
            "reg0_to_".to_string() + dest_reg,
        ]
    }
}

// add reg4 reg5
// add in reg4
// add reg4 3
// 的な
fn gen_math(instr: &str, line: &str) -> String {
    if let Some(l) = strip_instr(line, instr) {
        let args = l.split(" ").collect::<Vec<&str>>();
        let lhs = *args.get(0).unwrap();
        let rhs = *args.get(1).unwrap();

        let instrs = gen_load_register_or_immediate(lhs, "reg1")
            .into_iter()
            .chain(gen_load_register_or_immediate(rhs, "reg2"))
            .chain(vec![instr.to_string()])
            .chain(vec!["\n".to_string()])
            .collect();
        return concat_lines(instrs);
    } else {
        line.to_string()
    }
}

fn gen_add(line: &str) -> String {
    gen_math("add", line)
}

fn gen_sub(line: &str) -> String {
    gen_math("sub", line)
}

fn translate_tokens(input: String) -> String {
    input
        .lines()
        .map(|line| gen_goto(line.to_string()))
        .map(|line| gen_jneq(line.as_str()))
        .map(|line| gen_jeq(line.as_str()))
        .map(|line| gen_add(line.as_str()))
        .map(|line| gen_sub(line.as_str()))
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

fn remove_newlines(input: String) -> String {
    concat_lines(
        input
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
    )
}

fn main() {
    if let Some(filename) = env::args().nth(1) {
        let res: Result<String, std::io::Error> = fs::read_to_string(filename)
            .map(|s| translate_tokens(s))
            .map(|s| label_to_linum(s))
            .map(|s| labels_to_comments(s))
            .map(|s| remove_newlines(s));

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
