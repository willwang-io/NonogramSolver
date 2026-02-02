use nonogram_solver::nonogram_solver::{mask_to_color_index, solve_puzzle};
use nonogram_solver::puzzle_crawler::{fetch_puzzle, Group, PuzzleKind};

fn groups_from_grid(grid: &[Vec<u64>]) -> (Vec<Vec<Group>>, Vec<Vec<Group>>) {
    let rows = grid.len();
    let cols = grid.first().map(|row| row.len()).unwrap_or(0);
    let mut row_groups = Vec::with_capacity(rows);
    for row in grid {
        let mut groups = Vec::new();
        let mut col = 0;
        while col < cols {
            let color = mask_to_color_index(row[col]).expect("unsolved cell");
            let start = col;
            while col < cols && mask_to_color_index(row[col]).unwrap() == color {
                col += 1;
            }
            let len = col - start;
            if color > 0 && len > 0 {
                groups.push(Group { len, color_id: color });
            }
        }
        row_groups.push(groups);
    }

    let mut col_groups = Vec::with_capacity(cols);
    for col in 0..cols {
        let mut groups = Vec::new();
        let mut row = 0;
        while row < rows {
            let color = mask_to_color_index(grid[row][col]).expect("unsolved cell");
            let start = row;
            while row < rows && mask_to_color_index(grid[row][col]).unwrap() == color {
                row += 1;
            }
            let len = row - start;
            if color > 0 && len > 0 {
                groups.push(Group { len, color_id: color });
            }
        }
        col_groups.push(groups);
    }
    (row_groups, col_groups)
}

fn network_tests_enabled() -> bool {
    std::env::var("RUN_NETWORK_TESTS").is_ok()
}

#[tokio::test]
async fn solves_color_puzzle_matches_hints() -> Result<(), Box<dyn std::error::Error>> {
    if !network_tests_enabled() {
        eprintln!("skipping: set RUN_NETWORK_TESTS=1 to enable");
        return Ok(());
    }
    let data = fetch_puzzle(PuzzleKind::Color, "79575").await?;
    let solved = solve_puzzle(data.clone())?;
    let (row_out, col_out) = groups_from_grid(&solved.grid);
    assert_eq!(row_out, data.row_groups);
    assert_eq!(col_out, data.col_groups);
    Ok(())
}

#[tokio::test]
async fn solves_black_white_puzzle_matches_hints() -> Result<(), Box<dyn std::error::Error>> {
    if !network_tests_enabled() {
        eprintln!("skipping: set RUN_NETWORK_TESTS=1 to enable");
        return Ok(());
    }
    let data = fetch_puzzle(PuzzleKind::BlackWhite, "1822").await?;
    let solved = solve_puzzle(data.clone())?;
    let (row_out, col_out) = groups_from_grid(&solved.grid);
    assert_eq!(row_out, data.row_groups);
    assert_eq!(col_out, data.col_groups);
    Ok(())
}
