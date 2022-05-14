extern crate ncurses;
use crate::display::trie::window::UIContext;
use crate::display::trie::window::Window;
use crate::display::trie::window::Position;
use crate::display::trie::window::CursesBox;
use crate::tree::TreePath;
use std::fs::File;
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
    data: String,
    count:  usize,
    max_depth: usize
}

type UITree = LazyTree<UIStateNode>;


struct UIState {
    tree: UITree,
    selected_node: TreePath
}

pub mod window {
    use std::ops::Sub;
    use std::ops::Add;
    use std::sync::atomic::{AtomicBool, Ordering};
    use ncurses::*;

    static initialized: AtomicBool = AtomicBool::new(false);

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

    /// *_diff is a value such that, when added to * (that is, x or y), gives a value which is in
    /// bounds
    pub enum BoundsResult {
        YXOutOfBounds { x_diff: i32, y_diff: i32 },
        XOutOfBounds { x_diff: i32 },
        YOutOfBounds { y_diff: i32 },
        InBound
    }

    impl Window<'_> {

        pub fn context() -> Option<UIContext> {
            let result = initialized.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed);
            match result {
                Ok(_) => {
                    Some(UIContext {
                        ncurses_window: initscr(),
                        windows: vec![],
                    })
                }
                Err(_) => { None }
            }
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
    data: String,
    bounding_box: CursesBox
}



fn create_ui_state(trie: &Trie, selected: bool) -> UITree {
    let mut children = trie.children();
    children.sort_by(|a, b| b.count().cmp(&a.count()));
    let mut text = trie.data().map(|c| c.to_string()).unwrap_or("<root>".to_string());
    while children.len() == 1 && children[0].count() == trie.count() {
        let child = &children[0];
        text += &child.data().unwrap().to_string();
        children = child.children();
    }

    
    LazyTree::new(
        UIStateNode {
            selected: selected,
            expanded: false,
            data: text,
            count: trie.count(),
            max_depth: trie.max_depth()
        },
        Rc::new(
                          move || children
                                     .iter()
                                     .map(|child| (create_ui_state(child, false)))
                                     .collect::<Vec<_>>()
        ))
}

pub fn display_trie(context: &mut UIContext, trie: Trie) -> Option<i32> {
    let mut window = Window::new(context);
    noecho();
    refresh();

    let mut ui_state = UIState {
        tree: create_ui_state(&trie, true),
        selected_node: vec![]
    };

    loop {
        ui_state = ui_state.select_node();
        display_ui(&mut window, &ui_state);
        refresh();
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
    let bounds = get_bounds(&state.tree, 0, 0);
    let selected_box = bounds.get_node(&state.selected_node);
    window.scroll_into_view(&Position {
        x: selected_box.bounding_box.x,
        y: selected_box.bounding_box.y,
    });
    display_box_tree(window, &bounds);
}


fn get_bounds(state: &UITree, y: i32, x: i32) -> Tree<TextBox> {
    let left_add = 3;
    let bottom_add = 1;
    if state.value.expanded {
        let mut children = vec![];

        for child in state.children() {
            if children.len() == 0  {
                children.push(get_bounds(child, y + 1, x + left_add ));
            } else {
                let prev_state = children.last().unwrap();
                let new_y = prev_state.value.bounding_box.y + bottom_add + prev_state.value.bounding_box.height - 1;
                children.push(get_bounds(child, 
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
                data: state.value.data.clone(),
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
                data: state.value.data.clone(),
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
    window.addstr(&Position { y, x }, &tree.value.data);
    for &attribute in tree.value.text_attributes.iter() { attroff(attribute); }
    for child in tree.children.iter() {
        display_box_tree(window, &child);
    }
}

