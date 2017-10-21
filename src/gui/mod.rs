pub mod hex;

use termbox_simple::*;

pub struct Gui<'gui> {
    hex_gui: hex::HexGui<'gui>,
}

impl<'gui> Gui<'gui> {
    pub fn new_hex_gui(
        tb: Termbox,
        contents: &'gui [u8],
        path: &'gui str,
        width: i32,
        height: i32,
    ) -> Gui<'gui> {
        Gui {
            hex_gui: hex::HexGui::new(tb, contents, path, width, height),
        }
    }

    pub fn mainloop(&mut self) {
        self.hex_gui.init();
        self.hex_gui.mainloop();
    }
}
