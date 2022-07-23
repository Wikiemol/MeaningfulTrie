extern crate ncurses;
use std::fs;
use std::fs::metadata;
use std::collections::HashMap;
use crate::trie::TrieNodeRef;
use crate::tree::BiDirectionalTree;
use crate::trie::_TrieNode;
use crate::trie::Trie;
use crate::display::window::UIContext;
use crate::display::window::Window;
use crate::display::window::DrawerPosition;
use crate::display::window::Position;
use crate::display::window::CursesBox;
use crate::tree::TreePath;
use std::cmp::max;
use crate::tree::Tree;
use crate::Rc;
use crate::LazyTreeZipper;
use ncurses::*;

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

enum UIMode {
    FileExplorer(FileExplorerState),
    TrieNavigation, 
    Command
}

struct FileExplorerState {
    files: LazyTreeZipper<String>,
    hovered_file: usize
}



struct UIState {
    tree: UITree,
    selected_node: TreePath,
    mode: UIMode,
    command_buffer: String,
    display_meaningfulness: bool
}    

pub fn start_display(mut context: UIContext, trie: BiDirectionalTree<_TrieNode>) -> Option<i32> {
    let mut window = Window::new(&mut context);

    let mut ui_state = UIState::new(trie);

    loop {
        ui_state = ui_state.select_node();
        display_ui(&mut window, &ui_state);

        window.get_context().refresh();

        match ui_state.mode {
            UIMode::TrieNavigation => {
                ui_state = handle_trie_navigation_mode(ui_state);
            },
            UIMode::Command => {
                ui_state = handle_command_mode(ui_state);
            }
            UIMode::FileExplorer(_)  => {
                ui_state = handle_file_explorer_mode(ui_state);
            }

        }
        clear();
    }
}

fn clamp(i: usize, min: usize, max: usize) -> usize {
    if i < min { min } else if i > max { max } else { i }
}
fn clamp_i(i: isize, min: isize, max: isize) -> isize {
    if i < min { min } else if i > max { max } else { i }
}



enum MoveForwardResult {
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
                parents: node
                    .parents
                    .iter()
                    .map(|parent| trie
                         .get_node(*parent)
                         .value
                         .data
                         .map(|data| data.to_string())
                         .unwrap_or("<root>".to_string())
                    )
                    .collect()
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


struct TextBox {
    text_attributes: Vec<attr_t>,
    text: String,
    bounding_box: CursesBox
}



fn handle_trie_navigation_mode(ui_state: UIState) -> UIState {
        let ch = getch();
        let character = char::from_u32(ch as u32).unwrap_or('\0');
        let mut ui_state = ui_state;

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
            ':' | ';' => {
                ui_state.set_mode(UIMode::Command);
            }
            _ => { }
        }
        ui_state
}

fn handle_command_mode(ui_state: UIState) -> UIState {
        let ch = getch();
        let character = char::from_u32(ch as u32).unwrap_or('\0');
        let mut ui_state = ui_state;

        ui_state = ui_state.deselect_node();
        match character {
            '\n' => {
                ui_state = ui_state.execute_command();
            },
            // Delete
            '\x7F' => {
                if ui_state.command_buffer.len() > 0 {
                    ui_state.command_buffer.remove(ui_state.command_buffer.len() - 1);
                }
            },
            // Ctrl-U/Ctrl-X
            '\x15' | '\x18' => {
                    ui_state.command_buffer = "".to_string();
            }
            chr => { 
                ui_state.command_buffer = ui_state.command_buffer + &chr.to_string();
            }
        }
        ui_state
}

