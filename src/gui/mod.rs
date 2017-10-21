#[macro_use]
pub mod macros;

pub mod hex;

pub struct Gui<'gui> {
    hex_gui: hex::HexGui<'gui>,
}

impl<'gui> Gui<'gui> {
    pub fn new_hex_gui(
        contents: &'gui [u8],
        path: &'gui str,
        width: i32,
        height: i32,
        pos_x: i32,
        pos_y: i32,
    ) -> Gui<'gui> {
        Gui {
            hex_gui: hex::HexGui::new(contents, path, width, height, pos_x, pos_y),
        }
    }

    pub fn mainloop(&mut self) {
        self.hex_gui.init();
        self.hex_gui.mainloop();
    }
}
