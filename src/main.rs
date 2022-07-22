
extern crate ncurses;
use crate::trie::TrieBuilder;
use crate::display::window::UIContext;
use crate::display::window::Window;
use crate::display::trie::start_display;
use crate::tree::LazyTreeZipper;
use crate::display::*;
use std::cmp::max;
use std::rc::Rc;
use ncurses::*;
use std::fs;
mod trie;
use trie::Trie;
mod lazy;
mod tree;
use crate::tree::LazyTree;
mod display;



fn main() {
    let test_file = "./heart_sutra.txt";
    // let test_file = "./is_you_is.txt";
    // let test_file = "./tale_of_two_cities.txt";
    // let test_file = "./genesis.txt";

    let test_text = fs::read_to_string(test_file)
        .expect("Unable to read file")
        .replace("\n", "\\n")
        // .replace(" ", "xn");
        .to_lowercase();

    // let test_text = "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light, it was the season of Darkness, it was the spring of hope, it was the winter of despair, we had everything before us, we had nothing before us, we were all going direct to Heaven, we were all going direct the other way – in short, the period was so far like the present period, that some of its noisiest authorities insisted on its being received, for good or for evil, in the superlative degree of comparison only.".to_lowercase();
    // let test_text = "test the test text".to_lowercase();
    // let test_text = "asasasasasasas".to_lowercase();
    // let test_text = "Is you, is or, is you ain't my baby, Maybe baby's found somebody new, Or is my baby still my baby true, Is you, is or, is you ain't my baby, The way you acting lately makes me down, Youse is still my baby, baby, Seems my flame in your heart's done gone out".to_lowercase();
    //


    let test_text = "welcome!";


    // string_to_trie(&test_text, Some(20)).borrow().print();
    let suffix_trie = TrieBuilder::suffix(&test_text, Some(100));

    // suffix_trie.print();
    let context: UIContext = Window::context().unwrap();
    start_display(context, suffix_trie);
}
