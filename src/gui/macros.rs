#[macro_export]
macro_rules! with_attr {
    ( $guard:expr, $attr_expr:expr, $body:expr ) => {
        if $guard {
            nc::attron($attr_expr);
        }

        $body;

        if $guard {
            nc::attroff($attr_expr);
        }
    };
}
