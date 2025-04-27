"""
Command-line parameter definitions and argument parsing.
"""

import argparse
from typing import Callable, TypeAlias


Fn: TypeAlias = Callable[[argparse.Namespace], None]


def parse_args(*, main: Fn, test: Fn):
    parser = argparse.ArgumentParser()
    parser.prog = "python -m build"
    parser.set_defaults(fn=main)
    parser.add_argument(
        "-w",
        "--write",
        help="write output instead of simply performing a dry run",
        action="store_true",
    )
    parser.add_argument(
        "-f",
        "--force",
        help="delete output folder before writing",
        action="store_true",
    )
    parser.add_argument(
        "--ignore-errors",
        help="ignore errors during input processing",
        action="store_true",
    )
    parser.add_argument(
        "--profile",
        help="profile execution",
        action="store_true",
    )

    subparsers = parser.add_subparsers()

    parser_test = subparsers.add_parser("test", help="alias to run unittest")
    parser_test.set_defaults(fn=test)

    return parser.parse_args()
