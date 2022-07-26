use crate::tree::Tree;
use std::ops::Sub;
use std::ops::Add;
use std::sync::atomic::{AtomicBool, Ordering};
use ncurses::*;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy)]
pub struct Position {
    pub x: i32, 
    pub y: i32
}


impl Add<Position> for Position {
    type Output = Position;
    fn add(self, other: Position) -> Self {
        Position {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl Sub<Position> for Position {
    type Output = Position;
    fn sub(self, other: Position) -> Self {
        Position {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }
}

pub struct CursesBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32
}
pub struct UIContext {
    ncurses_window: ncurses::ll::WINDOW,
    windows: Vec<_Window>
}

impl UIContext {
    pub fn refresh(&self) {
        refresh();
    }
}



pub struct Dimensions {
    pub width: i32, 
    pub height: i32
}

struct _Window {
    bounding_box: CursesBox,
    scroll: Position
}

pub struct Window<'a> {
    id: usize,
    context: &'a mut UIContext
}

/// x_diff is a value such that, when added to x, gives a value which is in
/// bounds
pub enum BoundsResult {
    YXOutOfBounds { x_diff: i32, y_diff: i32 },
    XOutOfBounds { x_diff: i32 },
    YOutOfBounds { y_diff: i32 },
    InBound
}

pub enum DrawerPosition {
    BOTTOM
}

impl Window<'_> {

    pub fn context() -> Option<UIContext> {
        let result = INITIALIZED.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed);
        match result {
            Ok(_) => {
                let result = Some(UIContext {
                    ncurses_window: initscr(),
                    windows: vec![],
                });
                noecho();
                result

            }
            Err(_) => { None }
        }
    }

    pub fn get_context(&mut self) -> &mut UIContext {
        self.context
    }

    pub fn new(context: &mut UIContext) -> Window {
        let dimensions = Window::get_n_curse_dimensions(context.ncurses_window);
        let window = _Window {
            bounding_box: CursesBox {
                x: 0,
                y: 0,
                width: dimensions.width ,
                height: dimensions.height
            },
            scroll: Position {
                x: 0, 
                y: 0
            }
        };

        context.windows.push(window);
        Window {
            id: context.windows.len() - 1,
            context
        }
    }

    pub fn create_drawer(&mut self, drawerPosition: DrawerPosition, size: i32) -> Window {
        match drawerPosition {
            BOTTOM => {
                let window = _Window {
                    bounding_box: CursesBox {
                        x: 0,
                        y: self.get_bounding_box().height - size,
                        width: self.get_bounding_box().width,
                        height: size
                    },
                    scroll: Position {
                        x: 0, 
                        y: 0
                    }
                };
                self.get_context().windows.push(window);
                Window {
                    id: self.get_context().windows.len() - 1,
                    context: self.get_context()
                }
            }
        }

    }



    fn absolute_to_window_relative(&self, absolute_pos: &Position) -> Position {
        *absolute_pos - self.get_absolute_position()
    }

    pub fn is_visible(&self, absolute_pos: &Position) -> BoundsResult {

        let window_position = self.absolute_to_window_relative(absolute_pos);

        let CursesBox { 
            height: window_height, 
            width: window_width, 
            ..
        } = self.get_bounding_box();

        let x_gt_bound = 0 <= window_position.x;
        let x_lt_bound = 0 < *window_width - window_position.x;
        let y_gt_bound = 0 <= window_position.y;
        let y_lt_bound = 0 < *window_height - window_position.y;

        let x_in_bounds = x_gt_bound && x_lt_bound;
        let y_in_bounds = y_gt_bound && y_lt_bound;
        if x_in_bounds && y_in_bounds {
            BoundsResult::InBound
        } else if !x_in_bounds && !y_in_bounds {
            BoundsResult::YXOutOfBounds {
                x_diff: if !x_gt_bound { -window_position.x } else { *window_width - 1 - window_position.x },
                y_diff: if !y_gt_bound { -window_position.y } else { *window_height - 1 - window_position.y }
            }
        } else if !x_in_bounds {
            BoundsResult::XOutOfBounds {
                x_diff: if !x_gt_bound { -window_position.x } else { *window_width - 1 - window_position.x },
            }
        } else { 
            BoundsResult::YOutOfBounds {
                y_diff: if !y_gt_bound { -window_position.y } else { *window_height - 1 - window_position.y }
            }
        } 
    }


