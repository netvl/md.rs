use std::cell::Cell;

pub trait CellOps<T> {
    fn modify<F: FnOnce(T) -> T>(&self, f: F);
}

impl<T: Copy> CellOps<T> for Cell<T> {
    fn modify<F: FnOnce(T) -> T>(&self, f: F) {
        self.set(f(self.get()));
    }
}

pub trait CharOps {
    fn is_emphasis(self) -> bool;
    fn is_code(self) -> bool;
    fn is_space(self) -> bool;
    fn is_numeric(self) -> bool;
}

impl CharOps for u8 {
    fn is_emphasis(self) -> bool {
        self == b'*' || self == b'_'
    }

    fn is_code(self) -> bool {
        self == b'`'
    }

    fn is_space(self) -> bool {
        self == b' ' || self == b'\n'
    }

    fn is_numeric(self) -> bool {
        static DIGITS: &'static [u8] = b"0123456789";
        DIGITS.contains(&self)
    }
}

pub trait ByteMatcher {
    fn matches(&mut self, b: u8) -> bool;
}

impl<F> ByteMatcher for F where F: FnMut(u8) -> bool {
    #[inline]
    fn matches(&mut self, b: u8) -> bool { self(b) }
}

impl ByteMatcher for u8 {
    #[inline]
    fn matches(&mut self, b: u8) -> bool { *self == b }
}

impl<'a> ByteMatcher for &'a [u8] {
    #[inline]
    fn matches(&mut self, b: u8) -> bool {
        self.iter().map(deref).any(|mut c| c.matches(b))
    }
}

#[inline(always)]
fn deref<T: Copy>(x: &T) -> T { *x }

pub trait ByteSliceOps<'a> {
    fn trim_left<M: ByteMatcher>(&self, m: M) -> &'a [u8];
    fn trim_right<M: ByteMatcher>(&self, m: M) -> &'a [u8];
    fn trim_left_one<M: ByteMatcher>(&self, m: M) -> &'a [u8];
    fn trim_right_one<M: ByteMatcher>(&self, m: M) -> &'a [u8];
}

static EMPTY_SLICE: &'static [u8] = &[];

impl<'a> ByteSliceOps<'a> for &'a [u8] {
    fn trim_left<M: ByteMatcher>(&self, mut m: M) -> &'a [u8] {
        match self.iter().position(|&b| !m.matches(b)) {
            None => EMPTY_SLICE,
            Some(idx) => &self[idx..]
        }
    }

    fn trim_right<M: ByteMatcher>(&self, mut m: M) -> &'a [u8] {
        match self.iter().rposition(|&b| !m.matches(b)) {
            None => EMPTY_SLICE,
            Some(idx) => &self[..idx+1]
        }
    }

    #[inline]
    fn trim_left_one<M: ByteMatcher>(&self, mut m: M) -> &'a [u8] {
        match self.first() {
            Some(&c) => if m.matches(c) { &self[1..] } else { *self },
            _ => *self
        }
    }

    #[inline]
    fn trim_right_one<M: ByteMatcher>(&self, mut m: M) -> &'a [u8] {
        match self.last() {
            Some(&c) => if m.matches(c) { &self[..self.len()-1] } else { *self },
            _ => *self
        }
    }

}
