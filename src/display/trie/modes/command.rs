extern crate ncurses;
use crate::Rc;
use std::fs;
use crate::LazyTree;
use crate::LazyTreeZipper;
use crate::window::Position;
use crate::window::DrawerPosition;
use crate::Window;
use crate::display::trie::ui_state::*;
use ncurses::*;

pub fn handle_command_mode(ui_state: UIState) -> UIState {
        let ch = getch();
        let character = char::from_u32(ch as u32).unwrap_or('\0');
        let mut ui_state = ui_state;

        ui_state = ui_state.deselect_node();
        match character {
            '\n' => {
                ui_state = execute_command(ui_state);
            },
            '\x1B' => {
                ui_state.set_mode(UIMode::TrieNavigation);
            }
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

pub fn display_command_line(window: &mut Window, state: &UIState) {
    match state.mode {
        UIMode::Command => {
            let mut command_line = window.create_drawer(DrawerPosition::Bottom, 1);

            command_line.addch(&Position {
                x: 1,
                y: 0
            }, ':' as u32);
            command_line.addstr(&Position {
                x: 3,
                y: 0
            }, &state.command_buffer);

            command_line.delete();
        }
        _ => {}
    }

}

fn execute_command(ui_state: UIState) -> UIState {
    let command_buffer = ui_state.command_buffer.trim().to_uppercase();
    let mut ui_state = ui_state;
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
        "Q" => {
            ui_state.set_mode(UIMode::Exit);
            ui_state
        }
        _ => {
            ui_state.set_mode(UIMode::TrieNavigation);
            ui_state
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
