use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq)]
enum Digit {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Height,
    Nine,
}

const BLOCK_SIDE: usize = 3;
const NB_DIGIT: usize = BLOCK_SIDE * BLOCK_SIDE;
const NB_CELL: usize = NB_DIGIT * NB_DIGIT;

#[derive(Clone, Copy, Debug)]
struct Cell {
    value: Option<Digit>,
}

/// Guarantees that no digit are in direct contradiction
/// The grid maybe unsolvable though
struct Grid {
    data: [Cell; NB_CELL as usize],
}

impl Grid {
    fn empty() -> Grid {
        Grid {
            data: [Cell { value: None }; NB_CELL],
        }
    }

    /// [try_solve] take a [Grid] as mutable reference for performance reason, but guarantees that self has the same value after this function returns
    fn try_solve(&mut self) -> GridSolver {
        GridSolver { initial_grid: self }
    }

    fn can_accept_digit_at_pos(&self, d: Digit, pos: usize) -> bool {
        let line_does_not_contain_digit = || {
            let first_cell_in_line_index = pos / NB_DIGIT * NB_DIGIT;
            (0..NB_DIGIT)
                .all(|column| self.data[first_cell_in_line_index + column].value != Some(d))
        };

        let column_does_not_contain_digit = || {
            let first_cell_in_column_index = pos % NB_DIGIT;
            (0..NB_DIGIT).all(|line| {
                self.data[first_cell_in_column_index + line * NB_DIGIT].value != Some(d)
            })
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
                        .all(|column| self.data[line * NB_DIGIT + column].value != Some(d))
                })
        };

        line_does_not_contain_digit()
            && column_does_not_contain_digit()
            && block_does_not_contain_digit()
    }
}

/// All Cell in [grid] strictly before the cell at index [fill_until] are filled
/// Cell after fill_until may or may not be filled
/// All cells are guaranteed to not contradict with each other, per [Grid] guarantee
struct PartialySolvedGrid<'a> {
    grid: &'a mut Grid,
    fill_until: usize,
}

impl<'a> PartialySolvedGrid<'a> {
    fn try_fill_next_cell(&'a mut self) -> bool {
        match self.grid.data[self.fill_until].value {
            Some(_) => {
                // a digit is already here
                self.fill_until += 1;
                true
            }
            None => {
                for d in Digit::iter() {
                    if self.grid.can_accept_digit_at_pos(d, self.fill_until) {
                        self.fill_until += 1;
                        return true;
                    }
                }
                // No digit can fit in the first empty cell. We should backtrack
                false
            }
        }
    }
}

struct GridSolver<'a> {
    initial_grid: &'a mut Grid,
}

impl<'a> Iterator for GridSolver<'a> {
    type Item = SolvedGrid;

    fn next(&mut self) -> Option<Self::Item> {
        let mut psg = PartialySolvedGrid {
            grid: self.initial_grid,
            fill_until: 0,
        };

        psg.try_fill_next_cell();

        // let data = &mut self.initial_grid.data;

        // for i in 0..data.len() {
        //     match data[i].value {

        //         Some(_) => {
        //             // a digit is already here
        //             continue;
        //         },
        //         None => {
        //             for d in Digit::iter() {

        //             }
        //             data[i] =
        //         },

        //     }
        // }

        /* let empty_cells = data.iter().filter(|cell| cell.value.is_none());
        for empty_cell in empty_cells {
            data[0] = Cell {value: Some(Digit::One)}
        } */
        None
        /*  match first_empty_cell {
            None => {

            },

            Some(_) => {
                todo!()
            },
        } */
    }
}

/// Like PartiallySolvedGrid, but with fill_until = NB_CELL
/// So:
///  - No cell contradict each other
///  - All cells are filled
/// So the grid is solved
struct SolvedGrid {
    data: [Digit; NB_CELL],
}

impl SolvedGrid {
    fn from_psg(psg: PartialySolvedGrid) -> SolvedGrid {
        assert_eq!(psg.fill_until, NB_CELL);
        SolvedGrid {
            data: psg.grid.data.map(|maybe_digit| maybe_digit.value.expect("Because fill_until == NB_CELL, and data.len() == fill_until, digit should always be Some"))
        }
    }
    // fn from(grid: Grid) -> SolvedGrid {
    //     SolvedGrid {
    //         data: grid.data.map(|maybe_digit| maybe_digit.value.unwrap())
    //     }
    // }
}

fn main() {
    let mut grid = Grid::empty();

    for solution in grid.try_solve() {}
}
