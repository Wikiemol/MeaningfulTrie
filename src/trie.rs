use std::collections::BTreeSet;
use crate::tree::EditNodeResult;
use crate::tree::EditNodeError;
use crate::tree::BiDirectionalTree;
use std::collections::HashMap;
use std::hash::Hash;
use std::collections::HashSet;

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


pub type Trie = BiDirectionalTree<_TrieNode>;


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


impl Trie {

    pub fn debug(&self) {

        for (node_ref, node) in self.nodes.iter().enumerate() {

            println!("Ref: {}, Value: {},\tCount: {}, \tParents: {:<10}, \tChildren: {:<10}", 
                     node_ref, 
                     node.value.data.map(|x| x.to_string()).unwrap_or("<root>".to_string()), 
                     format!("{:?}", node.value.count),
                     format!("{:?}", node.parents),
                     format!("{:?}", node.children),
            );
        }
    }

    /// Returns the corresponding character for the node,
    /// except in the case when it is the root node, in which case
    /// it will return the empty string.
    pub fn get_string(&self, node_ref: TrieNodeRef) -> String {
        self.get_value(node_ref).data
           .map(|x| x.to_string())
           .unwrap_or("".to_string())
    }

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

    pub fn find_longest_path_to_node(&self, idx: TrieNodeRef) -> Vec<TrieNodeRef> {
        match self.find_maximum_depth_parent(idx) {
            None => {
                assert_eq!(self.root, idx);
                vec![]
            },
            Some(parent) => {
                let mut result = self.find_longest_path_to_node(parent);
                result.push(idx);
                result
            }
        }
    }

    pub fn find_maximum_depth_parent(&self, idx: TrieNodeRef) -> Option<TrieNodeRef> {
        let mut maximum_parent = None;
        for parent in self.get_node(idx).parents.iter() {
            let parent_node = self.get_node(*parent);
            match maximum_parent {
                None => {
                    maximum_parent = Some(*parent);
                }
                Some(current_maximum_ref) => {
                    let current_maximum = self.get_node(current_maximum_ref);
                    if current_maximum.value.max_depth < parent_node.value.max_depth {
                        maximum_parent = Some(*parent);
                    }
                }
            }
        }
        maximum_parent
    }


    /// Creates a suffix trie from the given input string.
    ///
    /// This suffix trie has some unique properties though.
    /// It stores on each node the number of occurences of the string up until that point
    /// It also stores the maximum depth that that particular subtree is found in the tree
    /// It also stores the ids of the parents of all nodes
    pub fn suffix(text: &str, max_depth: Option<usize>) -> Trie {
        let mut trie = Trie::new(_TrieNode {
            data: None,
            count: 0,
            max_depth: 0,
        });
        let root_pointer = TriePointer {
            position: trie.root,
            depth: 0
        };
        
        // Using a HashSet can make this function technically impure.
        // Specifically, the specific node_refs in the tree can change. 
        // Often, when debugging, its useful to change to a BTreeSet
        let mut current_positions: HashSet<TriePointer> = HashSet::new();
        current_positions.insert(root_pointer);

        for character in text.chars() {
            let mut child_map: HashMap<Option<TrieNodeRef>, TrieNodeRef> = HashMap::new();
            // println!("=========================================");
            for parent_pointer in current_positions.clone() {

                let TriePointer {position: parent_position, depth} = parent_pointer;
                match trie.find_child(parent_position, character) {
                    None => {
                        match child_map.get(&None) {
                            None => {
                                match trie.append_character(parent_position, character) {
                                    Ok(child_ref) => {
                                        // println!("appended character");
                                        let child_pointer = TriePointer {
                                            position: child_ref,
                                            depth: depth + 1
                                        };
                                        current_positions.insert(child_pointer);
                                        assert!(current_positions.remove(&parent_pointer));
                                        child_map.insert(None, child_ref);
                                    },
                                    Err(EditNodeError::NodeDeleted(_)) => {}
                                }
                            }
                            Some(&child) => {
                                match trie.add_child_ref(parent_position, child) {
                                    Ok(child) => {
                                        // println!("add child");
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
                                    Err(EditNodeError::NodeDeleted(parent)) => {}
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
                                        // println!("replace child {} of parent {} with clone {}", 
                                                        // child, parent_position, cloned_child);
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
                                    Ok(existing_child) => {
                                        // println!("replace child with existing");
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
                // trie.debug();
                // let mut s = "".to_string();
                // std::io::stdin().read_line(&mut s);
                // println!("--------------------------------");
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
impl PartialOrd for TriePointer {

    fn partial_cmp(&self, other: &TriePointer) -> std::option::Option<std::cmp::Ordering> { 
        self.position.partial_cmp(&other.position)
    }
}

impl Ord for TriePointer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { 
        self.position.cmp(&other.position)
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

