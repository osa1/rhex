mod ascii_view;
mod goto;
mod hex_grid;
mod info_line;
mod lines;
mod search;

use colors;
use self::ascii_view::AsciiView;
use self::goto::{GotoOverlay, OverlayRet};
use self::hex_grid::HexGrid;
use self::info_line::InfoLine;
use self::lines::Lines;
use self::search::{SearchOverlay, SearchRet};

use libc;
use nix::poll::{poll, PollFd, POLLIN};
use term_input::{Event, Input, Key};
use termbox_simple::*;

/// GUI is the main thing that owns every widget. It's also responsible for
/// ncurses initialization and finalization.
pub struct HexGui<'gui> {
    tb: Termbox,
    width: i32,
    height: i32,

    hex_grid: HexGrid<'gui>,
    lines: Lines,
    ascii_view: AsciiView<'gui>,
    info_line: InfoLine,
    overlay: Overlay<'gui>,
    contents: &'gui [u8],

    highlight: Vec<usize>,
    highlight_len: usize,

    z_pressed: bool,
}

pub enum Overlay<'overlay> {
    NoOverlay,
    SearchOverlay(SearchOverlay<'overlay>),
    GotoOverlay(GotoOverlay),
}

struct Layout {
    lines_width: i32,
    hex_grid_x: i32,
    hex_grid_width: i32,
    ascii_view_x: i32,
    ascii_view_width: i32,
}

fn layout(w: i32, content_size: usize) -> Layout {
    // Calculate cols needed for showing the addresses
    let hex_digits_needed = (content_size as f32).log(16.0f32) as i32;
    let lines_width_pre = hex_digits_needed + 2; // take 0x prefix into account
    let lines_width = if lines_width_pre as f32 > w as f32 / 40.0 * 100.0 {
        0
    } else {
        lines_width_pre
    };

    // -1 for the vertical line between hex and ascii views
    // Another -1 for a vertical line between lines and hex view if we draw lines
    let grid_width = w - lines_width - 1 - if lines_width == 0 { 1 } else { 0 };

    // Every byte takes 3 characters in hex view and 1 character in ascii view.
    // So we have this 3/1 ratio.
    let unit_column = grid_width / 4;
    let hex_grid_width = unit_column * 3;
    Layout {
        lines_width,
        hex_grid_x: lines_width + 1,
        hex_grid_width,
        ascii_view_x: lines_width + if lines_width == 0 { 0 } else { 1 } + hex_grid_width,
        ascii_view_width: unit_column,
    }
}

// WARNING: Moving this after init() will cause a segfault. Not calling init()
// will cause a segfault.

