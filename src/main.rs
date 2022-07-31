
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

    let test_text = "Welcome!";


    let suffix_trie = Trie::suffix(test_text, Some(100));
                                   
    suffix_trie.debug();


    let context: UIContext = Window::context().unwrap();
    start_display(context, suffix_trie);
}

