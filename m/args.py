"""
Command-line parameter definitions and argument parsing.
"""

import argparse
from typing import Callable, TypeAlias


Fn: TypeAlias = Callable[[argparse.Namespace], None]


def parse_args(*, build: Fn, test: Fn, serve: Fn):
    parser = argparse.ArgumentParser()
    parser.prog = "python -mm"

    subparsers = parser.add_subparsers()
    subparsers.required = True

    parser_build = subparsers.add_parser("build", help="build the site")
    parser_build.set_defaults(fn=build)
    parser_build.add_argument(
        "-w",
        "--write",
        help="write output instead of simply performing a dry run",
        action="store_true",
    )
    parser_build.add_argument(
        "-f",
        "--force",
        help="delete output folder before writing",
        action="store_true",
    )
    parser_build.add_argument(
        "--ignore-errors",
        help="ignore errors during input processing",
        action="store_true",
    )
    parser_build.add_argument(
        "--profile",
        help="profile execution",
        action="store_true",
    )

    parser_test = subparsers.add_parser("test", help="alias to run unittest")
    parser_test.set_defaults(fn=test)

    parser_serve = subparsers.add_parser("serve", help="alias to run http.server")
    parser_serve.set_defaults(fn=serve)
    parser_serve.add_argument(
        "-w",
        "--watch",
        help="watch for changes and automatically regenerate files",
        action="store_true",
    )

    return parser.parse_args()
