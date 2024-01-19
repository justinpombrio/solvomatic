# Display Sudoku boards nicely

import sys

assert len(sys.argv) == 3

NUMBERS=None
if sys.argv[2] == "number":
    NUMBERS=True
elif sys.argv[2] == "letter":
    NUMBERS=False
else:
    raise Exception("arg 2 say what to print: 'number' or 'letter'")

for i, line in enumerate(open(sys.argv[1], 'r')):
    if i % 3 == 0:
        print("|  +-------+-------+-------+")
    print("|  |", end="")
    for block in [0, 3, 6]:
        print(" ", end="")
        for column in [0, 1, 2]:
            ch = line[block + column]
            if ch == ".":
                print(".", end="")
            elif NUMBERS:
                print(ch, end="")
            else:
                print("abcdefghij"[int(ch)], end="")
            print(" ", end="")
        print("|", end="")
    print()
print("|  +-------+-------+-------+")
