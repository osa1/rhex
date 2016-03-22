pub trait Widget {
    fn get_height(&self) -> i32 {
        1
    }

    fn focus(&mut self) -> bool {
        false
    }

    fn keypressed(&mut self, key : i32) -> WidgetRet {
        WidgetRet::KeyIgnored
    }

    fn draw(&self, pos_x : i32, pos_y : i32, width : i32, height : i32, higlight : bool);
}

pub enum WidgetRet {
    LostFocus, KeyHandled, KeyIgnored
}
