"""A cell represents by a Processing rectangle shape.

Easily control the position and color of a cell.
"""
class Cell:
    """A cell has a position, size, and color

    Args:
        x: x-coordinate of the top left corner.
        y: y-coordinate of the top right corner.
        w: width of the cell.
        clr: color of the cell, initially white.
    """
    def __init__(self, x, y, w, clr="#FFFFFF"):
        self.clr = clr
        self.x = x
        self.y = y
        self.w = w

    def show(self):
        """Display the cell to View"""
        fill(self.clr)
        noStroke()
        rect(self.x * self.w, self.y * self.w, self.w, self.w)

    def update_color(self, clr):
        """Mutator of self.clr"""
        self.clr = clr
