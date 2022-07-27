use crate::Rc;
use crate::Trie;
use crate::tree::TreePath;
use crate::LazyTreeZipper;
use crate::trie::TrieNodeRef;
use crate::tree::LazyTree;
use std::fs;
use std::fs::metadata;

#[derive(Clone )]
pub struct UIStateNode {
    pub selected: bool,
    pub expanded: bool,
    pub text: String,
    pub original_refs: Vec<TrieNodeRef>,
    pub count:  usize,
    pub max_depth: usize,
    pub longest_path: Vec<TrieNodeRef>
}

pub type UITree = LazyTree<UIStateNode>;


pub enum UIMode {
    FileExplorer(FileExplorerState),
    TrieNavigation, 
    Command
}

pub struct FileExplorerState {
    pub files: LazyTreeZipper<String>,
    pub hovered_file: usize
}



pub struct UIState {
    pub tree: UITree,
    pub selected_node: TreePath,
    pub mode: UIMode,
    pub command_buffer: String,
    pub display_meaningfulness: bool
}    

pub enum MoveForwardResult {
    NotExpanded,
    Moved,
    NoMoreChildren
}

impl UIState {
    pub fn new(trie: Trie) -> UIState {
        let root = trie.root;
        UIState {
            tree: UIState::new_tree(Rc::new(trie), root, true),
            selected_node: vec![],
            mode: UIMode::TrieNavigation,
            command_buffer: "".to_string(),
            display_meaningfulness: false
        }
    }

    fn new_tree(trie: Rc<Trie>, node_ref: TrieNodeRef, selected: bool) -> UITree {

        let node = trie.get_node(node_ref);
        let mut children = node.children.iter().map(|&x| x).collect::<Vec<_>>();
        children.sort_unstable_by(|&a, &b| trie.get_value(b).count.cmp(&trie.get_value(a).count));
        let mut text = trie.get_value(node_ref).data.map(|c| c.to_string()).unwrap_or("<root>".to_string());
        let mut original_refs = vec![node_ref];
        let mut last = children.iter().last().map(|&l| trie.get_node(l));
        
        while children.len() == 1 && last.unwrap().value.count == trie.get_value(node_ref).count {
            let child = &last.unwrap();
            text += &child.value.data.unwrap().to_string();
            original_refs.push(*children.iter().last().unwrap());
            children = child.children.iter().map(|&x| x).collect::<Vec<_>>();

            last = children.iter().last().map(|&l| trie.get_node(l));
        }

        LazyTree::new(
            UIStateNode {
                selected,
                expanded: false,
                text,
                original_refs,
                count: trie.get_value(node_ref).count,
                max_depth: trie.get_value(node_ref).max_depth,
                longest_path: trie.find_longest_path_to_node(node_ref),
                    // .parents
                    // .iter()
                    // .map(|parent| trie
                    //      .get_node(*parent)
                    //      .value
                    //      .data
                    //      .map(|data| data.to_string())
                    //      .unwrap_or("<root>".to_string())
                    // )
                    // .collect()
                // parents: trie.get_parents(node_ref)
                // parents: vec![]
            },
            Rc::new(
                              move || children
                                         .iter()
                                         .map(|child| (UIState::new_tree(trie.clone(), *child, false)))
                                         .collect::<Vec<_>>()
            ))
    }

    pub fn move_to_longest_path(mut self) -> Self {
        let longest_path = self.longest_path_to_selected();
        self.selected_node = longest_path.clone();
        self.move_to_path(&longest_path)
    }

    fn longest_path_to_selected(&self) -> TreePath {
        let mut ref_path = self.get_selected_tree().value.longest_path.clone();
        ref_path.reverse();
        let mut result = vec![];

        let mut current_tree = &self.tree;
        loop {
            let head_option = ref_path.pop();
            match head_option {
                Some(head) => {
                    if !current_tree.value.original_refs.contains(&head) {
                        let (index, child) = current_tree.children()
                            .iter()
                            .enumerate()
                            .find(|(_, child)| 
                                  child
                                  .value
                                  .original_refs
                                  .contains(&head)
                            )
                            .unwrap_or_else(|| panic!("Invalid state searching for ref  {} in char {}", 
                                                      head, 
                                                      current_tree.value.text
                                                )
                            );
                        result.push(index);

                        current_tree = child;
                    } else {
                        continue;
                    }
                }
                None => { break; }
            }
        }
        result
        // self.root;
        // self.selected_node.
    }


    pub fn number_of_siblings_for_selected(&self) -> usize {
        self.get_selected_parent().map(|parent| parent.children().len()).unwrap_or(1)
    }

    pub fn number_of_children_for_selected(&self) -> usize {
        self.get_selected_tree().children().len()
    }

