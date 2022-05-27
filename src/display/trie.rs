extern crate ncurses;
use std::collections::HashMap;
use crate::trie::TrieNodeRef;
use crate::tree::BiDirectionalTree;
use crate::trie::_TrieNode;
use crate::trie::TrieBuilder;
use crate::display::window::UIContext;
use crate::display::window::Window;
use crate::display::window::Position;
use crate::display::window::CursesBox;
use crate::tree::TreePath;
use std::cmp::max;
use crate::tree::Tree;
use crate::Rc;
use crate::LazyTreeZipper;
use ncurses::*;

use crate::trie::Trie;

use crate::tree::LazyTree;

#[derive(Clone, Debug)]
struct UIStateNode {
    selected: bool,
    expanded: bool,
    text: String,
    count:  usize,
    max_depth: usize,
    parents: Vec<String>
}

type UITree = LazyTree<UIStateNode>;

impl UITree {
}


struct UIState {
    tree: UITree,
    selected_node: TreePath
}    

fn clamp(i: usize, min: usize, max: usize) -> usize {
    if i < min { min } else if i > max { max } else { i }
}


enum MoveForwardResult {
    NotExpanded,
    Moved,
    NoMoreChildren
}


impl UIState {
    pub fn new(trie: TrieBuilder) -> UIState {
        let root = trie.root;
        UIState {
            tree: UIState::new_tree(Rc::new(trie), root, true),
            selected_node: vec![]
        }
    }

