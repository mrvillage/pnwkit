#!/usr/bin/env python3

# Taken from https://github.com/crate-ci/cargo-release/blob/19a59b29d8864d1ce1c239c40e563dda1f46a7e9/.github/workflows/release-notes.py
# See https://github.com/mrvillage/cargo-release-action for more info

import argparse
import pathlib
import re
import sys

_STDIO = pathlib.Path("-")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-i", "--input", type=pathlib.Path, default="CHANGELOG.md")
    parser.add_argument("--tag", required=True)
    parser.add_argument("-o", "--output", type=pathlib.Path, required=True)
    args = parser.parse_args()

    if args.input == _STDIO:
        lines = sys.stdin.readlines()
    else:
        with args.input.open() as fh:
            lines = fh.readlines()
    version = args.tag.lstrip("v")

    note_lines = []
    for line in lines:
        if line.startswith("## ") and version in line:
            note_lines.append(line)
        elif note_lines and line.startswith("## "):
            break
        elif note_lines:
            note_lines.append(line)

    notes = "".join(note_lines).strip()
    if args.output == _STDIO:
        print(notes)
    else:
        args.output.write_text(notes)


if __name__ == "__main__":
    main()