    /// #Returns
    /// `true` if actually moved, `false` if stayed in the same place (due to no possible moves)
    pub fn move_up(&mut self) -> bool {
        let num_siblings = self.number_of_siblings_for_selected();
        match self.selected_node.last_mut() {
            None => { false }
            Some(i) => {
                *i = clamp(i.checked_sub(1).unwrap_or(0) , 0, num_siblings - 1);
                true
            }
        }
    }

    /// #Returns
    /// `true` if actually moved, `false` if stayed in the same place (due to no possible moves)
    pub fn move_down(&mut self) -> bool {
        let num_siblings = self.number_of_siblings_for_selected();
        match self.selected_node.last_mut() {
            None => { false }
            Some(i) => {
                *i = clamp(*i + 1, 0, num_siblings - 1);
                true
            }
        }
    }

    /// #Returns
    /// `true` if actually moved, `false` if stayed in the same place (due to no possible moves)
    pub fn move_back(&mut self) -> bool {
        match self.get_selected_parent() {
            None => { false } 
            Some(_) => {
                self.selected_node.pop();
                true
            }
        }
    }

    /// #Returns
    /// `true` if actually moved, `false` if stayed in the same place (due to no possible moves)
    pub fn move_forward(&mut self) -> MoveForwardResult {
        if self.number_of_children_for_selected() == 0 {
            MoveForwardResult::NoMoreChildren
        } else {
            if self.get_selected_tree().value.expanded {
                self.selected_node.push(0);
                MoveForwardResult::Moved
            } else {
                MoveForwardResult::NotExpanded
            }
        }
    }

    pub fn move_to_path(mut self, path: &TreePath) -> Self {
        let mut state_zipper = LazyTreeZipper::new(self.tree);
        for idx in path {
            state_zipper = state_zipper.child(*idx);
            let mut node = state_zipper.value().clone();
            node.expanded = true;
            state_zipper.replace(node);
        }
        self.tree = state_zipper.build();
        self
    }


    fn get_selected_tree(&self) -> &UITree {
        let mut child = &self.tree;
        for &i in self.selected_node.iter() {
            child = &child.children()[i];
        }
        child
    }
    fn get_selected_parent(&self) -> Option<&UITree> {
        let mut parent = &self.tree;
        if self.selected_node.len() == 0 {  return None; }
        for (i, &child_idx) in self.selected_node.iter().enumerate() {
            if i == self.selected_node.len().checked_sub(1).unwrap_or(0) {
                break;
            } else {
                parent = &parent.children()[child_idx];
            }
        }
        Some(parent)
    }

    pub fn set_selected(mut self, selected: bool) -> Self {
        let mut state_zipper = LazyTreeZipper::new(self.tree);
        state_zipper = state_zipper.follow_path(&self.selected_node);
        let mut node = state_zipper.value().clone();
        node.selected = selected;
        state_zipper.replace(node);
        self.tree = state_zipper.build();
        self
    }

    pub fn toggle_expanded(mut self) -> Self  {
        let mut state_zipper = LazyTreeZipper::new(self.tree);
        state_zipper = state_zipper.follow_path(&self.selected_node);
        let mut node = state_zipper.value().clone();
        node.expanded = !node.expanded;
        state_zipper.replace(node);
        self.tree = state_zipper.build();
        self
    }

    pub fn select_node(self) -> Self {
        self.set_selected(true)
    }

    pub fn deselect_node(self) -> Self {
        self.set_selected(false)
    }

    pub fn set_mode(&mut self, mode: UIMode) {
        self.mode = mode;
    }

    pub fn execute_command(self) -> Self {
        let command_buffer = self.command_buffer.trim().to_uppercase();
        let mut ui_state = self;
        match command_buffer.as_str() {
            "MEANING" => {
                ui_state.display_meaningfulness = !ui_state.display_meaningfulness;
                ui_state.set_mode(UIMode::TrieNavigation);
                ui_state
            }
            "LOAD" => {
                ui_state.set_mode(UIMode::FileExplorer(FileExplorerState {
                    files: LazyTreeZipper::new(create_file_tree(".".to_string())),
                    hovered_file: 0
                }));
                ui_state
            }
            _ => {
                ui_state.set_mode(UIMode::TrieNavigation);
                ui_state
            }
        }
    }

}

fn create_file_tree(folder_path: String) -> LazyTree<String> {

    let tree: LazyTree<String> = LazyTree::new(folder_path.clone(), Rc::new(move || 
                fs::read_dir(folder_path.clone()).unwrap()
                    .map(|path_result| 
                        {
                            create_file_tree(path_result.unwrap().path().into_os_string().into_string().unwrap())
                        }
                    )
                    .collect::<Vec<_>>()
        )
    );
    tree
    
}
pub fn clamp(i: usize, min: usize, max: usize) -> usize {
    if i < min { min } else if i > max { max } else { i }
}
pub fn clamp_i(i: isize, min: isize, max: isize) -> isize {
    if i < min { min } else if i > max { max } else { i }
}