    pub fn scroll_into_view(&mut self, scroll_relative_position: &Position) -> BoundsResult  {
        let bounds_result = self.is_visible(&self.scroll_relative_to_absolute(scroll_relative_position));
        match bounds_result {
            BoundsResult::InBound => {  },
            BoundsResult::XOutOfBounds { x_diff } => {
                self.scroll_x(-x_diff);
            },
            BoundsResult::YOutOfBounds { y_diff } => {
                self.scroll_y(-y_diff);
            },
            BoundsResult::YXOutOfBounds { y_diff, x_diff } => {
                self.scroll_y(-y_diff);
                self.scroll_x(-x_diff);
            }
        }
        bounds_result
    }

    pub fn scroll_x(&mut self, scroll_x: i32) {
        let Window {id, context} = self;
        context.windows[*id].scroll.x += scroll_x;
    }
    pub fn scroll_y(&mut self, scroll_y: i32) {
        let Window {id, context} = self;
        context.windows[*id].scroll.y += scroll_y;
    }

    fn scroll_relative_to_absolute(&self, &scroll_relative_pos: &Position) -> Position {
        let scroll_origin = self.get_scroll_origin();
        scroll_origin + scroll_relative_pos
    }

    fn absolute_to_scroll_relative(&self, absolute_pos: &Position) -> Position {
        let scroll_origin = self.get_scroll_origin();
        *absolute_pos - scroll_origin
    }

    fn get_scroll_origin(&self) -> Position {
        let CursesBox {
            x: window_x, y: window_y, ..
        } = self.get_bounding_box();

        let Position { x: scroll_x, y: scroll_y } = self.get_scroll();

        Position {
            x: window_x - scroll_x,
            y: window_y - scroll_y
        }
    }

    pub fn addch(&self, scroll_relative_pos: &Position, ch: chtype) {
        let absolute_position = self.scroll_relative_to_absolute(scroll_relative_pos);
        match self.is_visible(&absolute_position)  {
            BoundsResult::InBound => {
                wmove(self.context.ncurses_window, absolute_position.y, absolute_position.x);
                waddch(self.context.ncurses_window, ch);
            }
            _ => {}
        }
    }

    pub fn addstr(&self, scroll_relative_pos: &Position, s: &str) {
        let mut position = *scroll_relative_pos;
        for ch in s.chars() {
            self.addch(&position, ch as u32);
            position.x += 1;
        }
    }

    pub fn get_scroll(&self) -> &Position {
        let Window {id, context} = self;
        &context.windows[*id].scroll
    }
    pub fn get_bounding_box(&self) -> &CursesBox {
        let Window {id, context} = self;
        &context.windows[*id].bounding_box
    }
    pub fn get_absolute_position(&self) -> Position {
        let &CursesBox { x, y, .. } = self.get_bounding_box();
        Position {
            x, y
        }
    }

    pub fn vertical_line(&self, y1:i32, y2: i32, x: i32) {
        for y in y1..y2 {
            self.addch(&Position { y, x }, ACS_VLINE());
        }
    }

    pub fn horizontal_line(&self, y:i32, x1: i32, x2: i32) {
        for x in x1..x2 {
            self.addch(&Position { y, x }, ACS_HLINE());
        }
    }
    fn get_n_curse_dimensions(window: ncurses::ll::WINDOW) -> Dimensions  {
        let mut dimensions = Dimensions {
            width: 0,
            height: 0
        };
        getmaxyx(window, &mut dimensions.height, &mut dimensions.width);
        dimensions
    }

}

pub struct TextBox {
    pub text_attributes: Vec<attr_t>,
    pub text: String,
    pub bounding_box: CursesBox
}

pub fn display_box_tree(window: &Window, tree: &Tree<TextBox>) {
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

