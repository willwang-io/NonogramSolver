# Nonogram Solver

A Rust + Dioxus implementation of a colored Nonogram solver. The original version was written in Python with Processing and is preserved in the `archive/` folder.

This project started as an Advanced Algorithms course project and case study. I chose this topic because I was addicted to these puzzles at the time. The approach is inspired by Batenburg and Kosters’ paper, *Solving Nonograms by Combining Relaxations[^1]*. The solver is not just a brute-force DP routine — it models the kind of incremental, constraint-based reasoning a human solver uses, but in a systematic and efficient way. Note that a black-and-white Nonogram is a special case of a colored Nonogram with only two colors.

## Features

- Color and black-and-white puzzles from [nonograms.org](https://www.nonograms.org/)
- Incremental solver with step-by-step visualization 
- Adaptive cell sizing for small and large grids
- Minimal UI with palette preview and puzzle size

## Quick start

### Prerequisites

- Rust (stable)
- Dioxus CLI (`dx`)

Install Dioxus CLI:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
```

### Run the web app

```sh
dx serve
```

Open the dev server URL (typically `http://127.0.0.1:8080`).

## Usage

Paste a puzzle URL or ID into the input box:

- Color: `https://www.nonograms.org/nonograms2/i/79575`
- Black/white: `https://www.nonograms.org/nonograms/i/1822`
- Or just the ID (defaults to color)

Use the slider below the grid to step through the solving process.

## How it works

- The crawler fetches the puzzle page and decodes the embedded `var d = [...]` data.
- The solver runs a line-by-line dynamic program to refine possible colors per cell.
- Cell state is stored as a bitmask: bit 0 = white, bit i = color i. Intersections are computed with bitwise AND, and a cell is “solved” when exactly one bit remains.
- Each iteration is recorded as a step for visualization.

## Project layout

- `src/main.rs`: app entry, input handling, server function
- `src/components/`: UI components (`PuzzleViewer`, `PuzzleGrid`)
- `src/nonogram_solver.rs`: incremental solver + step generation
- `src/one_line_solver.rs`: line solver (DP)
- `src/puzzle_crawler.rs`: nonograms.org decoder (color + BW)
- `assets/style.css`: UI styles
- `archive/`: original Python/Processing version

## Notes

- The web app uses a server function as a proxy to avoid browser CORS limits.
- Black-and-white puzzles use a fixed palette: white and black.

[^1]: https://www.sciencedirect.com/science/article/abs/pii/S0031320308005153
