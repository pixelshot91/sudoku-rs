use core::str;
use std::io::Read;

use itertools::Itertools;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq)]
#[repr(u8)]
enum Digit {
    One = 1,
    Two,
    Three,
    Four,
    // Five = 4,
    // Six = 5,
    // Seven = 6,
    // Height = 7,
    // Nine = 8,
}
impl Digit {
    fn to_char(&self) -> char {
        match self {
            Digit::One => '1',
            Digit::Two => '2',
            Digit::Three => '3',
            Digit::Four => '4',
        }
    }
}

trait Next: Sized {
    fn get_all_next(&self) -> Vec<Digit>;
}
impl Next for Cell {
    fn get_all_next(&self) -> Vec<Digit> {
        match self {
            None => Digit::iter().collect_vec(),

            Some(base_digit) => Digit::iter()
                .skip_while(|d| d != base_digit)
                .skip(1)
                .collect_vec(),
        }
    }
}

const BLOCK_SIDE: usize = 2;
const NB_DIGIT: usize = BLOCK_SIDE * BLOCK_SIDE;
const NB_CELL: usize = NB_DIGIT * NB_DIGIT;

type Cell = Option<Digit>;

/// Guarantees that no digit are in direct contradiction
/// The grid maybe unsolvable though
#[derive(Clone, Debug, PartialEq, Eq)]
struct Grid {
    data: [Cell; NB_CELL],
}

impl Grid {
    fn empty() -> Grid {
        Grid {
            data: [None; NB_CELL],
        }
    }

    /// Useful for test to visualize the grid being created
    /// 0 stand for empty cell
    /// Other digit stand for themselves
    /// PANIC if an element is not in the range 0..=NB_CELL
    #[cfg(test)]
    fn from_u8s(array: [u8; NB_CELL]) -> Grid {
        let data = array.map(|c| {
            let mut i = [None].into_iter().chain(Digit::iter().map(|d| Some(d)));
            i.nth(c.into()).unwrap()
        });
        Grid { data }
    }

    #[cfg(test)]
    fn to_u8s(&self) -> [u8; NB_CELL] {
        self.data.map(|c| c.map_or(0, |d| d as u8))
    }

    /// [try_solve] take a [Grid] as mutable reference for performance reason, but guarantees that self has the same value after this function returns
    fn try_solve<'a>(&'a self) -> GridSolver<'a> {
        GridSolver::from_grid(&self)
    }

    fn can_accept_digit_at_pos(&self, d: Digit, pos: usize) -> bool {
        let line_does_not_contain_digit = || {
            let first_cell_in_line_index = pos / NB_DIGIT * NB_DIGIT;
            (0..NB_DIGIT).all(|column| self.data[first_cell_in_line_index + column] != Some(d))
        };

        let column_does_not_contain_digit = || {
            let first_cell_in_column_index = pos % NB_DIGIT;
            (0..NB_DIGIT)
                .all(|line| self.data[first_cell_in_column_index + line * NB_DIGIT] != Some(d))
        };

        let block_does_not_contain_digit = || {
            let line_index = pos / NB_DIGIT;
            let column_index = pos % NB_DIGIT;

            let first_cell_in_block_line_index = line_index / BLOCK_SIDE * BLOCK_SIDE;
            let first_cell_in_block_column_index = column_index / BLOCK_SIDE * BLOCK_SIDE;

            (0..BLOCK_SIDE)
                .map(|y| y + first_cell_in_block_line_index)
                .all(|line| {
                    (0..BLOCK_SIDE)
                        .map(|x| x + first_cell_in_block_column_index)
                        .all(|column| self.data[line * NB_DIGIT + column] != Some(d))
                })
        };

        line_does_not_contain_digit()
            && column_does_not_contain_digit()
            && block_does_not_contain_digit()
    }
}

fn times(n: usize) -> impl Iterator {
    std::iter::repeat(()).take(n)
}
impl std::fmt::Display for Grid {
    #[allow(unstable_name_collisions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use itertools::Itertools;

        const TOP_LEFT_CORNER: char = '┌';
        const TOP_RIGHT_CORNER: char = '┐';
        const BOTTOM_RIGHT_CORNER: char = '┘';
        const BOTTOM_LEFT_CORNER: char = '└';

        const HORIZONTAL_BORDER: char = '─';
        const VERTICAL_BORDER: char = '│';

        const UP_TEE: &str = "┬";
        const RIGHT_TEE: char = '┤';
        const DOWN_TEE: &str = "┴";
        const LEFT_TEE: char = '├';

        const CROSS: &str = "┼";

        const NB_BLOCK: usize = BLOCK_SIDE;

        let line_length =
        // All digit will be on the line
        NB_DIGIT
        // As many separator as blocks
        + NB_BLOCK
        // end of block
        + 1
        // new line
        + 1;

        // TODO: allocate only the right amount, then only use push or push_str, but od not create extra String
        let mut s = String::with_capacity(line_length * line_length);

