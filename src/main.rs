use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

extern crate clap;
use clap::{App, Arg};

extern crate regex;
use regex::Regex;

fn main() {
    let matches = App::new("lua_call_tgf")
        .about(
            "takes a .lua file and outputs a .tgf file representing its call graph",
        )
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .help("The lua file to read from")
                .index(1)
                .required(true),
        )
        .arg(Arg::with_name("count").short("c").long("count").help(
            "instead of anything else, just output the count of ideas in the file",
        ))
        .arg(
            Arg::with_name("verbose")
                .short("V")
                .long("verbose")
                .help("Print extra info in addition to printing the tgf file"),
        )
        .get_matches();

    let filename = matches.value_of("file").expect("No filename found!");

    let reader = BufReader::new(
        File::open(filename).expect(&format!("Cannot open {}", filename)),
    );

    let print_more = matches.is_present("verbose");

    let mut edges: Vec<(String, String)> = Vec::new();

    let mut function_stack = vec!["<top level>".to_owned()];

    let mut in_comment = false;

    //This only works on top level named functions
    let function_def = Regex::new(r"^function +([A-Za-z_][A-Za-z0-9_]*)").unwrap();

    //only matching the first paren keeps the matches fro m overlapping in things like this:
    // f(g(), h())
    let function_call = Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)\(").unwrap();

    if matches.is_present("count") {
        println!("count: {}", reader.lines().count());

        return;
    }

    for l in reader.lines() {
        let line = l.unwrap();
        let blank = line.chars().all(|c| c.is_whitespace());
        if blank {
            continue;
        }

        //TODO handle function definitions inside functions
        //function defintion
        if let Some(captures) = function_def.captures(&line) {
            function_stack.push(captures[1].to_owned());

            continue;
        }

        //function call
        if let Some(current_function) = function_stack.last() {
            for captures in function_call.captures(&line) {
                edges.push((current_function.clone(), captures[1].to_owned()));
            }
        }

        //function end
        //This is a quick and dirty hack that assumes reasonable indentation.
        if line.starts_with("end") {
            function_stack.pop();
        }

        //we're not handling --[[===[[ style comments right now
        //enter comment
        if !in_comment && line.contains("--[[") {
            in_comment = true;
        }

        //leave comment
        //TODO handle "[[]] -- [["
        if in_comment && line.contains("]]") {
            in_comment = false;
        }
    }

    edges.sort();

    let tgf = get_tgf(&edges);

    println!("{}", tgf);
}

fn get_tgf<T: AsRef<str>>(edges: &Vec<(T, T)>) -> String {
    use std::collections::HashMap;

    let mut node_labels = HashMap::new();

    let mut counter = 0;
    for &(ref s1, ref s2) in edges.iter() {
        node_labels.entry(s1.as_ref()).or_insert_with(|| {
            counter += 1;
            counter
        });
        node_labels.entry(s2.as_ref()).or_insert_with(|| {
            counter += 1;
            counter
        });
    }

    let mut tgf = String::new();

    let mut node_label_pairs: Vec<_> = node_labels.iter().collect();

    node_label_pairs.sort();

    for &(node, label) in node_label_pairs.iter() {
        tgf.push_str(&format!("{} {}\n", label, node));
    }

    tgf.push_str("#\n");

    for edge in edges.iter() {
        let label1: usize = *node_labels.get(edge.0.as_ref()).unwrap_or(&0);
        let label2: usize = *node_labels.get(edge.1.as_ref()).unwrap_or(&0);

        tgf.push_str(&format!("{} {}\n", label1, label2))
    }

    tgf
}
