use std::collections::HashMap;
use std::hash::Hash;
use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;

pub type TrieNodeRef = usize;
pub struct TrieNode {
    data: Option<char>, 
    count: usize,
    max_depth: usize,
    parents: HashSet<TrieNodeRef>,
    children: HashSet<TrieNodeRef>,
}

pub struct Trie {
    nodes: Rc<Vec<Rc<TrieNode>>>,
    root: TrieNodeRef
}
struct TrieBuilder {
    nodes: Vec<Rc<RefCell<TrieNode>>>,
    root: TrieNodeRef
}

impl Clone for Trie  {

    fn clone(&self) -> Self { 
        Trie {
            nodes: self.nodes.clone(),
            root: self.root
        }
    }
}


impl Clone for TrieNode {
    fn clone(&self) -> Self {
        TrieNode {
            data: self.data,
            count: self.count,
            max_depth: self.max_depth,
            parents: HashSet::new(),
            children: self.children.clone()
        }
    }
}

struct PrintParamaters {
    prefixes: Vec<String>, 
    final_child: bool, 
    print_prefix: bool, 
    prev_max_depth: usize
}

impl Trie {

    #[inline(always)]
    pub fn get_nodes(&self, indices: impl IntoIterator<Item = TrieNodeRef>) -> Vec<Rc<TrieNode>> {
        indices.into_iter().map(|trie_ref| self.get_node(trie_ref)).collect::<Vec<_>>()
    }
    #[inline(always)]
    pub fn get_node(&self, idx: TrieNodeRef) -> Rc<TrieNode> {
        self.nodes[idx].clone()
    }
    #[inline(always)]
    pub fn get_root(&self ) -> Rc<TrieNode> {
        self.get_node(self.root)
    }
    #[inline(always)]
    pub fn data(&self) -> Option<char> {
        self.get_node(self.root).data
    }

    #[inline(always)]
    pub fn count(&self) -> usize {
        self.get_node(self.root).count
    }

    #[inline(always)]
    pub fn max_depth(&self) -> usize {
        self.get_node(self.root).max_depth
    }
    #[inline(always)]
    pub fn children(&self) -> Vec<Trie> {
        self.get_root().children.
            iter()
            .map(|&child| Trie {
                nodes: self.nodes.clone(),
                root: child
            })
        .collect::<Vec<_>>()
    }

    #[inline(always)]
    fn _children(&self) -> Vec<Rc<TrieNode>> {
        self.get_nodes(self.get_root().children.clone())
    }

    #[inline(always)]
    pub fn parents(&self) -> Vec<Rc<TrieNode>> {
        self.get_nodes(self.get_root().parents.clone())
    }


    // fn _print(&self, prefixes: Vec<String>, final_child: bool, print_prefix: bool, prev_max_depth: usize) {
    fn _print(&self, parameters: &PrintParamaters) {
        let PrintParamaters {prefixes, print_prefix,  prev_max_depth, final_child} = parameters;
        let chr = match self.data() {
            None => "<root>".to_string(),
            Some(chr) => chr.to_string()
        };
        let children = self._children();
        let count = self.count();
        let has_single_child = children.len() == 1 && children[0].count == count;
        let max_depth = if has_single_child { *prev_max_depth } else { self.max_depth() };

        print!("{}{}{}{}{}{}", 
               if *print_prefix { prefixes.join("") } else { "".to_string() }, 
               chr, 
               if has_single_child { "" } else { ", "},  
               if has_single_child { "".to_string() } else {  count.to_string() },
               if has_single_child { "".to_string() } else {  "|".to_string() + &max_depth.to_string() },
               if has_single_child { "" } else { "\n" });


        let new_prefix: Vec<String> = 
            if !print_prefix || prefixes.len() == 0 {
                prefixes.to_vec()
            } else {
                let mut a = prefixes[0..(prefixes.len() - 1)].to_vec();
                a.push(
                    prefixes[prefixes.len() - 1]
                        .chars()
                        .filter(|&c| c == '\t')
                        .flat_map(|_| if *final_child {
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
        for &child in &self.get_root().children {
            if i < self._children().len() - 1 {
                let mut child_prefix = new_prefix.clone();
                if !has_single_child {
                    child_prefix.push("├──────\t".to_string());
                }
                let trie = Trie {
                    nodes: self.nodes.clone(),
                    root: child
                };
                trie._print(&PrintParamaters { 
                    prefixes: child_prefix, 
                    final_child: false, 
                    print_prefix: !has_single_child, 
                    prev_max_depth: self.max_depth()
                });
            } else {
                let mut child_prefix = new_prefix.clone();
                if !has_single_child {
                    child_prefix.push("└──────\t".to_string());
                }
                let trie = Trie {
                    nodes: self.nodes.clone(),
                    root: child
                };
                trie._print(&PrintParamaters {
                    prefixes: child_prefix, 
                    final_child: true, 
                    print_prefix: !has_single_child, 
                    prev_max_depth: max_depth
                });
            }
            i += 1;
        }

    }

    pub fn print(&self) {
        self._print(&PrintParamaters {
            prefixes: vec!["".to_string()], 
            final_child: true, 
            print_prefix: false, 
            prev_max_depth: 0
        });
    }

    pub fn suffix(text: &str, max_depth: Option<usize>) -> Trie {
        let mut trie = Trie::builder();
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
                                    trie.set_max_depth(child, child_pointer.depth);
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
                                trie.set_max_depth(cloned_child, child_pointer.depth);
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
                                    trie.set_max_depth(existing_child, child_pointer.depth);
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
        trie.build()
    }

    fn builder() -> TrieBuilder {
        let root = Rc::new(RefCell::new(TrieNode {
            data: None,
            count: 0,
            max_depth: 0,
            parents: HashSet::new(),
            children: HashSet::new()
        }));

        TrieBuilder {
            nodes: vec![root.clone()],
            root: 0
        }
    }
    

}

impl TrieBuilder {
    /// Creates a new node that is a copy of the old node and returns the index of that node
    /// Parents are emptied
    fn clone_node(&mut self, idx: TrieNodeRef) -> TrieNodeRef {
        let mut clone = self.get_node(idx).borrow().clone();
        self.nodes.push(Rc::new(RefCell::new(clone)));
        return self.nodes.len() - 1;
    }

    fn create_node(&mut self, node: &Rc<RefCell<TrieNode>>) -> TrieNodeRef {
        self.nodes.push(node.clone());
        self.nodes.len() - 1
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
        self.nodes[idx].clone()
    }
    #[inline(always)]
    fn get_root(&self ) -> Rc<RefCell<TrieNode>> {
        self.get_node(self.root)
    }

    #[inline(always)]
    fn set_max_depth(&mut self, idx: TrieNodeRef, max_depth: usize) {
        self.get_node(idx).borrow_mut().max_depth = max_depth;
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
            max_depth: self.get_node(idx).borrow().max_depth + 1,
            parents: vec![idx].iter().map(|&idx| idx).collect::<HashSet<_>>(),
            children: HashSet::new()
        }));
        let node_reference = self.create_node(&child);
        self.add_child(idx, node_reference);
        node_reference
    }

    fn build(&self) -> Trie {
        Trie {
            nodes: Rc::new(self.nodes.iter().map(|node| Rc::new(node.borrow().clone())).collect::<Vec<_>>()),
            root: self.root
        }
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

