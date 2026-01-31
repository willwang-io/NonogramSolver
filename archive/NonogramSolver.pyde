"""Solve a color Nonogram puzzle and display the solving progress step by step.

Note: the solving progress is supported by the Processing Python 2 and must
run with the Processing IDE.
"""

import math
import time

from copy import copy
from one_line_solver import OneLineSolver
from puzzle_reader import PuzzleReader
from Cell import Cell


class NonogramSolver:
    """Read and solve a colored Nonogram puzzle.

    The solving is step by step, just like a human solver. That is, each step
    can be derived from the previous step (except the first step). Furthermore,
    we can save each step and display a nice solving progress later.

    Args:
        puzzle_id: a string of integer, the puzzle you would like to solve.
    """
    def __init__(self, puzzle_id):
        reader = PuzzleReader(puzzle_id)

        self.row_groups = reader.get_row_groups()
        self.col_groups = reader.get_col_groups()
        self.color_panel = reader.get_color_panel()

        self.m = len(self.row_groups)
        self.n = len(self.col_groups)
        # A cell initially can be any colors from the color panel.
        self.row_masks = [
            [(1 << len(self.color_panel)) - 1] * self.n
            for _ in range(self.m)]
        self.col_masks = [
            [(1 << len(self.color_panel)) - 1] * self.m
            for _ in range(self.n)]

    def solve(self):
        """Solve the puzzle.

        Solve each row and column by deriving from their groups hints. Update
        and save the state of the puzzle after each line solve to avoid repeat
        computation for the next step.

        Returns:
            list of list of integers, each list of integers is one step.
        """
        dead_rows = [False] * self.m
        dead_cols = [False] * self.n
        ans = [[x[:] for x in self.row_masks]]
        line_solver = OneLineSolver(max(self.n, self.m))

        start_time = time.time()

        prev_sum = -1
        while True:
            if not self.update_state(line_solver, dead_rows, dead_cols):
                print("Unable to update further")
                return []
            cur_sum = self.update_cell_values()
            if cur_sum == prev_sum:
                print("Solving progress completed")
                break
            prev_sum = cur_sum
            ans.append([x[:] for x in self.row_masks])

        print('--- %s seconds ---' % (time.time() - start_time))
        return ans

    def update_state(self, solver, dead_rows, dead_cols):
        """Update the state of all the rows and columns.

        Args:
            solver: OneLineSolver object.
            dead_rows: list of booleans of size m
            dead_rows: list of booleans of size n

        Returns:
            True if successfully updated, False otherwise.
        """
        row_masks = copy(self.row_masks)
        col_masks = copy(self.col_masks)
        row_groups = copy(self.row_groups)
        col_groups = copy(self.col_groups)

        if not self.update_groups_state(
                solver,
                dead_rows,
                row_groups,
                row_masks):
            return False
        if not self.update_groups_state(
                solver,
                dead_cols,
                col_groups,
                col_masks):
            return False
        return True

    @staticmethod
    def update_groups_state(solver, dead, groups, masks):
        """Update the stage of groups after each line updated.

        Args:
            solver: OneLineSolver object.
            dead: list of booleans.
            groups: list of list of tuples.
            masks: list of list of integers.

        Returns:
            True if groups updated, False otherwise.
        """
        for i, group in enumerate(groups):
            if not dead[i]:
                if not solver.update_state(group, masks[i]):
                    return False
                is_dead = True
                for num in masks[i]:
                    # a line is solved if only one bit is set to 1.
                    if bin(num).count('1') != 1:
                        is_dead = False
                        break
                dead[i] = is_dead
        return True

    def update_cell_values(self):
        """Update cell values and returns its sum.

        Returns:
            an integer that is the total value of updated.
        """
        total = 0
        row_masks = copy(self.row_masks)
        col_masks = copy(self.col_masks)
        for row in range(self.m):
            for col in range(self.n):
                row_masks[row][col] &= col_masks[col][row]
                col_masks[col][row] &= row_masks[row][col]
                total += row_masks[row][col]
        return total


# The following are for Processing Python Mode. Processing is a very easy MVC
# solution. where global variables (bad practice, but this is how Processing
# work) is Model, draw() is View, and keyPressed() is Control.
nonogram_solver = NonogramSolver('19048')
solving_progress = nonogram_solver.solve()
m = len(solving_progress[0])
n = len(solving_progress[0][0])
CELL_WIDTH = 10
# pointer to the solving_progress. That is, the View will only display the
# the state where STEP current is pointing at.
STEP = 0
# Initially empty grid.
GRID = [[Cell(i, j, CELL_WIDTH) for i in range(n)] for j in range(m)]


def setup():
    """Setup Processing canvas"""
    size(n * CELL_WIDTH, m * CELL_WIDTH)


def draw():
    """Display the puzzle at current STEP"""
    background('#454545')
    for i in range(m):
        for j in range(n):
            # color of the cell at grid[i][j] at STEP has
            x = solving_progress[STEP][i][j]
            tmp = 1
            # if it is a power of two and not zero (white)
            if (x & (x - 1) == 0) and x != 0:
                # get the index of that bit
                tmp = int(math.log(x, 2)) + 1
            GRID[i][j].update_color(nonogram_solver.color_panel[tmp])
            GRID[i][j].show()


def keyPressed():
    """Go to the next step if pressed w and the previous step if pressed s"""
    global STEP
    if key == 'w':
        STEP = min(STEP + 1, len(solving_progress) - 1)
    if key == 's':
        STEP = max(STEP - 1, 0)
