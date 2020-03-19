#[derive(Debug)]
pub struct BitIteratorLE<E> {
    t: E,
    n: usize,
    sz: usize
}

impl<E: AsRef<[u64]>> BitIteratorLE<E> {
    pub fn new(t: E) -> Self {
        let sz = t.as_ref().len() * 64;

        BitIteratorLE { t, n:0, sz }
    }
}

impl<E: AsRef<[u64]>> Iterator for BitIteratorLE<E> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.n == self.sz {
            None
        } else {
            let part = self.n / 64;
            let bit = self.n - (64 * part);
            self.n += 1;

            Some(self.t.as_ref()[part] & (1 << bit) > 0)
        }
    }
}