impl<'gui> HexGui<'gui> {
    pub fn new(
        tb: Termbox,
        contents: &'gui [u8],
        path: &'gui str,
        width: i32,
        height: i32,
    ) -> HexGui<'gui> {
        let layout = layout(width, contents.len());
        let hex_grid = HexGrid::new(
            layout.hex_grid_width,
            height - 1,
            layout.hex_grid_x,
            0,
            contents,
            path,
        );
        let lines = Lines::new(
            hex_grid.bytes_per_line(),
            contents.len() as i32,
            layout.lines_width,
            height,
        );
        let ascii_view = AsciiView::new(
            layout.ascii_view_width,
            height - 1,
            layout.ascii_view_x,
            0,
            contents,
        );
        let info_line = InfoLine::new(width, 0, height - 1, format!("{} - 0: 0", path));
        HexGui {
            tb: tb,
            width: width,
            height: height,

            hex_grid: hex_grid,
            lines: lines,
            ascii_view: ascii_view,
            info_line: info_line,
            overlay: Overlay::NoOverlay,
            contents: contents,

            highlight: Vec::new(),
            highlight_len: 0,

            z_pressed: false,
        }
    }

    pub fn init(&mut self) {
        let self_ptr = self as *mut HexGui;
        self.hex_grid.set_gui(self_ptr);
    }

    pub fn get_lines(&mut self) -> &mut Lines {
        &mut self.lines
    }

    pub fn get_ascii_view(&mut self) -> &mut AsciiView<'gui> {
        &mut self.ascii_view
    }

    pub fn get_info_line(&mut self) -> &mut InfoLine {
        &mut self.info_line
    }

    pub fn draw(&mut self) {
        self.tb.clear();

        self.lines.draw(&mut self.tb);

        let vsplit_x = self.lines.width();
        for y in 0..self.height - 1 {
            self.tb
                .change_cell(vsplit_x, y, '│', colors::DEFAULT.fg, colors::DEFAULT.bg);
        }

        self.hex_grid
            .draw(&mut self.tb, &self.highlight, self.highlight_len);

        let vsplit_x = vsplit_x + self.hex_grid.width();
        for y in 0..self.height - 1 {
            self.tb
                .change_cell(vsplit_x, y, '│', colors::DEFAULT.fg, colors::DEFAULT.bg);
        }

        self.ascii_view
            .draw(&mut self.tb, &self.highlight, self.highlight_len);

        self.info_line.draw(&mut self.tb);

        match self.overlay {
            Overlay::NoOverlay =>
                {}
            Overlay::SearchOverlay(ref o) =>
                o.draw(&mut self.tb),
            Overlay::GotoOverlay(ref o) =>
                o.draw(&mut self.tb),
        }

        self.tb.present();
    }

    pub fn mainloop(&mut self) {
        let mut input = Input::new();
        let mut evs = Vec::with_capacity(10);
        self.draw();

        loop {
            let mut fds = [PollFd::new(libc::STDIN_FILENO, POLLIN)];
            let _ = poll(&mut fds, -1);

            input.read_input_events(&mut evs);

            let mut brk = false;
            for ev in evs.drain(..) {
                brk |= self.handle_event(ev);
            }
            if brk {
                break;
            }
            self.draw();
        }
    }

    fn handle_event(&mut self, ev: Event) -> bool {
        match ev {
            Event::Key(key) =>
                self.keypressed(key),
            Event::String(_) |
            Event::Resize |
            Event::FocusGained |
            Event::FocusLost |
            Event::Unknown(_) =>
                false,
        }
    }

    fn keypressed(&mut self, key: Key) -> bool {
        let mut reset_overlay = false;
        match self.overlay {
            Overlay::NoOverlay => {
                if key == Key::Char('q') {
                    return true;
                }
                self.keypressed_no_overlay(key)
            }

            Overlay::GotoOverlay(ref mut o) =>
                match o.keypressed(key) {
                    OverlayRet::Ret(offset) => {
                        self.hex_grid.move_cursor_offset(offset);
                        reset_overlay = true;
                    }
                    OverlayRet::GotoBeginning => {
                        self.hex_grid.move_cursor_offset(0);
                        reset_overlay = true;
                    }
                    OverlayRet::Continue =>
                        {}
                    OverlayRet::Abort => {
                        reset_overlay = true;
                    }
                },

            Overlay::SearchOverlay(ref mut o) => {
                match o.keypressed(key) {
                    SearchRet::Highlight {
                        all_bytes: bs,
                        len: l,
                        ..
                    } => {
                        self.highlight = bs;
                        self.highlight_len = l;
                        reset_overlay = true;
                    }
                    SearchRet::Abort => {
                        reset_overlay = true;
                    }
                    SearchRet::Continue =>
                    { /* nothing to do */ }
                }
            }
        };

        if reset_overlay {
            self.overlay = Overlay::NoOverlay;
        }

        false
    }

    fn keypressed_no_overlay(&mut self, key: Key) {
        match key {
            Key::Char('g') => {
                self.z_pressed = false;
                self.mk_goto_overlay();
            }
            Key::Char('/') => {
                self.z_pressed = false;
                self.mk_search_overlay();
            }
            Key::Char('z') =>
                if self.z_pressed {
                    self.hex_grid.try_center_scroll();
                    self.lines.set_scroll(self.hex_grid.get_scroll());
                    self.ascii_view.set_scroll(self.hex_grid.get_scroll());
                    self.z_pressed = false;
                } else {
                    self.z_pressed = true;
                },
            Key::Char('n') => {
                self.z_pressed = false;
                let hls = &self.highlight;
                let byte_idx = self.hex_grid.get_byte_idx() as usize;
                for &hl_offset in hls {
                    if hl_offset > byte_idx {
                        self.hex_grid.move_cursor_offset(hl_offset as i32);
                        return;
                    }
                }
                // We couldn't jump to a match, start from the beginning
                if let Some(&hl_offset) = hls.get(0) {
                    self.hex_grid.move_cursor_offset(hl_offset as i32);
                }
            }
            Key::Char('N') => {
                self.z_pressed = false;
                let hls = &self.highlight;
                let byte_idx = self.hex_grid.get_byte_idx() as usize;
                for &hl_offset in hls.iter().rev() {
                    if hl_offset < byte_idx {
                        self.hex_grid.move_cursor_offset(hl_offset as i32);
                        return;
                    }
                }
                // We couldn't jump to a match, start from the beginning
                if let Some(&hl_offset) = hls.get(hls.len() - 1) {
                    self.hex_grid.move_cursor_offset(hl_offset as i32);
                }
            }
            _ => {
                self.z_pressed = false;
                self.hex_grid.keypressed(key);
            }
        }
    }

    fn mk_goto_overlay(&mut self) {
        self.overlay = Overlay::GotoOverlay(GotoOverlay::new(
            self.width / 2,
            self.height / 2,
            self.width / 4,
            self.height / 4,
        ));
    }

    fn mk_search_overlay(&mut self) {
        self.overlay = Overlay::SearchOverlay(SearchOverlay::new(
            self.width / 2,
            self.height / 2,
            self.width / 4,
            self.height / 4,
            self.contents,
        ));
    }
}
