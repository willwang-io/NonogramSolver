//! One-line (row or column) solver for colored Nonogram puzzles.
//!
//! Cells store a bitmask of possible colors; bit 0 is reserved for white.

#[derive(Debug, Clone)]
pub struct OneLineSolver {
    /// Cache marker for memoized states; value is the last update counter.
    cache: Vec<Vec<u64>>,
    /// Memoized solvability for (group, cell) within the current update.
    calc_fill: Vec<Vec<bool>>,
    /// Monotonic counter to invalidate `cache` without clearing.
    cache_cnt: u64,
    /// Union of colors that are possible for each cell after solving.
    result_cell: Vec<u64>,
}

impl OneLineSolver {
    /// Create a solver sized for `line_len` cells.
    pub fn new(line_len: usize) -> Self {
        let size = line_len + 1;
        Self {
            cache: vec![vec![0; size]; size],
            calc_fill: vec![vec![false; size]; size],
            cache_cnt: 0,
            result_cell: vec![0; line_len],
        }
    }

    /// Update the state of a line in-place.
    ///
    /// `groups` is a list of `(length, color_index)` pairs.
    /// `cells` contains bitmasks of possible colors for each position.
    /// Returns `false` if no valid filling exists for the given constraints.
    pub fn update_state(&mut self, groups: &[(usize, usize)], cells: &mut [u64]) -> bool {
        self.ensure_capacity(cells.len(), groups.len());

        self.cache_cnt = self.cache_cnt.wrapping_add(1);
        if self.cache_cnt == 0 {
            // Overflow: clear cache markers and restart the counter.
            for row in &mut self.cache {
                for entry in row {
                    *entry = 0;
                }
            }
            self.cache_cnt = 1;
        }

        self.result_cell.resize(cells.len(), 0);
        for cell in &mut self.result_cell {
            *cell = 0;
        }

        if !self.can_fill(groups, cells, 0, 0) {
            return false;
        }

        cells.copy_from_slice(&self.result_cell[..cells.len()]);
        true
    }

    fn ensure_capacity(&mut self, line_len: usize, group_len: usize) {
        let needed = line_len.max(group_len) + 1;
        if self.cache.len() < needed {
            self.cache = vec![vec![0; needed]; needed];
            self.calc_fill = vec![vec![false; needed]; needed];
            self.cache_cnt = 0;
        }
    }

    fn color_mask(color: usize) -> Option<u64> {
        1u64.checked_shl(color as u32)
    }

    fn can_place_color(cells: &[u64], color: usize, l_bound: usize, r_bound: usize) -> bool {
        if r_bound >= cells.len() {
            return false;
        }
        let Some(mask) = Self::color_mask(color) else {
            return false;
        };
        for idx in l_bound..=r_bound {
            if (cells[idx] & mask) == 0 {
                return false;
            }
        }
        true
    }

    fn set_place_color(&mut self, color: usize, l_bound: usize, r_bound: usize) {
        let Some(mask) = Self::color_mask(color) else {
            return;
        };
        for idx in l_bound..=r_bound {
            self.result_cell[idx] |= mask;
        }
    }

    fn can_fill(
        &mut self,
        groups: &[(usize, usize)],
        cells: &[u64],
        cur_group: usize,
        cur_cell: usize,
    ) -> bool {
        if cur_cell == cells.len() {
            return cur_group == groups.len();
        }
        if self.cache[cur_group][cur_cell] == self.cache_cnt {
            return self.calc_fill[cur_group][cur_cell];
        }

        let mut answer = false;

        if Self::can_place_color(cells, 0, cur_cell, cur_cell)
            && self.can_fill(groups, cells, cur_group, cur_cell + 1)
        {
            self.set_place_color(0, cur_cell, cur_cell);
            answer = true;
        }

        if cur_group < groups.len() {
            let (group_len, cur_color) = groups[cur_group];
            if group_len > 0 {
                let Some(end_exclusive) = cur_cell.checked_add(group_len) else {
                    self.calc_fill[cur_group][cur_cell] = answer;
                    self.cache[cur_group][cur_cell] = self.cache_cnt;
                    return answer;
                };
                let r_bound = end_exclusive - 1;
                let l_bound = cur_cell;

                let mut can_place = Self::can_place_color(cells, cur_color, l_bound, r_bound);
                let mut place_white = false;
                let mut next_cell = r_bound + 1;

                if can_place && cur_group + 1 < groups.len() && groups[cur_group + 1].1 == cur_color
                {
                    // Same-color groups must be separated by a white cell.
                    place_white = true;
                    can_place = Self::can_place_color(cells, 0, next_cell, next_cell);
                    next_cell += 1;
                }

                if can_place && self.can_fill(groups, cells, cur_group + 1, next_cell) {
                    answer = true;
                    self.set_place_color(cur_color, l_bound, r_bound);
                    if place_white {
                        self.set_place_color(0, r_bound + 1, r_bound + 1);
                    }
                }
            }
        }

        self.calc_fill[cur_group][cur_cell] = answer;
        self.cache[cur_group][cur_cell] = self.cache_cnt;
        answer
    }
}

#[cfg(test)]
mod tests {
    use super::OneLineSolver;

    #[test]
    fn fills_when_group_is_forced() {
        let mut solver = OneLineSolver::new(2);
        let groups = vec![(2, 1)];
        let mut cells = vec![(1u64 << 0) | (1u64 << 1); 2];

        assert!(solver.update_state(&groups, &mut cells));
        assert_eq!(cells, vec![1u64 << 1, 1u64 << 1]);
    }

    #[test]
    fn keeps_union_of_options() {
        let mut solver = OneLineSolver::new(3);
        let groups = vec![(1, 1)];
        let mut cells = vec![(1u64 << 0) | (1u64 << 1); 3];

        assert!(solver.update_state(&groups, &mut cells));
        assert_eq!(cells, vec![(1u64 << 0) | (1u64 << 1); 3]);
    }
}
