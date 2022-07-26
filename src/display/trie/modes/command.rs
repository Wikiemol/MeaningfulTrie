extern crate ncurses;
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

pub fn display_command_line(window: &mut Window, state: &UIState) {
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
