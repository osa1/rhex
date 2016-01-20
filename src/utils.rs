////////////////////////////////////////////////////////////////////////////////
// Utilities
////////////////////////////////////////////////////////////////////////////////

// FIXME: These must be in stdlib.

// TODO: Is there a way to make these polymorphic over reference types? i.e.
// same function should work with both mutable and immutable references, as long
// as first argument and function argument have same mutability.

pub fn opt<A, F>(o : &Option<A>, fun : F) where F : Fn(&A) {
    match o {
        &None => (),
        &Some(ref o) => fun(o),
    }
}

pub fn opt_mut<A, F>(o : &mut Option<A>, fun : F) where F : Fn(&mut A) {
    match o {
        &mut None => (),
        &mut Some(ref mut o) => fun(o),
    }
}

#[inline]
pub fn hex_char(nibble : u8) -> u8 {
    if nibble < 10 {
        48 + nibble
    } else {
        97 + nibble - 10
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn draw_box(pos_x : i32, pos_y : i32, width : i32, height : i32,
                title : Option<&str>) {

    use ncurses as nc;

    // draw corners
    nc::mvaddch( pos_y,          pos_x,         nc::ACS_ULCORNER() );
    nc::mvaddch( pos_y,          pos_x + width, nc::ACS_URCORNER() );
    nc::mvaddch( pos_y + height, pos_x,         nc::ACS_LLCORNER() );
    nc::mvaddch( pos_y + height, pos_x + width, nc::ACS_LRCORNER() );

    // draw edges
    nc::mvhline( pos_y,          pos_x + 1,     0, width - 1 );
    nc::mvhline( pos_y + height, pos_x + 1,     0, width - 1 );
    nc::mvvline( pos_y + 1,      pos_x,         0, height - 1 );
    nc::mvvline( pos_y + 1,      pos_x + width, 0, height - 1 );

    // Print title
    match title {
        None => {},
        Some(title) => {
            nc::mvaddstr( pos_y, pos_x + 2, title );
        },
    }
}
