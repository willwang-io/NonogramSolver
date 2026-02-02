//! Crawl a color nonogram puzzle from nonograms.org.
//!
//! This only handles color puzzles (not black-white) and keeps everything in memory.

use serde::{Deserialize, Serialize};

const COLOR_URL: &str = "https://www.nonograms.org/nonograms2/i/";
const BW_URL: &str = "https://www.nonograms.org/nonograms/i/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuzzleKind {
    Color,
    BlackWhite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub len: usize,
    pub color_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PuzzleData {
    pub color_panel: Vec<String>,
    pub row_groups: Vec<Vec<Group>>,
    pub col_groups: Vec<Vec<Group>>,
}

#[derive(Debug)]
pub enum CrawlError {
    Network(String),
    MissingData(&'static str),
    InvalidData(&'static str),
}

impl std::fmt::Display for CrawlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "network error: {msg}"),
            Self::MissingData(label) => write!(f, "missing data: {label}"),
            Self::InvalidData(label) => write!(f, "invalid data: {label}"),
        }
    }
}

impl std::error::Error for CrawlError {}

/// Fetch and parse a color puzzle into structured data.
///
/// ```no_run
/// # use nonogram_solver::puzzle_crawler::fetch_color_puzzle;
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let data = fetch_color_puzzle("19048").await?;
/// assert_eq!(data.color_panel[0], "#ffffff");
/// # Ok(())
/// # }
/// ```
pub async fn fetch_color_puzzle(puzzle_id: &str) -> Result<PuzzleData, CrawlError> {
    fetch_puzzle(PuzzleKind::Color, puzzle_id).await
}

pub async fn fetch_puzzle(kind: PuzzleKind, puzzle_id: &str) -> Result<PuzzleData, CrawlError> {
    let html = fetch_html(kind, puzzle_id).await?;
    parse_puzzle(kind, &html)
}

/// Parse a puzzle from a page's HTML.
pub fn parse_puzzle(kind: PuzzleKind, html: &str) -> Result<PuzzleData, CrawlError> {
    let data = extract_d_array(html)?;
    decode_puzzle_data(kind, &data)
}

async fn fetch_html(kind: PuzzleKind, puzzle_id: &str) -> Result<String, CrawlError> {
    let base = match kind {
        PuzzleKind::Color => COLOR_URL,
        PuzzleKind::BlackWhite => BW_URL,
    };
    let url = format!("{base}{puzzle_id}");
    let response = reqwest::get(url)
        .await
        .map_err(|e| CrawlError::Network(e.to_string()))?;
    let status = response.status();
    if !status.is_success() {
        return Err(CrawlError::Network(format!("HTTP {status}")));
    }
    response
        .text()
        .await
        .map_err(|e| CrawlError::Network(e.to_string()))
}

