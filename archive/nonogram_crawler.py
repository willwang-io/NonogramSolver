#!/usr/bin/env python
# -*- coding: utf-8 -*-

import sys
from bs4 import BeautifulSoup
from selenium import webdriver

"""Web-crawl Nonogram puzzles from https://www.nonograms.org/

Collect the information of a color Nonogram puzzle from a website and
save as a text file for later use. The information are: the color panel that
include all the colors the puzzle will use, the row groups, and the column
groups.

Abstractly, the website store the puzzle in the following manner:
If a puzzle with size m * n, then row groups is in a m * max(# of groups
on a row) grid. Column groups is in a n * max(# of groups on a column)
grid. Color panel is just a list of size L.

Typical usage example:
    python nonogram_crawler.py <PUZZLE_ID>

TODO:
    Crawl black-white puzzle.
"""


def crawl_color_panel(soup: BeautifulSoup) -> dict:
    """ Fetches the color panel of the puzzle.

    Args:
        soup: a Beautiful Soup object in html.parser.
    Returns:
        a dictionary, where key is hexadecimal color codeand value is the ID
        of the color.
    """
    color_panel_ele = soup.find('table', {'class': 'nonogram_color_table'})
    colors = color_panel_ele.findAll('td')
    color_panel = {'#ffffff': 0}  # initial with the white color
    for i, color in enumerate(colors, start=1):
        color_panel[color['style'].split(';')[0].split(':')[1]] = i
    return color_panel


def crawl_row_groups(soup: BeautifulSoup, color_panel: dict) -> list:
    """ Fetches the row groups of the puzzle.

    Args:
        soup: a Beautiful Soup object in html.parser.
        color_panel: a dictionary generates by crawl_color_panel().

    Returns:
        a list of tuples (block_length, color_id).
    """
    rows_ele = soup.find('td', {'class': 'nmtl'}).findAll('tr')
    row_groups = []
    for row in rows_ele:
        cur_row = []
        for cell in row.findAll('td'):
            # empty cell
            if not cell.has_attr('style'):
                continue
            cell_color = cell['style'].split(';')[0].split(':')[1]
            cur_row.append((int(cell.text), color_panel[cell_color]))
        row_groups.append(cur_row)
    return row_groups


def crawl_col_groups(soup: BeautifulSoup, color_panel: dict) -> list:
    """ Fetches the column groups of the puzzle.

    Args:
        soup: a Beautiful Soup object in html.parser.
        color_panel: a dictionary generates by crawl_color_panel().

    Returns:
        a list of tuples (block_length, color_id).
    """
    cols_ele = soup.find('td', {'class': 'nmtt'}).findAll('tr')
    temp_col_groups = []
    for row in cols_ele:
        cur_row = []
        for cell in row.findAll('td'):
            # empty cell, place a dummy value first
            if not cell.has_attr('style'):
                cur_row.append((-1, 0))
            else:
                cell_color = cell['style'].split(';')[0].split(':')[1]
                cur_row.append((int(cell.text), color_panel[cell_color]))
        temp_col_groups.append(cur_row)
    # we collect column groups like row groups first, from left to right and
    # top to bottom. However, the actual column group is actually the transpose
    # of the temp_col_groups.
    return [[(i, j) for i, j in col if i * j != 0]
            for col in zip(*temp_col_groups)]


def main():
    """Main function.

    Save the color panel, row groups, and column groups into a text file, in
    this order and separated by a '-' symbol. Name it as <COLOR_ID>.txt under
    ./puzzle/ directory.
    """
    argv = sys.argv
    puzzle_id = argv[1]

    opt = webdriver.ChromeOptions()
    opt.add_argument('headless')

    driver = webdriver.Chrome(options=opt)
    puzzle_url = 'https://www.nonograms.org/nonograms2/i/'
    driver.get(puzzle_url + puzzle_id)

    driver.get(puzzle_url + puzzle_id)
    soup = BeautifulSoup(driver.page_source, 'html.parser')

    color_panel = crawl_color_panel(soup)
    row_groups = crawl_row_groups(soup, color_panel)
    col_groups = crawl_col_groups(soup, color_panel)

    with open(f'./puzzle/{puzzle_id}.txt', 'w', encoding='UTF-8') as file:
        for k in color_panel:
            file.write(f'{k}\n')
        file.write('-\n')

        for row_group in row_groups:
            for color_id, size in row_group:
                file.write(f'{color_id}:{size},')
            file.write('\n')

        file.write('-\n')

        for col_group in col_groups:
            for color_id, size in col_group:
                file.write(f'{color_id}:{size},')
            file.write('\n')


if __name__ == '__main__':
    main()
