use crate::one_line_solver::OneLineSolver;
use crate::puzzle_crawler::{Group, PuzzleData};

#[derive(Debug, Clone, PartialEq)]
pub struct SolvedPuzzle {
    pub color_panel: Vec<String>,
    pub grid: Vec<Vec<u64>>,
}

#[derive(Debug)]
pub enum SolveError {
    TooManyColors(usize),
    Unsolvable,
}

impl std::fmt::Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyColors(count) => {
                write!(f, "too many colors to fit in a u64 mask: {count}")
            }
            Self::Unsolvable => write!(f, "puzzle cannot be solved with current constraints"),
        }
    }
}

impl std::error::Error for SolveError {}

pub fn solve_puzzle(data: PuzzleData) -> Result<SolvedPuzzle, SolveError> {
    let color_count = data.color_panel.len();
    if color_count == 0 || color_count > 63 {
        return Err(SolveError::TooManyColors(color_count));
    }
    let full_mask = (1u64 << color_count) - 1;

    let row_groups = convert_groups(&data.row_groups);
    let col_groups = convert_groups(&data.col_groups);

    let m = row_groups.len();
    let n = col_groups.len();

    let mut row_masks = vec![vec![full_mask; n]; m];
    let mut col_masks = vec![vec![full_mask; m]; n];

    let mut dead_rows = vec![false; m];
    let mut dead_cols = vec![false; n];
    let mut solver = OneLineSolver::new(m.max(n));

    let mut prev_sum = u64::MAX;
    loop {
        if !update_groups_state(&mut solver, &mut dead_rows, &row_groups, &mut row_masks) {
            return Err(SolveError::Unsolvable);
        }
        if !update_groups_state(&mut solver, &mut dead_cols, &col_groups, &mut col_masks) {
            return Err(SolveError::Unsolvable);
        }

        let cur_sum = update_cell_values(&mut row_masks, &mut col_masks);
        if cur_sum == prev_sum {
            break;
        }
        prev_sum = cur_sum;
    }

    Ok(SolvedPuzzle {
        color_panel: data.color_panel,
        grid: row_masks,
    })
}

fn convert_groups(groups: &[Vec<Group>]) -> Vec<Vec<(usize, usize)>> {
    groups
        .iter()
        .map(|row| row.iter().map(|g| (g.len, g.color_id)).collect())
        .collect()
}

fn update_groups_state(
    solver: &mut OneLineSolver,
    dead: &mut [bool],
    groups: &[Vec<(usize, usize)>],
    masks: &mut [Vec<u64>],
) -> bool {
    for (idx, group) in groups.iter().enumerate() {
        if dead[idx] {
            continue;
        }
        if !solver.update_state(group, &mut masks[idx]) {
            return false;
        }
        dead[idx] = masks[idx].iter().all(|mask| is_single_bit(*mask));
    }
    true
}

fn update_cell_values(row_masks: &mut [Vec<u64>], col_masks: &mut [Vec<u64>]) -> u64 {
    let mut total: u64 = 0;
    for row in 0..row_masks.len() {
        for col in 0..row_masks[row].len() {
            let combined = row_masks[row][col] & col_masks[col][row];
            row_masks[row][col] = combined;
            col_masks[col][row] = combined;
            total = total.wrapping_add(combined);
        }
    }
    total
}

fn is_single_bit(mask: u64) -> bool {
    mask != 0 && (mask & (mask - 1)) == 0
}

pub fn mask_to_color_index(mask: u64) -> Option<usize> {
    if is_single_bit(mask) {
        Some(mask.trailing_zeros() as usize)
    } else {
        None
    }
}
