use std::io;
use std::collections::HashMap;

use proc_macro2::{TokenStream, Ident, Span};
//use quote::TokenStreamExt;

#[derive(Clone, Debug)]
pub enum BTNodeType {
    Root(Option<Box<BTNode>>),
    Sequence(Vec<BTNode>),
    Fallback(Vec<BTNode>),
    Action,
    Condition,
}

#[derive(Clone, Debug)]
pub struct BTNode {
    pub node_type: BTNodeType,
    pub name: String,
    pub parent: Option<String>,
}

impl BTNode {
    pub fn get_ident(&self) -> String
    {
        match self.node_type {
            BTNodeType::Root(_) => format!("root"),
            BTNodeType::Sequence(_) => format!("sequence_{}", self.name),
            BTNodeType::Fallback(_) => format!("fallback_{}", self.name),
            BTNodeType::Action => format!("action_{}", self.name),
            BTNodeType::Condition => format!("condition_{}", self.name),
        }
    }
}

pub type BTGraph = BTNodeType;

fn add_to_parent(mut parent: BTNode, mut nodes: &mut Vec<BTNode>) -> Result<BTNode, io::Error>
{
    let children: Vec<_> = nodes.drain_filter(|x| x.parent.as_ref().map_or(false, |n| &parent.name == n)).collect();

    for child in &children
    {
        let new_child = add_to_parent(child.clone(), &mut nodes)?;
        match parent.node_type {
            BTNodeType::Root(ref mut r) => {
                r.replace(Box::new(new_child));
            },
            BTNodeType::Sequence(ref mut s) => {
                s.push(new_child);
            },
            BTNodeType::Fallback(ref mut f) => {
                f.push(new_child);
            },
            _ => (), // Actually should be an error if parent
        }
    }

    return Ok(parent);
}

pub fn bt_from_nodes(nodes: &Vec<BTNode>) -> Result<BTGraph, io::Error>
{
    let mut remaining = nodes.clone();

    let root = match nodes.iter().find(|&x| x.name == "root") {
        Some(r) => r,
        None => return Ok(BTNodeType::Action) //Err(io::Error::new(1,"tjena"))
    };

    let new_root = add_to_parent(root.clone(), &mut remaining)?;

    return Ok(new_root.node_type);
}

fn action_node_to_ast(name: &str) -> TokenStream
{
    let token_name = Ident::new(name, Span::call_site());

    let tokens = quote! {
        
        pub async fn #token_name() -> bool
        {
            println!("Stepping through {}...", #name);

            return true;
        }

    };

    return tokens;
}

fn bt_node_to_action_asts(node: &BTNode) -> HashMap<String, TokenStream>
{
    let mut asts = HashMap::new();

    match node.node_type {
        BTNodeType::Root(ref r) => {
            if let Some(child) = r {
                asts.extend(bt_node_to_action_asts(&child));
            }
        },
        BTNodeType::Sequence(ref s) => {
            for child in s {
                asts.extend(bt_node_to_action_asts(child));
            }
        },
        BTNodeType::Fallback(ref f) => {
            for child in f {
                asts.extend(bt_node_to_action_asts(child));
            }
        },
        _ => {
            asts.insert(node.get_ident(), action_node_to_ast(&node.get_ident()));
        },
    }

    return asts;
}

pub fn bt_to_action_asts(bt: &BTGraph) -> HashMap<String, TokenStream>
{
    if let BTNodeType::Root(Some(ref node)) = bt
    {
        return bt_node_to_action_asts(node);
    }
    else {
        return HashMap::new();
    }
}

fn root_node_to_ast(node: &BTNode) -> TokenStream
{
    //let name = node.name.clone();
    let name = Ident::new(&node.get_ident(), Span::call_site());

    let tokens = quote! {
        
        pub async fn  #name() -> bool
        {
            return true;
        }

    };

    return tokens;
}

fn sequence_node_to_ast(node: &BTNode) -> TokenStream
{
    let print_name = node.name.clone();
    let name = Ident::new(&node.get_ident(), Span::call_site());

    let mut parts = TokenStream::new();

    if let BTNodeType::Sequence(ref s) = node.node_type {
        for child in s {
            //let name = &child.name;
            let name = Ident::new(&child.get_ident(), Span::call_site());
            let part = quote! {
                if ! await!(#name()) {
                    return false;
                }
            };
            parts.extend(part);
        }
    }

    let tokens = quote! {
        
        pub async fn #name() -> bool
        {
            println!("Stepping through {}...", #print_name);

            #parts

            return true;
        }

    };

    return tokens;
}

fn fallback_node_to_ast(node: &BTNode) -> TokenStream
{
    //let name = node.name.clone();
    let print_name = node.name.clone();
    let name = Ident::new(&node.get_ident(), Span::call_site());

    let mut parts = TokenStream::new();

    if let BTNodeType::Fallback(ref f) = node.node_type {
        for child in f {
            //let name = &child.name;
            let name = Ident::new(&child.get_ident(), Span::call_site());
            let part = quote! {
                if await!(#name()) {
                    return true;
                }
            };
            parts.extend(part);
        }
    }

    let tokens = quote! {
        
        pub async fn #name() -> bool
        {
            println!("Stepping through {}...", #print_name);

            #parts

            return false;
        }

    };

    return tokens;
}

fn bt_node_to_ast(node: &BTNode) -> TokenStream
{
    let mut asts = TokenStream::new();

    match node.node_type {
        BTNodeType::Root(ref r) => {
            asts.extend(root_node_to_ast(&node));
            if let Some(child) = r {
                asts.extend(bt_node_to_ast(&child));
            }
        },
        BTNodeType::Sequence(ref s) => {
            asts.extend(sequence_node_to_ast(&node));
            for child in s {
                asts.extend(bt_node_to_ast(child));
            }
        },
        BTNodeType::Fallback(ref f) => {
            asts.extend(fallback_node_to_ast(&node));
            for child in f {
                asts.extend(bt_node_to_ast(child));
            }
        },
        _ => (),
    }

    return asts;
}

fn bt_node_to_imports(node: &BTNode) -> TokenStream
{
    let mut asts = TokenStream::new();

    match node.node_type {
        BTNodeType::Root(ref r) => {
            if let Some(child) = r {
                asts.extend(bt_node_to_imports(&child));
            }
        },
        BTNodeType::Sequence(ref s) => {
            for child in s {
                asts.extend(bt_node_to_imports(child));
            }
        },
        BTNodeType::Fallback(ref f) => {
            for child in f {
                asts.extend(bt_node_to_imports(child));
            }
        },
        _ => {
            let name = Ident::new(&node.get_ident(), Span::call_site());
            let tokens = quote! {
                use crate::#name::#name;
            };
            asts.extend(tokens);
        },
    }

    return asts;
}

pub fn bt_to_ast(bt: &BTGraph) -> TokenStream
{
    if let BTNodeType::Root(Some(ref node)) = bt {
        let mut tokens = bt_node_to_imports(node);
        tokens.extend(bt_node_to_ast(node));
        return tokens;
    }
    else {
        return TokenStream::new();
    }
}
