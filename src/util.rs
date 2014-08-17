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
}
