#![warn(unsafe_op_in_unsafe_fn)]
#![allow(dead_code, unused_variables)]

mod board;
mod cell;
mod hint;

use cell::Cell;
use hint::Hint;
use std::fmt;
use std::num::NonZeroUsize;
use std::pin::Pin;

struct HintHolder<'a> {
    source: Pin<Vec<Vec<NonZeroUsize>>>,
    individuals: Vec<Hint<'a>>,
}

impl<'a> HintHolder<'a> {
    pub fn new(hints: Vec<Vec<NonZeroUsize>>) -> Self {
        let source = Pin::new(hints);
        let individuals = source
            .as_ref()
            .iter()
            .map(|v| Hint::new(unsafe { &*(v.as_slice() as *const [NonZeroUsize]) }))
            .collect();
        Self {
            source,
            individuals,
        }
    }
    pub fn get(&'a self) -> &'a [Hint<'a>] {
        &self.individuals
    }
}

#[derive(Debug, Default)]
struct Board<'a> {
    board: Vec<Option<Cell>>,
    row_hints: &'a [Hint<'a>],
    col_hints: &'a [Hint<'a>],
    _backtrack: Vec<Self>,
}

impl<'a> Board<'a> {
    pub fn new(row_hints: &'a [Hint<'a>], col_hints: &'a [Hint<'a>]) -> Self {
        let num_cells = row_hints.len() * col_hints.len();
        Self {
            board: vec![None; num_cells],
            row_hints,
            col_hints,
            ..Self::default()
        }
    }
}

impl Board<'_> {
    pub const fn width(&self) -> usize {
        self.col_hints.len()
    }
    pub const fn height(&self) -> usize {
        self.row_hints.len()
    }

    pub fn rows(&self) -> Vec<Vec<Option<Cell>>> {
        self.board
            .chunks_exact(self.width())
            .map(<[_]>::to_vec)
            .collect()
    }
    pub fn cols(&self) -> Vec<Vec<Option<Cell>>> {
        let (w, h) = (self.width(), self.height());
        let mut v = Vec::with_capacity(w);
        for x in 0..w {
            let mut col = Vec::with_capacity(h);
            for y in 0..h {
                col.push(self.board[x + y * w]);
            }
            v.push(col);
        }
        v
    }
    pub fn find_solution(&mut self) -> Option<Vec<Cell>> {
        let _ = self;
        todo!()
    }
}

impl fmt::Display for Board<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height() {
            if y > 0 {
                writeln!(f)?;
            }
            for x in 0..self.width() {
                let c = match self.board[x + y * self.width()] {
                    None => '?',
                    Some(false) => '.',
                    Some(true) => 'X',
                };
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

fn make_hints(s: &str) -> Option<HintHolder> {
    s.split(", ")
        .map(|h| {
            h.split(' ')
                .map(|n| n.parse().ok().and_then(NonZeroUsize::new))
                .collect()
        })
        .collect::<Option<_>>()
        .map(HintHolder::new)
}

fn main() {
    //let u = |x| NonZeroUsize::new(x).unwrap();
    // let row_hints = vec![vec![u(4)], vec![u(1)], vec![u(5)], vec![u(1)]];
    // let col_hints = vec![vec![u(3)], vec![u(1), u(1)], vec![u(1), u(1)], vec![u(1), u(1)], vec![u(2)]];
    let row_hints = make_hints("4, 1, 5, 1").unwrap();
    let col_hints = make_hints("3, 1 1, 1 1, 1 1, 2").unwrap();
    let mut b = Board::new(row_hints.get(), col_hints.get());
    println!("{}", b);
    let bs = b.find_solution().unwrap();
    println!("{:?}", bs);
}
