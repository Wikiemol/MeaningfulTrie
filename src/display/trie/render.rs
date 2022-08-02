
extern crate ncurses;
use ncurses::*;
use crate::trie::TrieNode;
use crate::tree::BiDirectionalTree;
use crate::Window;
use crate::UIContext;

use crate::display::trie::ui_state::*;
use crate::display::trie::modes::*;

pub fn start_display(mut context: UIContext, trie: BiDirectionalTree<TrieNode>) -> Option<i32> {
    let mut window = Window::new(&mut context);

    let mut ui_state = UIState::new(trie);

    loop {
        ui_state = ui_state.select_node();
        display_ui(&mut window, &ui_state);

        window.get_context().refresh();

        match ui_state.mode {
            UIMode::TrieNavigation => {
                ui_state = trie_navigation::handle_trie_navigation_mode(ui_state);
            },
            UIMode::Command => {
                ui_state = command::handle_command_mode(ui_state);
            }
            UIMode::FileExplorer(_)  => {
                ui_state = file_explorer::handle_file_explorer_mode(ui_state);
            }
            UIMode::Exit => {
                break;
            }

        }
        clear();
    }
    Some(0)
}

fn display_ui(window: &mut Window, state: &UIState) {
    match &state.mode {
        UIMode::FileExplorer(file_explorer_state) => {
            file_explorer::display_file_explorer(window, &file_explorer_state);
        }
        _ => {
            trie_navigation::display_trie(window, state);
            command::display_command_line(window, state);
        }
    }
}
