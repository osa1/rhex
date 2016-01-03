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
