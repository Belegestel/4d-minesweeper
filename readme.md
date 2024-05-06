# 4D Minesweeper
To start things off, code is not publicly available right now. It still needs to be tidied up. It also tends to freeze on autospreading with larger boards.
The game itself is available on [this website](https://belegestel.github.io/4d-minesweeper/index)
## How to play
The rules are exactly the same as in standard 2D minesweeper, except each cell has not 8, but 80 neighbors. To make it easier for the player, adjacent cells should be automatically highlighted on mouse hover. The player wins when all cells without mines are opened.
### Chording 
It is a technique available in both the standard 2D minesweeper, as well as in my 4D variant. If a cell is touching exactly as many flags, as the number inside of the cell, *chord* can be performed. It's done by clicking the cell with the number - it automatically reveals all neighboring non-flagged cells. It's a QOL feature, as it doesn't alter the rules. It makes life much easier, though.

---

There are a few settings:
1. `Click to flag` swaps primary and secondary mouse buttons. Instead of opening with left and flagging with right, it makes it the opposite.
2. `Dimensions` and `Number of mines` can be adjusted with the fields. Note, that the new board won't be generated until you click the `Restart` button.
3. `Automatic spreading` automatically opens all neighbors of cells with 0 adjacent mines. Not recommended for larger boards, at least in the current state of the game.
4. `Cell size` changes the visual size of the cells. Has no impact on the rules, but it makes playing on some mobile devices easier.
