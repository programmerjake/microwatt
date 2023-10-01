#!/usr/bin/python3

import sys

with open(sys.argv[1], "rb") as f:
    while True:
        word = f.read(8)
        if len(word) == 0:
            exit(0)
        print("%016x" % int.from_bytes(word, 'little'))