        // str::from_utf8(HORIZONTAL_TEE)
        // vec!["ds", "fds"].iter().as_slice().join(sep);

        // First border line
        s.push(TOP_LEFT_CORNER);
        s.push_str(
            &times(NB_BLOCK)
                .map(|_| times(BLOCK_SIDE).map(|_| HORIZONTAL_BORDER).join(""))
                .join(UP_TEE),
        );
        s.push(TOP_RIGHT_CORNER);
        s.push('\n');

        let horizontal_border_line = {
            let mut s = LEFT_TEE.to_string();
            s.push_str(
                &times(BLOCK_SIDE)
                    .map(|_| times(BLOCK_SIDE).map(|_| HORIZONTAL_BORDER).join(""))
                    .join(CROSS),
            );
            s.push(RIGHT_TEE);
            s.push('\n');
            s
        };

        let body = (0..NB_BLOCK)
            .map(|block_y_index| {
                (0..BLOCK_SIDE)
                    .map(|line_in_block| {
                        let line = block_y_index * BLOCK_SIDE + line_in_block;
                        let mut number_line = String::new();
                        number_line.push(VERTICAL_BORDER);
                        let number_line_body = (0..NB_BLOCK)
                            .map(|block_x_index| {
                                (0..BLOCK_SIDE)
                                    .map(|column_in_block| {
                                        let column = block_x_index * BLOCK_SIDE + column_in_block;
                                        let cell = self.data[line * NB_DIGIT + column];
                                        match cell {
                                            None => '.',
                                            Some(d) => d.to_char(),
                                        }
                                    })
                                    .join("")
                            })
                            .join(&VERTICAL_BORDER.to_string());
                        number_line.push_str(&number_line_body);

                        number_line.push(VERTICAL_BORDER);
                        number_line.push('\n');

                        number_line
                    })
                    .join("")
            })
            .join(&horizontal_border_line);

        s.push_str(&body);

        // Bottom border line
        s.push(BOTTOM_LEFT_CORNER);
        s.push_str(
            &times(NB_BLOCK)
                .map(|_| times(BLOCK_SIDE).map(|_| HORIZONTAL_BORDER).join(""))
                .join(DOWN_TEE),
        );
        s.push(BOTTOM_RIGHT_CORNER);
        s.push('\n');

        f.write_str(&s)
    }
}

/// All Cell in [grid] strictly before the cell at index [fill_until] are filled
/// Cell after fill_until may or may not be filled
/// All cells are guaranteed to not contradict with each other, per [Grid] guarantee
struct PartialySolvedGrid {
    grid: Grid,
    fill_until: usize,
}

impl PartialySolvedGrid {
    fn try_fill_next_cell(&mut self) -> bool {
        if self.fill_until == self.grid.data.len() {
            return false;
        }
        match self.grid.data[self.fill_until] {
            Some(_) => {
                // a digit is already here
                self.fill_until += 1;
                true
            }
            None => {
                for d in Digit::iter() {
                    if self.grid.can_accept_digit_at_pos(d, self.fill_until) {
                        self.grid.data[self.fill_until] = Some(d);
                        self.fill_until += 1;
                        return true;
                    }
                }
                // No digit can fit in the first empty cell. We should backtrack
                false
            }
        }
    }

    fn try_increment_cell_at_index(&mut self, cell_index: usize) -> bool {
        let original_digit = self.grid.data[cell_index].take();
        let d = original_digit;
        for d in d.get_all_next() {
            if self.grid.can_accept_digit_at_pos(d, cell_index) {
                self.grid.data[cell_index] = Some(d);
                return true;
            }
        }
        self.fill_until -= 1;
        false
    }
}

impl std::fmt::Display for PartialySolvedGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.grid.fmt(f)
    }
}

struct GridSolver<'a> {
    initial_grid: &'a Grid,
    psg: PartialySolvedGrid,
}

impl<'a> GridSolver<'a> {
    fn from_grid(grid: &'a Grid) -> GridSolver<'a> {
        GridSolver {
            initial_grid: grid,
            psg: PartialySolvedGrid {
                grid: grid.clone(),
                fill_until: 0,
            },
        }
    }

    // Either fill the next cell, or backtrack until a previous cell can be incremented
    // If we see the grid digit in a list and interpret that as a number (empty cell meaning 0),
    // then this number after this function should be strictly greather than before calling the function
    // Return if a progress has been made
    // Returning false mean there is no more solution to be found
    fn make_progress(&mut self) -> bool {
        match self.psg.try_fill_next_cell() {
            // The cell has been filled, continue this way
            true => true,
            // No cell could have been filled: we are at a dead-end: backtrack
            false => {
                fn guessed_cells(
                    self_psg_fill_until: &usize,
                    self_initial_grid_data: &[Cell; NB_CELL],
                ) -> Vec<usize> {
                    (0..*self_psg_fill_until)
                        .rev()
                        // Only keep the cell which were empty in the initial grid
                        .filter(|cell_index| self_initial_grid_data[*cell_index].is_none())
                        .collect::<Vec<usize>>()
                }

                let guessed_cells = guessed_cells(&self.psg.fill_until, &self.initial_grid.data);
                for guessed_cell in guessed_cells {
                    if self.psg.try_increment_cell_at_index(guessed_cell) {
                        // the last guessed cell has been incremented,
                        // TODO: break out of the little loop, but stay inside the big loop
                        return true;
                    }
                }
                // Could not increment any of the already filled cells
                // We already know that the next cannot be filled either
                // There is no more solution
                false
            }
        }
    }
}

impl<'a> Iterator for GridSolver<'a> {
    type Item = SolvedGrid;

