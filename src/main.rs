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
    pub num_backtracks: usize,
}

impl<'a> Picross<'a> {
    pub fn new(row_hints: &'a [Hint<'a>], col_hints: &'a [Hint<'a>]) -> Self {
        Self {
            board: Board::new_default(col_hints.len(), row_hints.len()),
            row_hints,
            col_hints,
            backtrack: vec![],
            num_backtracks: 0,
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
            //print!("å");
            let mut progressed = false;
            let mut backtracked = false;
            for (y, hint) in self.row_hints.iter().enumerate() {
                let row = self.board.row(y);
                let new_row = match hint.brute_progress(row) {
                    Some(r) => r,
                    None => {
                        self.board = self.backtrack.pop()?;
                        backtracked = true;
                        break;
                    }
                };
                if row != new_row {
                    // if y >= 15 {
                    //     eprintln!("wtf???");
                    //     dbg!(y);
                    //     eprintln!("row     {:?}", row);
                    //     eprintln!("new_row {:?}", new_row);
                    //     eprintln!("{}", self);
                    // }
                    self.board.set_row(y, new_row);
                    progressed = true;
                    //println!("-----------\n{}", self.board);
                }
            }
            if backtracked {
                continue;
            }
            for (x, hint) in self.col_hints.iter().enumerate() {
                let col = self.board.col(x);
                let new_col = match hint.brute_progress(&col) {
                    Some(c) => c,
                    None => {
                        self.board = self.backtrack.pop()?;
                        backtracked = true;
                        break;
                    }
                };
                if col != new_col {
                    self.board.set_col(x, new_col);
                    progressed = true;
                    //println!("-----------\n{}", self.board);
                }
            }
            if backtracked {
                continue;
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
                let i = self
                    .board
                    .as_slice()
                    .iter()
                    .enumerate()
                    .find_map(|(i, v)| v.is_none().then(|| i));
                match i {
                    Some(i) => {
                        //unreachable!();
                        // Found an unsolved cell, branch into two different boards where that cell is filled or unfilled.
                        let mut alternate = self.board.clone();
                        self.board.as_slice_mut()[i] = Some(true);
                        alternate.as_slice_mut()[i] = Some(false);
                        self.backtrack.push(alternate);
                        self.num_backtracks += 1;
                        println!("uwu {} -> {}", self.num_backtracks, self.backtrack.len());
                    }
                    None => {
                        // If all cells are solved, attempt to backtrack.
                        self.board = self.backtrack.pop()?;
                        println!("owo");
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
            h.split_whitespace()
                //.filter(|s| !s.is_empty())
                .map(|n| n.parse().ok().and_then(NonZeroUsize::new))
                .collect()
        })
        .collect::<Option<_>>()
        .map(HintHolder::new)
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    //std::env::set_var("RUST_BACKTRACE", "full");
    let _row_hints = make_hints("2 1 1 1 1 1 1, 5 1 1 4, 4 4, 1 1, 3, 7, 6, 3, 1 3, 2 4, 3 8, 13, 12, 12, 15, 6 2 1, 6 1 2 1, 2 1 1 2 3 1 1, 2 2 1 4 1 2 1, 5 1 2 1 2 1 2").unwrap();
    let _col_hints = make_hints("2 4, 4 6, 7 1, 1 9, 2 6 2, 1 6, 2 6 3, 1 5, 3 5 2, 1 1 10, 1 2 7 2, 15 2, 11, 15 3, 1 2 1 2 1, 1 2 4, 3 2, 1 3, 2 1, 1 3").unwrap();
    let _row_hints =
        make_hints("2, 1 1, 1 2, 1 3, 1 1 2, 1 1 2, 1 2 1, 1 5, 1 3, 3, 3, 1 2, 1 2 2, 4 5, 2 5")
            .unwrap();
    let _col_hints =
        make_hints("6, 5, 4 2, 6, 7 3, 2 3, 2 2, 2 2 1, 3 1, 1 1, 1 2, 2, 1, 2, 2").unwrap();
    let row_hints = make_hints(
        "4, 7, 3 2 3, 4 4 1 4, 3 2 3 2 2,
        1 8 3, 2 4 2, 2 2 2, 1 6 2, 3 6 3,
        3 3 3 2 2, 6 1 1 4, 5 3, 6, 3",
    )
    .unwrap();
    let col_hints = make_hints(
        "1 1 1 1, 1 1 1 1, 4 5, 4 5, 3 1 5,
        2 3 1 3, 4 3 1, 2 2 3, 6 1, 2 3,
        2 2, 7, 9, 3 3, 2 1 1 2,
        3 3, 3 3, 1 1 1 1, 7, 7",
    )
    .unwrap();
    //let (row_hints, col_hints) = (col_hints, row_hints);
    //assert_eq!(row_hints.get().len(), 15);
    //assert_eq!(col_hints.get().len(), 20);
    let mut b = Picross::new(row_hints.get(), col_hints.get());
    let mut successful = false;
    println!("Running...");
    loop {
        let start = Instant::now();
        let bs = b.find_solution();
        let time = start.elapsed();
        let uwu = bs.is_none();
        match bs {
            Some(solved) => {
                println!("Found solution:\n{}", solved);
                if b.num_backtracks > 0 {
                    println!("Required {} backtracks.", b.num_backtracks);
                } else {
                    println!("No backtracking required.");
                }
                successful = true;
            }
            None => {
                if successful {
                    println!("No more solutions found.")
                } else {
                    println!("Failed - found partial solution:\n{}", b)
                }
                if b.num_backtracks > 0 {
                    println!("Required {} backtracks.", b.num_backtracks);
                } else {
                    println!("No backtracking required.");
                }
            }
        }
        println!("Time taken: {}μs", time.as_micros());
        if uwu {
            break;
        }
    }
    //assert!(bs.is_some());
}
