#![feature(drain_filter)]
#![feature(await_macro, async_await, futures_api)]

#[macro_use] extern crate lalrpop_util;

#[macro_use] extern crate quote;
extern crate proc_macro2;

use std::io::BufReader;
use std::io::{BufWriter, Write};
use std::io::BufRead;
use std::fs::File;
use std::path::Path;
use std::string::ToString;

extern crate clap;

use clap::{App, Arg}; 

pub mod bt;

use crate::bt::{bt_from_nodes, bt_to_action_asts, bt_to_ast};

lalrpop_mod!(pub bt_parser); // synthesized by LALRPOP

fn main() -> Result<(), std::io::Error>
{
    let matches = App::new("bt_generator")
                    .version("1.0")
                    .about("Generate async rust behavior tree from tree definition")
                    .author("Nils Bore")
                    .arg(Arg::with_name("input")
                        .help("Sets the input file to use")
                        .required(true)
                        .index(1))
                    .get_matches();

    let input_path = Path::new(matches.value_of("input").unwrap());
    println!("Using input file: {}", input_path.to_str().unwrap());

    let folder_path = match input_path.parent() {
        Some(p) => p,
        None => return Ok(()),
    };

    let logic_path = folder_path.join(Path::new("bt_logic.rs"));

    let mut nodes = Vec::new();
    let f = File::open("examples/noc_example.txt").unwrap();
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();

        if l.len()  == 0 {
            continue;
        }

        match bt_parser::NodesParser::new().parse(&l) {
            Ok(n) => {
                println!("{:?}", n);
                nodes.extend(n)
            },
            Err(e) => println!("{}", e),
        }
    }

     println!("{:?}", nodes);

     let tree = bt_from_nodes(&nodes)?;

     println!("{:?}", tree);

     let asts = bt_to_action_asts(&tree);

     for (key, value) in &asts {
         println!("{}:", key);
         println!("{}", value);
         let action_path = folder_path.join(Path::new(&(String::new() + key + ".rs")));
         if !action_path.exists() {
             let mut f = File::create(action_path).expect("Unable to create file");
             let s: String = value.to_string();
             f.write_all(s.as_bytes()).expect("Unable to write to file");
         }
     }

     let ast = bt_to_ast(&tree);
     println!("{}", ast);

    {
        let mut f = File::create(logic_path).expect("Unable to create file");
        let s: String = ast.to_string();
        f.write_all(s.as_bytes()).expect("Unable to write to file");
    }

     return Ok(())
}
