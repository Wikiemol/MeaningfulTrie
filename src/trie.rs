use crate::tree::EditNodeResult;
use crate::tree::BiDirectionalTree;
use crate::tree::EditNodeError;
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

#[derive(Clone)]
pub struct _TrieNode {
    pub data: Option<char>, 
    pub count: usize,
    pub max_depth: usize,
}


pub struct Trie {
    nodes: Rc<Vec<Rc<TrieNode>>>,
    root: TrieNodeRef
}
pub type TrieBuilder = BiDirectionalTree<_TrieNode>;

pub struct VirtualTrieNode<'a> {
    nodes: &'a Vec<TrieNode>,
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
    pub fn text(&self) -> Option<char> {
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
        let chr = match self.text() {
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

    

}

impl TrieBuilder {

    /// Returns the corresponding character for the node,
    /// except in the case when it is the root node, in which case
    /// it will return the empty string.
    pub fn get_string(&self, node_ref: TrieNodeRef) -> String {
        self.get_value(node_ref).data
           .map(|x| x.to_string())
           .unwrap_or("".to_string())
    }

    // pub fn get_parents(&self, node_ref: TrieNodeRef) -> Vec<String> {
    //     let node = self.get_node(node_ref);

    //     let grand_parents: Vec<String> = node.parents
    //         .iter()
    //             .flat_map(|&parent| 
    //               self.get_parents(parent)
    //                 .into_iter()
    //                     .map(|string|  string + &self.get_string(parent) + &parent.to_string())
    //                 .collect::<Vec<String>>()
    //             )
    //         .collect();

    //     if grand_parents.len() == 0 {
    //         vec!["".to_string()]
    //     } else {
    //         grand_parents
    //     }
    // }

    #[inline(always)]
    pub fn increment_count(&mut self, idx: TrieNodeRef) {
        self.get_value_mut(idx).count += 1;
    }


    pub fn set_max_depth(&mut self, idx: TrieNodeRef, max_depth: usize) {
        self.get_value_mut(idx).max_depth = max_depth;
    }

    #[inline(always)]
    pub fn find_child(&self, idx: TrieNodeRef, character: char) -> Option<TrieNodeRef> {
        self.get_node(idx).children.iter()
            .find(|&&child| self.get_value(child).data == Some(character))
            .map(|&child| child)
    }

    
    pub fn replace_child_ref(&mut self, parent: TrieNodeRef, old_child: TrieNodeRef, new_child: TrieNodeRef) -> EditNodeResult {
        let result = self.add_child_ref(parent, new_child);
        self.remove_child(parent, old_child);
        result
    }

    pub fn append_character(&mut self, idx: TrieNodeRef, character: char) -> EditNodeResult {
        let child = _TrieNode {
            data: Some(character),
            count: 1,
            max_depth: self.get_value(idx).max_depth + 1
        };
        self.add_child(idx, child)
    }


    /// Creates a suffix trie from the given input string.
    ///
    /// This suffix trie has some unique properties though.
    /// It stores on each node the number of occurences of the string up until that point
    /// It also stores the maximum depth that that particular subtree is found in the tree
    /// It also stores the ids of the parents of all nodes
    pub fn suffix(text: &str, max_depth: Option<usize>) -> TrieBuilder {
        let mut trie = TrieBuilder::new(_TrieNode {
            data: None,
            count: 0,
            max_depth: 0,
        });
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
                                match trie.append_character(parent_position, character) {
                                    Ok(child_ref) => {
                                        let child_pointer = TriePointer {
                                            position: child_ref,
                                            depth: depth + 1
                                        };
                                        current_positions.insert(child_pointer);
                                        assert!(current_positions.remove(&parent_pointer));
                                        child_map.insert(None, child_ref);
                                    },
                                    Err(_) => panic!()
                                }
                            }
                            Some(&child) => {
                                match trie.add_child_ref(parent_position, child) {
                                    Ok(_) => {
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
                                    Err(_) => panic!()
                                }
                            }
                        };
                    }
                    Some(child) => {
                        match child_map.get(&Some(child)) {
                            None => {
                                // Cannot just increment the count. 
                                // Another string could have the same suffix trie as a child
                                // but have been seen less often.
                                let cloned_child = trie.clone(child);
                                let mut cloned_value =  trie.get_value_mut(cloned_child);
                                cloned_value.count += 1;

                                let cloned_child = trie.replace_child_ref(parent_position, child, cloned_child);
                                match cloned_child {
                                    Ok(cloned_child) => {
                                        let child_pointer = TriePointer {
                                            position: cloned_child,
                                            depth: depth + 1
                                        };
                                        trie.set_max_depth(cloned_child, child_pointer.depth);
                                        current_positions.insert(child_pointer);
                                        assert!(current_positions.remove(&parent_pointer));
                                        child_map.insert(Some(child), cloned_child);
                                    }
                                    Err(_) => panic!()
                                }
                            }
                            Some(&existing_child) => {
                                match trie.replace_child_ref(parent_position, child, existing_child) {
                                    Ok(_) => {
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
                                    Err(_) => panic!()
                                }
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

