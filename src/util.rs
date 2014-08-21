use std::cell::Cell;

pub trait CellOps<T> {
    fn modify(&self, f: |T| -> T);
}

impl<T: Copy> CellOps<T> for Cell<T> {
    fn modify(&self, f: |T| -> T) {
        self.set(f(self.get()));
    }
}

pub trait CharOps {
    fn is_emphasis(self) -> bool;
    fn is_code(self) -> bool;
}

impl CharOps for u8 {
    fn is_emphasis(self) -> bool {
        one_of!(self, b'*', b'_')
    }

    fn is_code(self) -> bool {
        self == b'`'
    }

    fn is_space(self) -> bool {
        self == b' ' || self == b'\n'
    }
}

pub trait ByteMatcher {
    fn matches(&mut self, b: u8) -> bool;
}

impl<'a> ByteMatcher for |u8|:'a -> bool {
    #[inline]
    fn matches(&mut self, b: u8) -> bool { (*self)(b) }
}

impl ByteMatcher for u8 {
    #[inline]
    fn matches(&mut self, b: u8) -> bool { *self == b }
}

impl<'a> ByteMatcher for &'a [u8] {
    #[inline]
    fn matches(&mut self, b: &'a [u8]) -> bool {
        b.iter().any(|& mut c| c.matches(b))
    }
}

pub trait ByteSliceOps {
    fn trim_left<M: ByteMatcher>(&self, m: M) -> &[u8];
    fn trim_right<M: ByteMatcher>(&self, m: M) -> &[u8];
}

impl<'a> ByteSliceOps for &'a [u8] {
    fn trim_left<M: ByteMatcher>(&self, m: M) -> &[u8] {
        match self.iter().position(|b| !m.matches(b)) {
            None => &'static [],
            Some(idx) => self.slice_from(idx)
        }
    }

    fn trim_right<M: ByteMatcher>(&self, m: M) -> &[u8] {
        match self.iter().rposition(|b| !m.matches(b)) {
            None => &'static [],
            Some(idx) => self.slice_to(idx+1)
        }
    }
}
