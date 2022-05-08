use std::collections::HashSet;
use std::rc::Rc;
use std::rc::Weak;
use std::mem::replace;
use std::cell::RefCell;
struct Trie { 
    data: Option<char>, 
    count: usize,
    parents: Vec<Weak<RefCell<Trie>>>,
    children: Vec<Rc<RefCell<Trie>>>,
}

impl Clone for Trie {
    fn clone(&self) -> Self {
        Trie {
            data: self.data,
            count: self.count,
            parents: vec![],
            children: self.children.clone()
        }
    }
}

impl Trie {

    fn _print(&self, prefixes: Vec<String>, final_child: bool, print_prefix: bool) {
        let chr = match self.data {
            None => "<root>".to_string(),
            Some(chr) => chr.to_string()
        };
        let has_single_child = self.children.len() == 1 && self.children[0].borrow().count == self.count;

        print!("{}{}{}{}{}", 
               if print_prefix { prefixes.join("") } else { "".to_string() }, 
               chr, 
               if has_single_child { "" } else { ", "},  
               if has_single_child { "".to_string() } else {  self.count.to_string() },
               if has_single_child { "" } else { "\n" });


        let new_prefix: Vec<String> = 
            if !print_prefix || prefixes.len() == 0 {
                prefixes
            } else {
                let mut a = prefixes[0..(prefixes.len() - 1)].to_vec();
                a.push(
                    prefixes[prefixes.len() - 1]
                        .chars()
                        .filter(|&c| c == '\t')
                        .flat_map(|_| if final_child {
                                "\t".chars()
                            } else {
                                "│\t".chars()
                            }
                        )
                        .collect::<String>()
                );
                a
            };

        let mut i = 0;
        for child in &self.children {
            if i < self.children.len() - 1 {
                let mut child_prefix = new_prefix.clone();
                if !has_single_child {
                    child_prefix.push("├──────\t".to_string());
                }
                child.borrow_mut()._print(child_prefix, false, !has_single_child);
            } else {
                let mut child_prefix = new_prefix.clone();
                if !has_single_child {
                    child_prefix.push("└──────\t".to_string());
                }
                child.borrow_mut()._print(child_prefix, true, !has_single_child);
            }
            i += 1;
        }

    }

    fn print(&self) {
        self._print(vec!["".to_string()], true, false);
    }

}

fn string_to_trie(text: &str, limit: Option<usize>) -> Rc<RefCell<Trie>> {
    let root = Rc::new(RefCell::new(Trie {
        data: None,
        count: 1,
        parents: vec![],
        children: vec![]
    }));

    let mut trie_positions = vec![root.clone()];
    let mut trie_depth: Vec<usize> = vec![0];

    for character in text.chars() {
        let mut new_trie_positions: Vec<Rc<RefCell<Trie>>> = trie_positions.clone();
        let mut to_delete: Vec<bool> = new_trie_positions.iter().map(|_| false).collect::<Vec<_>>();
        let mut c_trie = None; 
        for (i, parent) in trie_positions.iter().enumerate() {
            let mut matching_idx_child = None;
            for (i, child) in parent.borrow().children.iter().enumerate() {
                if child.borrow().data == Some(character) {
                    matching_idx_child = Some((i, child.clone()));
                    break;
                }
            }
            match matching_idx_child {
                None => {
                    match c_trie {
                        None => {
                            let child = Rc::new(RefCell::new(Trie {
                                data: Some(character),
                                count: 1,
                                parents: vec![],
                                children: vec![]
                            }));
                            c_trie = Some(child.clone());

                            child.borrow_mut().parents.push(Rc::downgrade(parent));
                            parent.borrow_mut().children.push(child.clone());
                            let _ = replace(&mut new_trie_positions[i], child.clone());
                        }
                        Some(child) => {
                            child.borrow_mut().parents.push(Rc::downgrade(parent));
                            parent.borrow_mut().children.push(child.clone());
                            let _ = replace(&mut to_delete[i], true);
                            c_trie = Some(child);
                        }
                    }

                    trie_depth[i] += 1;
                }
                Some((j, child)) => {
                    let cloned_child: Rc<RefCell<Trie>> = Rc::new(RefCell::new(child.borrow().clone()));

                    cloned_child.borrow_mut().count += 1;
                    cloned_child.borrow_mut().parents.push(Rc::downgrade(parent)); 

                    let _ = replace(&mut (parent.borrow_mut().children[j]), cloned_child.clone());
                    let _ = replace(&mut new_trie_positions[i], cloned_child.clone());
                    trie_depth[i] += 1;
                }
            }
        }

        trie_positions = new_trie_positions;
        trie_positions = trie_positions
            .iter().zip(trie_depth.iter()).zip(to_delete.iter())
            // .filter(|(pointer, depth)| depth < 10)
            .filter(|((_, &depth), &to_delete)| depth <= limit.unwrap_or(usize::MAX) && !to_delete)
            .map(|((pointer, _), _)| pointer.clone())
            .collect::<Vec<_>>();
        trie_depth = trie_depth.iter().zip(to_delete.iter())
            .filter(|(&depth, &to_delete)| depth <= limit.unwrap_or(usize::MAX) && !to_delete)
            .map(|(&depth, _)| depth.clone())
            .collect::<Vec<_>>();
        trie_positions.push(root.clone());
        trie_depth.push(0);
        // let new = current.borrow().children[current.borrow().children.len() - 1].clone();
        // current = new;
    }
    return root;
}


fn main() {
    // let test_text = "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light, it was the season of Darkness, it was the spring of hope, it was the winter of despair, we had everything before us, we had nothing before us, we were all going direct to Heaven, we were all going direct the other way – in short, the period was so far like the present period, that some of its noisiest authorities insisted on its being received, for good or for evil, in the superlative degree of comparison only.".to_lowercase();
    // let test_text = "test the test text".to_lowercase();
    // let test_text = "asasasasasasasassa".to_lowercase();
    let test_text = "Body is nothing more than emptiness, emptiness is nothing more than body.  The body is exactly empty, and emptiness is exactly body.  The other four aspects of human existence -- feeling, thought, will, and consciousness -- are likewise nothing more than emptiness, and emptiness nothing more than they.  All things are empty: Nothing is born, nothing dies, nothing is pure, nothing is stained, nothing increases and nothing decreases.  So, in emptiness, there is no body, no feeling, no thought, no will, no consciousness.  There are no eyes, no ears, no nose, no tongue, no body, no mind.  There is no seeing, no hearing, no smelling, no tasting, no touching, no imagining.  There is nothing seen, nor heard, nor smelled, nor tasted, nor touched, nor imagined.  There is no ignorance, and no end to ignorance.  There is no old age and death, and no end to old age and death.  There is no suffering, no cause of suffering, no end to suffering, no path to follow.  There is no attainment of wisdom, and no wisdom to attain.  The Bodhisattvas rely on the Perfection of Wisdom, and so with no delusions, they feel no fear, and have Nirvana here and now.  All the Buddhas, past, present, and future, rely on the Perfection of Wisdom, and live in full enlightenment.  The Perfection of Wisdom is the greatest mantra.  It is the clearest mantra, the highest mantra, the mantra that removes all suffering. This is truth that cannot be doubted.".to_lowercase();


    string_to_trie(&test_text, Some(100)).borrow().print();

}




