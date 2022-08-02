extern crate ncurses;
use crate::window::DrawerPosition;
use crate::window::Position;
use crate::Window;
use crate::window::display_box_tree;
use crate::window::CursesBox;
use std::cmp::max;
use crate::window::TextBox;
use crate::tree::Tree;
use crate::display::trie::ui_state::*;
use ncurses::*;

pub fn handle_trie_navigation_mode(ui_state: UIState) -> UIState {
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
            '\t' => {
                ui_state = ui_state.move_to_longest_path();
            }
            ':' | ';' => {
                ui_state.set_mode(UIMode::Command);
            }
            _ => { }
        }
        ui_state
}


fn create_dom(state: &UIState, ui_tree: &UITree, y: i32, x: i32) -> Tree<TextBox> {
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

    if ui_tree.value.expanded {
        let mut children = vec![];

        for child in ui_tree.children() {
            if children.len() == 0  {
                children.push(create_dom(state, child, y + 1, x + left_add ));
            } else {
                let prev_state = children.last().unwrap();
                let new_y = prev_state.value.bounding_box.y + bottom_add + prev_state.value.bounding_box.height - 1;
                children.push(create_dom(state, child, 
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

pub fn display_trie(window: &mut Window, state: &UIState) {
    let dom = create_dom(state, &state.tree, 0, 0);
    let selected_box = dom.get_node(&state.selected_node);
    window.scroll_into_view(&Position {
        x: selected_box.bounding_box.x,
        y: selected_box.bounding_box.y,
    });

    // let window: Window = window.create_drawer(DrawerPosition::TOP, 2);
    display_box_tree(window, &dom);
}

