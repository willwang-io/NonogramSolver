#!/usr/bin/env python
# -*- coding: utf-8 -*-

"""Get the information of a puzzle from an input file.

The file consists of three parts that separated by '-'. The first part is the
colors the puzzle need, and the second and third part are the row groups
and column groups.

    Example:

    reader = PuzzleReader('19048')
"""


class PuzzleReader:
    """Unparse a text file of a puzzle.

    Args:
        puzzle_id: a string of integers.

    Attributes:
        get_color_panel(), get_row_groups(), and get_col_groups() are
        self-explanatory by their name.
    """
    def __init__(self, puzzle_id):
        linux_path = './puzzle/'
        with open(linux_path + puzzle_id + '.txt') as file:
            self.parts = file.read().split('-')

    def get_color_panel(self):
        """Get all the colors the puzzle needs.

        Returns:
            a dictionary, where the key is an integer and the value is the
            hex color code.
        """
        color_panel = {}
        color_token = self.parts[0].strip().split('\n')
        for i, clr in enumerate(color_token, start=1):
            color_panel[i] = clr
        return color_panel

    def get_row_groups(self):
        """Get the row groups of the puzzle.

        Returns:
            value returns by calling get_groups(row_groups_token)
        """
        return self._get_groups(self.parts[1])

    def get_col_groups(self):
        """Get the column groups of the puzzle.

        Returns:
            value returns by calling get_groups
        """
        return self._get_groups(self.parts[2])

    @staticmethod
    def _get_groups(token):
        """Helper function to parse row/column groups.

        Row and column groups in the input file have same representation.

        Args:
            token: a string of either row groups or column groups

        Returns:
            a list of integer tuples (S, C), represent the group has sizs S
            with color ID C.
        """
        ans = []
        for row in token.strip().split('\n'):
            cur = []
            for group in row.strip(',').split(','):
                size, clr = map(int, group.split(':'))
                cur.append((size, clr))
            ans.append(cur)
        return ans
