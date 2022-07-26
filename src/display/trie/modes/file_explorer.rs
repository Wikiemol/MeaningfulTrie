extern crate ncurses;
use crate::window::display_box_tree;
use crate::window::CursesBox;
use crate::window::TextBox;
use crate::tree::Tree;
use crate::window::Position;
use crate::Window;
use crate::Rc;
use crate::LazyTree;
use crate::Trie;
use std::fs::metadata;
use crate::display::trie::ui_state::*;
use ncurses::*;

pub fn handle_file_explorer_mode(ui_state: UIState) -> UIState {
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
                let file_text = std::fs::read_to_string(selected_file)
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
                std::fs::read_dir(folder_path.clone()).unwrap()
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


pub fn display_file_explorer(window: &mut Window, state: &FileExplorerState) {
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