    fn next(&mut self) -> Option<Self::Item> {
        // The only way out of this loop is to either:
        // - return a possible solution
        // - exhaust all possible solution, then return
        loop {
            if self.psg.fill_until == NB_CELL {
                let result = SolvedGrid::from_psg(&self.psg);
                self.make_progress();
                return Some(result);
            }

            if self.make_progress() == false {
                return None;
            }
        }
    }
}

/// Like PartiallySolvedGrid, but with fill_until = NB_CELL
/// So:
///  - No cell contradict each other
///  - All cells are filled
/// So the grid is solved
#[derive(Debug)]
struct SolvedGrid {
    grid: Grid,
    // data: [Digit; NB_CELL],
}

impl SolvedGrid {
    fn from_psg(psg: &PartialySolvedGrid) -> SolvedGrid {
        assert_eq!(psg.fill_until, NB_CELL);
        psg.grid.data.iter().for_each(|c| assert!(c.is_some()));

        SolvedGrid {
            grid: psg.grid.clone(),
        }
        // SolvedGrid {
        //     data: psg.grid.data.map(|maybe_digit| maybe_digit.expect("Because fill_until == NB_CELL, and data.len() == fill_until, digit should always be Some"))
        // }
    }
    // fn from(grid: Grid) -> SolvedGrid {
    //     SolvedGrid {
    //         data: grid.data.map(|maybe_digit| maybe_digit.value.unwrap())
    //     }
    // }
}

impl std::fmt::Display for SolvedGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.grid.fmt(f)
    }
}

fn main() {
    let grid = Grid::empty();
    let mut solver = grid.try_solve();

    loop {
        assert!(solver.make_progress());

        println!("{}", solver.psg);

        std::io::stdin().read(&mut [0u8]).unwrap();
    }

    return;

    for solution in grid.try_solve() {
        println!("{}", solution)
    }
}

#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;

    use crate::{times, Digit, Grid, Next, NB_CELL};

    #[test]
    fn digit_next() {
        assert_eq!(Some(Digit::Two).get_all_next().len(), 2);
        assert_eq!(None.get_all_next().len(), 4);
    }

    #[test]
    fn iter_solutions() {
        let grid = Grid::empty();
        let mut solver = grid.try_solve();

        let first_solution = solver.next().unwrap();

        #[rustfmt::skip]
        let expected = Grid::from_u8s([
            1, 2, 3, 4,
            3, 4, 1, 2,
            2, 1, 4, 3,
            4, 3, 2, 1
        ]);
        assert_eq!(first_solution.grid, expected);

        let second_solution = solver.next().unwrap();

        println!("{}", &second_solution);
        dbg!(second_solution.grid.to_u8s());

        #[rustfmt::skip]
        let expected = Grid::from_u8s([
            1, 2, 3, 4,
            3, 4, 1, 2,
            2, 3, 4, 1,
            4, 1, 2, 3
        ]);
        assert_eq!(second_solution.grid, expected);
    }

    #[test]
    fn make_progress_on_full_grid() {
        let grid = Grid::empty();
        let mut solver = grid.try_solve();

        times(NB_CELL).for_each(|_| assert!(solver.make_progress()));

        assert_eq!(solver.psg.fill_until, NB_CELL);
        println!("{}", solver.psg);

        assert!(solver.make_progress());

        #[rustfmt::skip]
        let expected = Grid::from_u8s([
                1, 2, 3, 4,
                3, 4, 1, 2,
                2, 3, 0, 0,
                0, 0, 0, 0,
            ]);

        assert_eq!(solver.psg.grid, expected);

        println!("{}", solver.psg);
    }

    #[test]
    fn make_progress_on_empty_grid() {
        let grid = Grid::empty();
        let mut solver = grid.try_solve();
        assert!(solver.make_progress());

        println!("{}", solver.psg);
    }

    #[test]
    fn display_empty_grid() {
        let grid = Grid::empty();
        let s = grid.to_string();
        assert_eq!(
            s,
            r"┌──┬──┐
│..│..│
│..│..│
├──┼──┤
│..│..│
│..│..│
└──┴──┘
"
        );
    }
}