fn handle_file_explorer_mode(ui_state: UIState) -> UIState {
    let mut ui_state = ui_state;
    let ch = getch();
    let character = char::from_u32(ch as u32).unwrap_or('\0');

    match character {
        'j' => {
        let mut file_explorer_state = match ui_state.mode {
            UIMode::FileExplorer(file_explorer_state) => file_explorer_state,
            _ => panic!("Needs to be in file explorer mode")
        };
            file_explorer_state.hovered_file = usize::try_from(
                clamp_i(
                    (file_explorer_state.hovered_file as isize) + 1, 
                    0, 
                    (file_explorer_state.files.number_of_children() as isize) - 1
                )
            ).unwrap_or(0);
            ui_state.mode = UIMode::FileExplorer(file_explorer_state);
        },
        'k' => {
            let mut file_explorer_state = match ui_state.mode {
                UIMode::FileExplorer(file_explorer_state) => file_explorer_state,
                _ => panic!("Needs to be in file explorer mode")
            };
            file_explorer_state.hovered_file = usize::try_from(
                clamp_i(
                    (file_explorer_state.hovered_file as isize) - 1, 
                    0, 
                    (file_explorer_state.files.number_of_children() as isize) - 1
                )
            ).unwrap_or(0);
            ui_state.mode = UIMode::FileExplorer(file_explorer_state);
        },
        '\n' => {
            let mut file_explorer_state = match ui_state.mode {
                UIMode::FileExplorer(file_explorer_state) => file_explorer_state,
                _ => panic!("Needs to be in file explorer mode")
            };
            let selected_file =  file_explorer_state
                .files
                .children()
                .get(file_explorer_state.hovered_file)
                .map(|&x| x.clone())
                .unwrap();
            if metadata(&selected_file).unwrap().is_dir() {
                file_explorer_state.files = file_explorer_state.files.child(file_explorer_state.hovered_file);
                ui_state.mode = UIMode::FileExplorer(file_explorer_state);
            } else {
                let file_text = fs::read_to_string(selected_file)
                    .expect("Unable to read file")
                    .replace("\n", "\\n")
                    // .replace(" ", "xn");
                    .to_lowercase();
                ui_state = UIState::new(Trie::suffix(&file_text, Some(100)));
            }

        }
        _ => { }
    }

    ui_state
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

fn display_ui(window: &mut Window, state: &UIState) {
    match &state.mode {
        UIMode::FileExplorer(file_explorer_state) => {
            display_file_explorer(window, &file_explorer_state);
        }
        _ => {
            display_trie(window, state);
            display_command_line(window, state);
        }
    }
}

fn display_file_explorer(window: &mut Window, state: &FileExplorerState) {
    let files = state.files.children();
    let mut position = Position {
        x: 0,
        y: 0
    };

    let mut ui_children = vec![];
    for file in files {
        let text_box = Tree {
            value: TextBox {
                bounding_box: CursesBox {
                    height: 1,
                    width: window.get_bounding_box().width,
                    x: 0,
                    y: ui_children.len() as i32,
                },
                text: file.to_string(),
                text_attributes: if state.hovered_file == ui_children.len() { 
                    vec![A_STANDOUT()] 
                } else { 
                    vec![] 
                }
            },
            children: vec![]
        };
        ui_children.push(text_box);
    }

    display_box_tree(window, &Tree {
        value: TextBox {
            bounding_box: CursesBox { ..*window.get_bounding_box() },
            text: "".to_string(),
            text_attributes: vec![]
        },
        children: ui_children
    });
    
}

fn display_command_line(window: &mut Window, state: &UIState) {
    match state.mode {
        UIMode::Command => {
            let command_line = window.create_drawer(DrawerPosition::BOTTOM, 1);
            command_line.addch(&Position {
                x: 1,
                y: 0
            }, ':' as u32);
            command_line.addstr(&Position {
                x: 3,
                y: 0
            }, &state.command_buffer);
        }
        _ => {}
    }

}


fn display_trie(window: &mut Window, state: &UIState) {
    let ui_trie = render(state, &state.tree, 0, 0);
    let selected_box = ui_trie.get_node(&state.selected_node);
    window.scroll_into_view(&Position {
        x: selected_box.bounding_box.x,
        y: selected_box.bounding_box.y,
    });
    display_box_tree(window, &ui_trie);

}


fn render(state: &UIState, ui_tree: &UITree, y: i32, x: i32) -> Tree<TextBox> {
    let left_add = 3;
    let bottom_add = 1;
    let mut text = ui_tree.value.text.clone();
    text += if ui_tree.value.expanded { " " } else { "... " };
    text += &ui_tree.value.count.to_string();
    text += " | ";
    text += &ui_tree.value.max_depth.to_string();
    text += " | ";
    text += &(
        if state.display_meaningfulness { 
            (((ui_tree.value.count as f64).log2()) * (ui_tree.value.max_depth as f64)).to_string() 
        } else { 
            "".to_string() 
        }
    );
    text += &ui_tree.value.parents
        .iter()
        // .filter(|s| s.starts_with('0'))
        .map(|s| "'".to_string() + s + "'")
        .collect::<Vec<_>>().join(" : ");

    let mut duplicates: HashMap<String, usize> = HashMap::new();
    let mut has_duplicate = false;
    for parent in ui_tree.value.parents.iter() {
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

    if ui_tree.value.expanded {
        let mut children = vec![];

        for child in ui_tree.children() {
            if children.len() == 0  {
                children.push(render(state, child, y + 1, x + left_add ));
            } else {
                let prev_state = children.last().unwrap();
                let new_y = prev_state.value.bounding_box.y + bottom_add + prev_state.value.bounding_box.height - 1;
                children.push(render(state, child, 
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
                text_attributes: if ui_tree.value.selected { vec![A_STANDOUT()] } else { vec![] },
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
                text_attributes: if ui_tree.value.selected { vec![A_STANDOUT()] } else { vec![] },
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

