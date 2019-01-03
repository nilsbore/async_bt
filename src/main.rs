#![feature(drain_filter)]

#[macro_use] extern crate lalrpop_util;

#[macro_use] extern crate quote;
extern crate proc_macro2;

use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

pub mod bt;

use crate::bt::{BTNode, BTNodeType, bt_from_nodes, bt_to_action_asts, bt_to_ast};

lalrpop_mod!(pub bt_parser); // synthesized by LALRPOP

fn main() -> Result<(), std::io::Error> {
    println!("Hello, world!");

    if let Ok(n) = bt_parser::NodesParser::new().parse("Root root")
    {
        println!("{:?}", n);
    }
    /*if let Err(e) = bt_parser::NodesParser::new().parse("Action a1, a2, a3, a4 -> s1")
    {
        println!("{}", e);
    }*/
    if let Ok(n) = bt_parser::NodesParser::new().parse("Action a1, a2, a3, a4 -> s1")
    {
        println!("{:?}", n);
    }
    if let Ok(n) = bt_parser::NodesParser::new().parse("Condition c1 -> f1")
    {
        println!("{:?}", n);
    }

    let mut nodes = Vec::new();

    let f = File::open("examples/noc_example.txt").unwrap();
    let file = BufReader::new(&f);
    for line in file.lines()
    {
        let l = line.unwrap();

        if l.len()  == 0
        {
            continue;
        }

        match bt_parser::NodesParser::new().parse(&l)
        {
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

     for (key, value) in &asts
     {
         println!("{}:", key);
         println!("{}", value);
     }

     let ast = bt_to_ast(&tree);
     println!("{}", ast);


     return Ok(())
}