    fn new_tree(trie: Rc<TrieBuilder>, node_ref: TrieNodeRef, selected: bool) -> UITree {

        let node = trie.get_node(node_ref);
        let mut children = node.children.iter().map(|&x| x).collect::<Vec<_>>();
        children.sort_unstable_by(|&a, &b| trie.get_value(b).count.cmp(&trie.get_value(a).count));
        let mut text = trie.get_value(node_ref).data.map(|c| c.to_string()).unwrap_or("<root>".to_string());
        let mut last = children.iter().last().map(|&l| trie.get_node(l));
        
        while children.len() == 1 && last.unwrap().value.count == trie.get_value(node_ref).count {
            let child = &last.unwrap();
            text += &child.value.data.unwrap().to_string();
            children = child.children.iter().map(|&x| x).collect::<Vec<_>>();
            last = children.iter().last().map(|&l| trie.get_node(l));
        }

        LazyTree::new(
            UIStateNode {
                selected,
                expanded: false,
                text,
                count: trie.get_value(node_ref).count,
                max_depth: trie.get_value(node_ref).max_depth,
                // parents: node.parents.iter().map(|x| x.to_string()).collect()
                // parents: trie.get_parents(node_ref)
                parents: vec![]
            },
            Rc::new(
                              move || children
                                         .iter()
                                         .map(|child| (UIState::new_tree(trie.clone(), *child, false)))
                                         .collect::<Vec<_>>()
            ))
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


}


struct TextBox {
    text_attributes: Vec<attr_t>,
    text: String,
    bounding_box: CursesBox
}

pub fn display_trie(mut context: UIContext, trie: BiDirectionalTree<_TrieNode>) -> Option<i32> {
    let mut window = Window::new(&mut context);

    let mut ui_state = UIState::new(trie);

    loop {
        ui_state = ui_state.select_node();
        display_ui(&mut window, &ui_state);

        window.get_context().refresh();

        let ch = getch();
        let character = char::from_u32(ch as u32).unwrap_or('\0');

        ui_state = ui_state.deselect_node();

        match character {
            '\n' => {
                ui_state = ui_state.toggle_expanded();
            },
            'l' => {
                match ui_state.move_forward() {
                    MoveForwardResult::Moved => {}
                    MoveForwardResult::NotExpanded => {
                        ui_state = ui_state.toggle_expanded();
                        ui_state.move_forward();
                    }
                    MoveForwardResult::NoMoreChildren => {
                        beep();
                    }
                    
                }
            }
            'h' => {
                ui_state.move_back();
            }
            'j' => {
                ui_state.move_down();
            }
            'k' => {
                ui_state.move_up();
            }
            _ => { }
        }
        clear();
    }
    Some(endwin())
}


fn display_ui(window: &mut Window, state: &UIState) {
    let bounds = render(&state.tree, 0, 0);
    let selected_box = bounds.get_node(&state.selected_node);
    window.scroll_into_view(&Position {
        x: selected_box.bounding_box.x,
        y: selected_box.bounding_box.y,
    });
    display_box_tree(window, &bounds);
}


fn render(state: &UITree, y: i32, x: i32) -> Tree<TextBox> {
    let left_add = 3;
    let bottom_add = 1;
    let mut text = state.value.text.clone();
    text += if state.value.expanded { " " } else { "... " };
    text += &state.value.count.to_string();
    text += " | ";
    text += &state.value.max_depth.to_string();
    text += " | ";
    text += &state.value.parents
        .iter()
        // .filter(|s| s.starts_with('0'))
        .map(|s| "'".to_string() + s + "'")
        .collect::<Vec<_>>().join(" : ");

    let mut duplicates: HashMap<String, usize> = HashMap::new();
    let mut has_duplicate = false;
    for parent in state.value.parents.iter() {
        let value = duplicates
            .entry(parent.clone())
            .and_modify(|v| *v += 1)
            .or_insert(1);
        if *value > 1 {
            has_duplicate = true;
            break;
        }
    }
    // text += &state.value.parents.len().to_string();
    // text += &(has_duplicate.to_string());

    // text += &((state.value.count as f64).log2() * (state.value.max_depth as f64)).to_string();

    if state.value.expanded {
        let mut children = vec![];

        for child in state.children() {
            if children.len() == 0  {
                children.push(render(child, y + 1, x + left_add ));
            } else {
                let prev_state = children.last().unwrap();
                let new_y = prev_state.value.bounding_box.y + bottom_add + prev_state.value.bounding_box.height - 1;
                children.push(render(child, 
                                         new_y, 
                                         x + left_add
                ));
            }
        }
        let total_height = children.iter().fold(0, |a, b| a + b.value.bounding_box.height - 1 ) 
            + bottom_add * children.len() as i32
            + 1;
        let total_width = children.iter().fold(0, |a, b| max(a, b.value.bounding_box.width + left_add + 1));
        Tree {
            value: TextBox {
                text_attributes: if state.value.selected { vec![A_STANDOUT()] } else { vec![] },
                text,
                bounding_box: CursesBox {
                    y, x,
                    height: total_height,
                    width: total_width
                }
            },
            children: children
        }
    } else {
        Tree {
            children: vec![],
            value: TextBox {
                text_attributes: if state.value.selected { vec![A_STANDOUT()] } else { vec![] },
                text,
                bounding_box: CursesBox {
                    x, y,
                    width: left_add,
                    height: 2 
                }
            }
        }
    }
}


fn display_box_tree(window: &Window, tree: &Tree<TextBox>) {
    // display_curses_box(window, &tree.value.box_);
    let y = tree.value.bounding_box.y;
    let x = tree.value.bounding_box.x;
    let last_y = tree.children.last().map(|child| child.value.bounding_box.y).unwrap_or(y) ;
    let last_x = tree.children.last().map(|child| child.value.bounding_box.x).unwrap_or(x) ;
    window.vertical_line(y, last_y, x);
    window.addch(&Position { y: last_y, x }, ACS_LLCORNER());
    window.horizontal_line(last_y, x + 1, last_x);

    for (i, child) in tree.children.iter().enumerate() {
        if i == tree.children.len() - 1 { break };
        window.addch(&Position {
            y: child.value.bounding_box.y,
            x
        }, ACS_LTEE());
        window.horizontal_line(child.value.bounding_box.y, x + 1, last_x);
    }

    for &attribute in tree.value.text_attributes.iter() { attron(attribute); }
    window.addstr(&Position { y, x }, &tree.value.text);
    for &attribute in tree.value.text_attributes.iter() { attroff(attribute); }
    for child in tree.children.iter() {
        display_box_tree(window, &child);
    }
}

