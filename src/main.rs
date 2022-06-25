#![warn(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]

mod board;
mod cell;
mod hint;

use board::Board;
use cell::Cell;
use hint::Hint;
use std::fmt;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::time::Instant;

struct HintHolder<'a> {
    _source: Pin<Vec<Vec<NonZeroUsize>>>,
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
            _source: source,
            individuals,
        }
    }
    pub fn get(&'a self) -> &'a [Hint<'a>] {
        &self.individuals
    }
}

type GuessBoard = Board<Option<Cell>>;

#[derive(Clone, Debug, Default)]
struct Picross<'a> {
    board: GuessBoard,
    row_hints: &'a [Hint<'a>],
    col_hints: &'a [Hint<'a>],
    backtrack: Vec<GuessBoard>,
}

impl<'a> Picross<'a> {
    pub fn new(row_hints: &'a [Hint<'a>], col_hints: &'a [Hint<'a>]) -> Self {
        Self {
            board: Board::new_default(col_hints.len(), row_hints.len()),
            row_hints,
            col_hints,
            ..Self::default()
        }
    }
}

impl Picross<'_> {
    pub const fn width(&self) -> usize {
        self.col_hints.len()
    }
    pub const fn height(&self) -> usize {
        self.row_hints.len()
    }
    pub fn find_solution(&mut self) -> Option<Board<Cell>> {
        loop {
            let mut progressed = false;
            for (y, hint) in self.row_hints.iter().enumerate() {
                let row = self.board.row(y);
                let new_row = hint.brute_progress(row)?;
                if row != new_row {
                    self.board.set_row(y, new_row);
                    progressed = true;
                    //println!("-----------\n{}", self.board);
                }
            }
            for (x, hint) in self.col_hints.iter().enumerate() {
                let col = self.board.col(x);
                let new_col = hint.brute_progress(&col)?;
                if col != new_col {
                    self.board.set_col(x, new_col);
                    progressed = true;
                    //println!("-----------\n{}", self.board);
                }
            }
            if progressed {
                if self.board.as_slice().iter().all(Option::is_some) {
                    let (w, h) = (self.width(), self.height());
                    let mut finished_board = Board::new_default(w, h);
                    for y in 0..h {
                        let finished_row = self
                            .board
                            .row(y)
                            .iter()
                            .copied()
                            .map(Option::unwrap)
                            .collect();
                        finished_board.set_row(y, finished_row);
                    }
                    return Some(finished_board);
                }
            } else {
                return None;
            }
        }
    }
}

impl fmt::Display for Picross<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)
    }
}

fn make_hints(s: &str) -> Option<HintHolder> {
    s.split(',')
        .map(|h| {
            h.split(' ')
                .filter(|s| !s.is_empty())
                .map(|n| n.parse().ok().and_then(NonZeroUsize::new))
                .collect()
        })
        .collect::<Option<_>>()
        .map(HintHolder::new)
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    //std::env::set_var("RUST_BACKTRACE", "full");
    let row_hints = make_hints("1, 5, 3, 1 2, 3 4, 7, 7, 7, 3 3, 5").unwrap();
    let col_hints = make_hints("3, 1 5, 2 6, 8 1, 2 6, 1 5, 5, 2, 2, 2").unwrap();
    let mut b = Picross::new(row_hints.get(), col_hints.get());
    println!("Running...");
    let start = Instant::now();
    let bs = b.find_solution();
    let time = start.elapsed();
    match bs {
        Some(solved) => println!("Found solution:\n{}", solved),
        None => println!("Failed - found partial solution:\n{}", b),
    }
    println!("Time taken: {}μs", time.as_micros());
    //assert!(bs.is_some());
}
