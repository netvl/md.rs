use std::cell::Cell;

trait CellOps<T> {
    fn modify(&self, f: |T| -> T);
}

impl<T: Copy> CellOps<T> for Cell<T> {
    fn modify(&self, f: |T| -> T) {
        self.set(f(self.get()));
    }
}
