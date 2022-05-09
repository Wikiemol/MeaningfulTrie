use std::collections::HashMap;
use std::hash::Hash;
use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;

type TrieNodeRef = usize;
struct TrieNode {
    data: Option<char>, 
    count: usize,
    parents: HashSet<TrieNodeRef>,
    children: HashSet<TrieNodeRef>,
}

pub struct Trie {
    nodes: Rc<RefCell<Vec<Rc<RefCell<TrieNode>>>>>,
    root: TrieNodeRef
}


impl Clone for TrieNode {
    fn clone(&self) -> Self {
        TrieNode {
            data: self.data,
            count: self.count,
            parents: HashSet::new(),
            children: self.children.clone()
        }
    }
}

impl Trie {

    #[inline(always)]
    fn data(&self) -> Option<char> {
        self.get_node(self.root).borrow().data
    }

    #[inline(always)]
    fn count(&self) -> usize {
        self.get_node(self.root).borrow().count
    }

    #[inline(always)]
    fn children(&self) -> Vec<Rc<RefCell<TrieNode>>> {
        self.get_nodes(self.get_root().borrow().children.clone())
    }

    #[inline(always)]
    fn parents(&self) -> Vec<Rc<RefCell<TrieNode>>> {
        self.get_nodes(self.get_root().borrow().parents.clone())
    }

