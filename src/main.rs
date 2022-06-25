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
            backtrack: vec![],
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
    pub fn get_solutions(&mut self) -> Vec<Board<Cell>> {
        let mut solutions = vec![];
        while let Some(solution) = self.find_solution() {
            solutions.push(solution);
        }
        solutions
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
                    // Found a solution
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
                // Solver got stuck, do bifurcation
                // TODO: Does this code only execute if there are multiple solutions?
                // First, find unsolved cell
                let i = self.board
                    .as_slice()
                    .iter()
                    .enumerate()
                    .find_map(|(i, v)| v.is_none().then(|| i));
                match i {
                    Some(i) => {
                        // Found an unsolved cell, branch into two different boards where that cell is filled or unfilled.
                        let mut alternate = self.board.clone();
                        self.board.as_slice_mut()[i] = Some(true);
                        alternate.as_slice_mut()[i] = Some(false);
                        self.backtrack.push(alternate);
                    }
                    None => {
                        // If all cells are solved, attempt to backtrack.
                        self.board = self.backtrack.pop()?;
                    }
                }
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
    let row_hints = make_hints("1, 1").unwrap();
    let col_hints = make_hints("1, 1").unwrap();
    let mut b = Picross::new(row_hints.get(), col_hints.get());
    println!("Running...");
    loop {
        let start = Instant::now();
        let bs = b.find_solution();
        let time = start.elapsed();
        let mut uwu = false;
        match bs {
            Some(solved) => println!("Found solution:\n{}", solved),
            None => {
                println!("Failed - found partial solution:\n{}", b);
                uwu = true;
            }
        }
        println!("Time taken: {}μs", time.as_micros());
        if uwu {
            break;
        }
    }
    //assert!(bs.is_some());
}
