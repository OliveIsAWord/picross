use crate::cell::Cell;
use indexmap::IndexSet;
use std::num::NonZeroUsize;

#[derive(Debug, Default)]
pub struct Hint<'a> {
    lines: &'a [NonZeroUsize],
}

impl<'a> Hint<'a> {
    pub const fn new(lines: &'a [NonZeroUsize]) -> Self {
        Self { lines }
    }
}

impl Hint<'_> {
    pub const fn len(&self) -> usize {
        self.lines.len()
    }
    pub fn permutations(&self, length: usize) -> Vec<Vec<Cell>> {
        match self.lines.split_last() {
            None => vec![vec![false; length]], // no hints, which means only one solution: a blank row/column
            Some((line, &[])) => {
                let line_len = line.get(); // get usize value from NonZeroUsize
                if line_len > length {
                    return vec![];
                }
                let mut perms = Vec::with_capacity(length - line_len + 1);
                for i in 0..=length - line_len {
                    let mut v = Vec::with_capacity(length);
                    v.resize(i, false); // pad left
                    v.resize(i + line_len, true); // draw a filled line of length `line_len`
                    v.resize(length, false); // pad right
                    assert_eq!(v.len(), length);
                    perms.push(v);
                }
                perms
            }
            Some((last, rest)) => {
                let line_len = last.get();
                let last_array = [*last];
                let line = Hint {
                    // can't use Self since the slice has a shorter lifetime
                    lines: last_array.as_slice(),
                };
                let rest = Self { lines: rest };
                let mut perms = vec![];
                for i in 1..length - line_len {
                    let mut subperms = rest.permutations(i);
                    let lastperms = line.permutations(length - i - 1);
                    for p in &mut subperms {
                        p.push(false); // add padding between lines
                        for mut lp in lastperms.iter().cloned() {
                            let mut perm = p.clone();
                            perm.append(&mut lp);
                            assert_eq!(perm.len(), length);
                            perms.push(perm);
                        }
                    }
                }
                // awful, order-preserving dedup that should be `O(n)` time on average
                perms
                    .into_iter()
                    .collect::<IndexSet<_>>()
                    .into_iter()
                    .collect()
            }
        }
    }
}

fn _overlay(dst: &mut [Option<Cell>], src: &[Option<Cell>]) {
    assert_eq!(dst.len(), src.len());
    for (a, b) in dst.iter_mut().zip(src) {
        *a = match (*a, *b) {
            (v, None) | (None, v) => v,
            (Some(v1), Some(v2)) => (v1 == v2).then(|| v1),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(x: usize) -> NonZeroUsize {
        NonZeroUsize::new(x).unwrap()
    }

    #[test]
    fn basic_perms() {
        let lines = [u(3)];
        let h = Hint::new(&lines);
        let empty = [] as [[_; 0]; 0];
        assert_eq!(h.permutations(0), empty);
        assert_eq!(h.permutations(1), empty);
        assert_eq!(h.permutations(2), empty);
        assert_eq!(h.permutations(3), [[true, true, true]]);
        assert_eq!(
            h.permutations(4),
            [[true, true, true, false], [false, true, true, true]]
        );
        assert_eq!(
            h.permutations(5),
            [
                [true, true, true, false, false],
                [false, true, true, true, false],
                [false, false, true, true, true]
            ]
        );
        assert_eq!(
            h.permutations(6),
            [
                [true, true, true, false, false, false],
                [false, true, true, true, false, false],
                [false, false, true, true, true, false],
                [false, false, false, true, true, true]
            ]
        );
    }

    #[test]
    fn two_line_perms() {
        let lines = [u(2), u(3)];
        let h = Hint::new(&lines);
        let empty = [] as [[_; 0]; 0];
        assert_eq!(h.permutations(5), empty);
        assert_eq!(h.permutations(6), [[true, true, false, true, true, true]]);
        assert_eq!(
            h.permutations(7),
            [
                [true, true, false, true, true, true, false],
                [true, true, false, false, true, true, true],
                [false, true, true, false, true, true, true]
            ]
        );
        assert_eq!(
            h.permutations(8),
            [
                [true, true, false, true, true, true, false, false],
                [true, true, false, false, true, true, true, false],
                [true, true, false, false, false, true, true, true],
                [false, true, true, false, true, true, true, false],
                [false, true, true, false, false, true, true, true],
                [false, false, true, true, false, true, true, true]
            ]
        );
    }
}