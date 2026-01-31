#!/usr/bin/env python
# -*- coding: utf-8 -*-

"""Update a single row or column (here claled line) of the puzzle by given
groups description and current state. Each cell is represented by an integer.
It potentially has color i if the i-th bit is set to 1.

Example:
    solver = OneLineSolver(10)
"""


class OneLineSolver:
    """Use top-down dynamic programming (DP) techniques to solve one line of
    the Nonogram puzzle. Initlaize the memorization lists for DP.

    Args:
        line_len: the size of the line.
    """
    def __init__(self, line_len):
        # save the last update_state() call count, when the [X][Y] element
        # was calculated, and protects from recalculations
        self.cache = [[0] * (line_len + 1) for _ in range(line_len + 1)]
        # [X][Y] memorize if it's possible to reach the end of the puzzle,
        # if we have placed X groups and currently on the Y-th cell.
        self.calc_fill = [[0] * (line_len + 1) for _ in range(line_len + 1)]
        # manage recalculations, increments when update_state() is called
        self.cache_cnt = 0
        # save intermediate results (bit-masks) before updating the cells list
        self.result_cell = [0] * line_len

    def update_state(self, groups, cells):
        """Update the state of a line. Return false if the puzzle is unsolvable
        or has wrong state, otherwise return true.

        Args:
            groups: list of tuples, the groups description of the line.
            cells: list of integers, the state of the line.

        Returns:
            False if the puzzle is unsolvable or finished. Otherwise, True.
        """
        self.cache_cnt += 1
        self.result_cell = [0] * len(cells)
        if not self.can_fill(groups, cells):
            return False
        for i, _ in enumerate(cells):
            cells[i] = self.result_cell[i]
        return True

    @staticmethod
    def can_place_color(cells, clr, l_bound, r_bound):
        """Determines the possibility of filling the cells in an intervals,
        inclusive with a certain color.

        Args:
            cells: list of integers, the state of the line
            clr: color
            l_bound: integer, left bound of the interval
            r_bound: integer, right bound of the interval

        Returns:
            True if possible, False otherwise.
        """
        if r_bound >= len(cells):
            return False
        mask = 1 << clr
        # Paint a block of cells with a certain color iff it is possible for
        # all cells to have this color (every cell from the block has color-th
        # bit set to 1).
        for i in range(l_bound, r_bound + 1):
            if (cells[i] & mask) == 0:
                return False
        return True

    def set_place_color(self, clr, l_bound, r_bound):
        """Filling the cells in an intervals , inclusive with a certain color.

        Args:
            clr: color
            l_bound: integer, left bound of the interval
            r_bound: integer, right boun dof the interval

        Returns:
            None
        """
        for i in range(l_bound, r_bound + 1):
            self.result_cell[i] |= (1 << clr)

    def can_fill(self, groups, cells, cur_group=0, cur_cell=0):
        """Check if it's possible to reach the end of the puzzle, if we have
        placed cur_group from groups and currenlty are on the cur_cell from
        cells.

        Args:
            groups: a list of tuples, the group description of the line
            cells: a list of integer, state of the line.
            cur_group: integer, index of the current group we are looking
            cur_cell: integer, index of the current cell we are looking

        Returns:
            0 (False) if we cannot fill, 1 (True) otherwise
        """
        # at the end of the puzzle, all the groups should have been placed
        if cur_cell == len(cells):
            return cur_group == len(groups)
        if self.cache[cur_group][cur_cell] == self.cache_cnt:
            return self.calc_fill[cur_group][cur_cell]
        answer = 0
        # try to place a white cell
        if self.can_place_color(cells, 0, cur_cell, cur_cell) and \
                self.can_fill(groups, cells, cur_group, cur_cell + 1):
            self.set_place_color(0, cur_cell, cur_cell)  # fill white
            answer = 1
        # try to place current-group-color cells
        if cur_group < len(groups):
            cur_color = groups[cur_group][1]
            l_bound = cur_cell
            r_bound = cur_cell + groups[cur_group][0] - 1

            can_place = self.can_place_color(cells, cur_color, l_bound, r_bound)
            # it may be required to place a white cell after current group
            place_white = False

            next_cell = r_bound + 1
            # check whether we are to put a white cell after the group
            if can_place:
                # same color group should sepaprate by a white cell
                if cur_group + 1 < len(groups) and \
                        groups[cur_group + 1][1] == cur_color:
                    place_white = True
                    can_place = self.can_place_color(cells,
                                                     0,
                                                     next_cell,
                                                     next_cell)
                    next_cell += 1
            if can_place:
                # remember this if after placement the puzzle can be solved
                if self.can_fill(groups, cells, cur_group + 1, next_cell):
                    answer = 1
                    self.set_place_color(cur_color, l_bound, r_bound)
                    if place_white:
                        self.set_place_color(0, r_bound + 1, r_bound + 1)
        # memorization
        self.calc_fill[cur_group][cur_cell] = answer
        self.cache[cur_group][cur_cell] = self.cache_cnt
        return answer
