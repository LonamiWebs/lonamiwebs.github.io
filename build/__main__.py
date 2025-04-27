import cProfile
import re
import shutil
import subprocess
import sys
from argparse import Namespace
from pathlib import Path

from .args import parse_args
from .conf import CNAME
from .lexer import lex
from .minifier import minify_css, minify_html
from .preprocessor import preprocess, template_replacer
from .generator import generate


def main(args: Namespace):
    input = Path("content")
    output = Path("www")

    generated = {"CNAME": CNAME.encode()}

    template_path = Path(input / "base.template.html")
    with template_path.open("rb") as fd:
        template_content = minify_html(fd.read())

    for root, dirs, files in input.walk():
        for f in files:
            if f == template_path.name:
                continue

            f = root / f
            with f.open("rb") as fd:
                content = fd.read()
            f = f.relative_to(input)

            if f.suffix == ".md":
                try:
                    html = minify_html(generate(*preprocess(*lex(content))))
                except ValueError as e:
                    print(
                        f"failed to convert markdown file to html: {f}\n  {e}",
                        file=sys.stderr,
                    )
                    if args.ignore_errors:
                        continue
                    sys.exit(1)

                generated[f.with_suffix(".html").as_posix()] = re.sub(
                    rb"\$(\w+)\b", template_replacer(f, html), template_content
                )
            elif f.suffix == ".css":
                generated[f.as_posix()] = minify_css(content)
            elif f.suffix == ".html":
                generated[f.as_posix()] = minify_html(content)
            else:
                generated[f.as_posix()] = content

    if args.write:
        if args.force:
            shutil.rmtree(output, ignore_errors=True)

        for f, content in generated.items():
            f = output / f
            f.parent.mkdir(parents=True, exist_ok=True)
            with f.open("wb") as fd:
                fd.write(content)


def test(_: Namespace):
    ret = subprocess.run(
        (
            sys.executable,
            "-m",
            "unittest",
            "discover",
            "--top-level-directory",
            ".",
            "--start-directory",
            "build/tests",
        )
    )
    exit(ret.returncode)


if __name__ == "__main__":
    args = parse_args(main=main, test=test)
    if args.profile:
        cProfile.run("args.fn(args)", sort="cumtime")
    else:
        args.fn(args)
