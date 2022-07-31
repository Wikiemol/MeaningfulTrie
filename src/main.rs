
extern crate ncurses;
use crate::display::trie::render::start_display;
use crate::trie::Trie;
use crate::display::window::UIContext;
use crate::display::window::Window;
use crate::tree::LazyTreeZipper;
use crate::display::*;
use std::cmp::max;
use std::rc::Rc;
use ncurses::*;
use std::fs;

use rand::{self, Rng};
use rand::distributions::{Alphanumeric, Uniform, Standard};

mod trie;
mod lazy;
mod tree;
use crate::tree::LazyTree;

mod display;



fn main() {

    // let test_text = "Welcome!";
    let test_file = "./Quantum.txt";

    let test_text = fs::read_to_string(test_file)
     .expect("Unable to read file")
     .replace("\n", "\\n")
     // .replace(" ", "xn");
     .to_lowercase();


    // println!("Length: {}", test_text.len());

    let suffix_trie = Trie::suffix(&test_text, Some(100));
                                   
    // suffix_trie.debug();


    // let context: UIContext = Window::context().unwrap();
    // start_display(context, suffix_trie);
}