    fn _print(&self, prefixes: Vec<String>, final_child: bool, print_prefix: bool) {
        let chr = match self.data() {
            None => "<root>".to_string(),
            Some(chr) => chr.to_string()
        };
        let children = self.children();
        let count = self.count();
        let has_single_child = children.len() == 1 && children[0].borrow().count == count;

        print!("{}{}{}{}{}", 
               if print_prefix { prefixes.join("") } else { "".to_string() }, 
               chr, 
               if has_single_child { "" } else { ", "},  
               if has_single_child { "".to_string() } else {  count.to_string() },
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
        for &child in &self.get_root().borrow().children {
            if i < self.children().len() - 1 {
                let mut child_prefix = new_prefix.clone();
                if !has_single_child {
                    child_prefix.push("├──────\t".to_string());
                }
                let trie = Trie {
                    nodes: self.nodes.clone(),
                    root: child
                };
                trie._print(child_prefix, false, !has_single_child);
            } else {
                let mut child_prefix = new_prefix.clone();
                if !has_single_child {
                    child_prefix.push("└──────\t".to_string());
                }
                let trie = Trie {
                    nodes: self.nodes.clone(),
                    root: child
                };
                trie._print(child_prefix, true, !has_single_child);
            }
            i += 1;
        }

    }

    pub fn print(&self) {
        self._print(vec!["".to_string()], true, false);
    }

    pub fn suffix(text: &str, max_depth: Option<usize>) -> Trie {
        let mut trie = Trie::create();
        let root_pointer = TriePointer {
            position: trie.root,
            depth: 0
        };
        let mut current_positions: HashSet<TriePointer> = HashSet::new();
        current_positions.insert(root_pointer);

        for character in text.chars() {
            let mut child_map: HashMap<Option<TrieNodeRef>, TrieNodeRef> = HashMap::new();
            for parent_pointer in current_positions.clone() {
                let TriePointer {position: parent_position, depth} = parent_pointer;
                match trie.find_child(parent_position, character) {
                    None => {
                        match child_map.get(&None) {
                            None => {
                                let child_ref = trie.append_character(parent_position, character);
                                let child_pointer = TriePointer {
                                    position: child_ref,
                                    depth: depth + 1
                                };
                                current_positions.insert(child_pointer);
                                assert!(current_positions.remove(&parent_pointer));
                                child_map.insert(None, child_ref);
                            }
                            Some(&child) => {
                                trie.add_parent(child, parent_position);
                                trie.add_child(parent_position, child);
                                assert!(current_positions.remove(&parent_pointer));

                                let child_pointer = TriePointer {
                                    position: child,
                                    depth: depth + 1
                                };

                                let existing_pointer = current_positions.get(&child_pointer);
                                if child_pointer.depth >= existing_pointer.map(|p| p.depth).unwrap_or(0) {
                                    current_positions.replace(child_pointer);
                                }
                            }
                        };
                    }
                    Some(child) => {
                        match child_map.get(&Some(child)) {
                            None => {
                                let cloned_child = trie.clone_node(child);
                                trie.increment_count(cloned_child);
                                trie.replace_child(parent_position, child, cloned_child);
                                let child_pointer = TriePointer {
                                    position: cloned_child,
                                    depth: depth + 1
                                };
                                current_positions.insert(child_pointer);
                                assert!(current_positions.remove(&parent_pointer));
                                child_map.insert(Some(child), cloned_child);
                            }
                            Some(&existing_child) => {
                                trie.replace_child(parent_position, child, existing_child);
                                let child_pointer = TriePointer {
                                    position: existing_child,
                                    depth: depth + 1
                                };
                                let existing_pointer = current_positions.get(&child_pointer);
                                if child_pointer.depth >= existing_pointer.map(|p| p.depth).unwrap_or(0) {
                                    current_positions.replace(child_pointer);
                                }
                                assert!(current_positions.remove(&parent_pointer));
                            }
                        }
                    }
                }
            }
            for pointer in current_positions.clone() {
                let TriePointer {depth, ..} = pointer;
                // println!("{}", depth);
                if depth > max_depth.unwrap_or(usize::MAX) { current_positions.remove(&pointer); }
            }
            current_positions.insert(TriePointer {
                position: trie.root,
                depth: 0
            });
        }

        trie
    }

    fn create() -> Trie {
        let root = Rc::new(RefCell::new(TrieNode {
            data: None,
            count: 0,
            parents: HashSet::new(),
            children: HashSet::new()
        }));

        Trie {
            nodes: Rc::new(RefCell::new(vec![root.clone()])),
            root: 0
        }
    }
    
    /// Creates a new node that is a copy of the old node and returns the index of that node
    /// Parents are emptied
    fn clone_node(&mut self, idx: TrieNodeRef) -> TrieNodeRef {
        let mut clone = self.get_node(idx).borrow().clone();
        self.nodes.borrow_mut().push(Rc::new(RefCell::new(clone)));
        return self.nodes.borrow().len() - 1;
    }

    fn create_node(&mut self, node: &Rc<RefCell<TrieNode>>) -> TrieNodeRef {
        self.nodes.borrow_mut().push(node.clone());
        self.nodes.borrow().len() - 1
    }

    #[inline(always)]
    fn increment_count(&mut self, idx: TrieNodeRef) {
        self.get_node(idx).borrow_mut().count += 1;
    }

    #[inline(always)]
    fn add_child(&mut self, parent: TrieNodeRef, child: TrieNodeRef) {
        self.get_node(parent).borrow_mut().children.insert(child);
    }

    #[inline(always)]
    fn add_parent(&mut self, child: TrieNodeRef, parent: TrieNodeRef) {
        self.get_node(child).borrow_mut().parents.insert(parent);
    }

    #[inline(always)]
    fn get_nodes(&self, indices: impl IntoIterator<Item = TrieNodeRef>) -> Vec<Rc<RefCell<TrieNode>>> {
        indices.into_iter().map(|trie_ref| self.get_node(trie_ref)).collect::<Vec<_>>()
    }
    #[inline(always)]
    fn get_node(&self, idx: TrieNodeRef) -> Rc<RefCell<TrieNode>> {
        self.nodes.borrow()[idx].clone()
    }
    #[inline(always)]
    fn get_root(&self ) -> Rc<RefCell<TrieNode>> {
        self.get_node(self.root)
    }

    #[inline(always)]
    fn find_child(&self, idx: TrieNodeRef, character: char) -> Option<TrieNodeRef> {
        self.get_node(idx).borrow().children.iter()
            .find(|&&child| self.get_node(child).borrow().data == Some(character))
            .map(|&child| child)
    }

    #[inline(always)]
    fn remove_child(&mut self, parent: TrieNodeRef, child: TrieNodeRef) {
        self.get_node(parent).borrow_mut().children.remove(&child);
    }
    
    #[inline(always)]
    fn replace_child(&mut self, parent: TrieNodeRef, old_child: TrieNodeRef, new_child: TrieNodeRef) {
        self.add_parent(new_child, parent);
        self.remove_child(parent, old_child);
        self.add_child(parent, new_child);
    }

    fn append_character(&mut self, idx: TrieNodeRef, character: char) -> TrieNodeRef {
        let child = Rc::new(RefCell::new(TrieNode {
            data: Some(character),
            count: 1,
            parents: vec![idx].iter().map(|&idx| idx).collect::<HashSet<_>>(),
            children: HashSet::new()
        }));
        let node_reference = self.create_node(&child);
        self.add_child(idx, node_reference);
        node_reference
    }

}



struct TriePointer {
    position: TrieNodeRef,
    depth: usize
}
impl Clone for TriePointer {
    fn clone(&self) -> Self {
        TriePointer {
            position: self.position,
            depth: self.depth
        }
    }
}


impl PartialEq for TriePointer { 
    fn eq(&self, other: &TriePointer) -> bool {
        self.position == other.position
    }
}
impl Eq for TriePointer { }


impl Hash for TriePointer {

    fn hash<H>(&self, hasher: &mut H) where H: std::hash::Hasher { 
        hasher.write_usize(self.position);
    }
}