fn extract_d_array(html: &str) -> Result<Vec<[i64; 4]>, CrawlError> {
    let marker = "var d=";
    let start = html
        .find(marker)
        .ok_or(CrawlError::MissingData("var d array"))?;
    let after = &html[start + marker.len()..];
    let end = after
        .find("];")
        .ok_or(CrawlError::MissingData("d array end"))?;
    let slice = &after[..end + 1];

    let mut nums = Vec::new();
    let mut cur: Option<i64> = None;
    let mut sign: i64 = 1;
    for ch in slice.chars() {
        if ch == '-' {
            if cur.is_some() {
                nums.push(sign * cur.take().unwrap());
            }
            sign = -1;
            cur = Some(0);
        } else if ch.is_ascii_digit() {
            let digit = (ch as i64) - ('0' as i64);
            cur = Some(cur.unwrap_or(0) * 10 + digit);
        } else if let Some(value) = cur.take() {
            nums.push(sign * value);
            sign = 1;
        }
    }
    if let Some(value) = cur {
        nums.push(sign * value);
    }

    if nums.len() % 4 != 0 {
        return Err(CrawlError::InvalidData("d array length"));
    }

    let mut out = Vec::with_capacity(nums.len() / 4);
    for chunk in nums.chunks(4) {
        out.push([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    Ok(out)
}

fn decode_puzzle_data(kind: PuzzleKind, data: &[[i64; 4]]) -> Result<PuzzleData, CrawlError> {
    if data.len() < 6 {
        return Err(CrawlError::InvalidData("d array too short"));
    }

    let mod_js = |a: i64, b: i64| {
        if b == 0 {
            0
        } else {
            let r = a % b;
            if r < 0 { r + b } else { r }
        }
    };

    let d = data;
    let cols = mod_js(d[1][0], d[1][3]) + mod_js(d[1][1], d[1][3]) - mod_js(d[1][2], d[1][3]);
    let rows = mod_js(d[2][0], d[2][3]) + mod_js(d[2][1], d[2][3]) - mod_js(d[2][2], d[2][3]);
    let colors = mod_js(d[3][0], d[3][3]) + mod_js(d[3][1], d[3][3]) - mod_js(d[3][2], d[3][3]);

    if rows <= 0 || cols <= 0 || colors <= 0 {
        return Err(CrawlError::InvalidData("decoded dimensions"));
    }

    let rows = rows as usize;
    let cols = cols as usize;
    let colors = colors as usize;

    if d.len() < 5 + colors {
        return Err(CrawlError::InvalidData("color data truncated"));
    }

    let base = d[4];
    let mut color_panel = Vec::with_capacity(colors + 1);
    match kind {
        PuzzleKind::BlackWhite => {
            color_panel.push("#ffffff".to_string());
            color_panel.push("#000000".to_string());
        }
        PuzzleKind::Color => {
            color_panel.push("#ffffff".to_string());
            for i in 0..colors {
                let entry = d[5 + i];
                let r = ((entry[0] - base[1]) % 256 + 256) % 256;
                let g = ((entry[1] - base[0]) % 256 + 256) % 256;
                let b = ((entry[2] - base[3]) % 256 + 256) % 256;
                color_panel.push(format!("#{:02x}{:02x}{:02x}", r, g, b));
            }
        }
    }

    let v_idx = colors + 5;
    if d.len() <= v_idx + 1 {
        return Err(CrawlError::InvalidData("grid data truncated"));
    }
    let ha = mod_js(d[v_idx][0], d[v_idx][3]) * mod_js(d[v_idx][0], d[v_idx][3])
        + mod_js(d[v_idx][1], d[v_idx][3]) * 2
        + mod_js(d[v_idx][2], d[v_idx][3]);

    let ia = d[v_idx + 1];
    let mut grid = vec![vec![0i64; cols]; rows];
    let max_idx = v_idx + 1 + (ha as usize);
    if d.len() <= max_idx {
        return Err(CrawlError::InvalidData("grid data out of bounds"));
    }

    for entry in &d[(v_idx + 2)..=max_idx] {
        let row = entry[3] - ia[3] - 1;
        let start = entry[0] - ia[0] - 1;
        let len = entry[1] - ia[1];
        let color = entry[2] - ia[2];
        if row < 0 || start < 0 || len <= 0 {
            continue;
        }
        let row = row as usize;
        let start = start as usize;
        let len = len as usize;
        if row >= rows || start >= cols {
            continue;
        }
        let end = (start + len).min(cols);
        for col in start..end {
            grid[row][col] = color;
        }
    }

    let mut row_groups = Vec::with_capacity(rows);
    for row in &grid {
        let mut groups = Vec::new();
        let mut col = 0;
        while col < cols {
            let z = row[col];
            let start = col;
            while col < cols && row[col] == z {
                col += 1;
            }
            let len = col - start;
            if z > 0 && len > 0 {
                groups.push(Group {
                    len,
                    color_id: z as usize,
                });
            }
        }
        row_groups.push(groups);
    }

    let mut col_groups = Vec::with_capacity(cols);
    for col in 0..cols {
        let mut groups = Vec::new();
        let mut row = 0;
        while row < rows {
            let z = grid[row][col];
            let start = row;
            while row < rows && grid[row][col] == z {
                row += 1;
            }
            let len = row - start;
            if z > 0 && len > 0 {
                groups.push(Group {
                    len,
                    color_id: z as usize,
                });
            }
        }
        col_groups.push(groups);
    }

    Ok(PuzzleData {
        color_panel,
        row_groups,
        col_groups,
    })
}
