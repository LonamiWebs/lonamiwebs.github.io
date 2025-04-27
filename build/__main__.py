import cProfile
import re
import shutil
import subprocess
import sys
import threading
from argparse import Namespace
from functools import partial
from pathlib import Path
from http.server import HTTPServer, SimpleHTTPRequestHandler
import time

from .args import parse_args
from .conf import CNAME, INPUT_PATH, OUTPUT_PATH, TEMPLATE_PATH
from .lexer import lex
from .minifier import minify_css, minify_html
from .preprocessor import preprocess, template_replacer
from .generator import generate
from .watch import FileAction, watch


cached_template: bytes | None = None


def load_template() -> bytes:
    global cached_template
    if cached_template is None:
        with TEMPLATE_PATH.open("rb") as fd:
            return (cached_template := minify_html(fd.read()))

    return cached_template


def process_file(f: Path) -> tuple[Path, bytes]:
    with f.open("rb") as fd:
        content = fd.read()

    if f.suffix == ".md":
        content = re.sub(
            rb"\$(\w+)\b",
            template_replacer(f, minify_html(generate(*preprocess(*lex(content))))),
            load_template(),
        )
        f = f.with_suffix(".html")
    elif f.suffix == ".css":
        content = minify_css(content)
    elif f.suffix == ".html":
        content = minify_html(content)
    else:
        content = content

    return OUTPUT_PATH / f.relative_to(INPUT_PATH), content


def commit_file(f: Path, content: bytes):
    f.parent.mkdir(parents=True, exist_ok=True)
    with f.open("wb") as fd:
        fd.write(content)


def main(args: Namespace):
    generated = {Path(OUTPUT_PATH / "CNAME"): CNAME.encode()}

    for root, dirs, files in INPUT_PATH.walk():
        for f in files:
            f = root / f
            if f == TEMPLATE_PATH:
                continue

            try:
                path, content = process_file(f)
                generated[path] = content
            except ValueError as e:
                print(f"failed to process file: {f}\n  {e}", file=sys.stderr)
                if args.ignore_errors:
                    continue
                sys.exit(1)

    if args.write:
        if args.force:
            shutil.rmtree(OUTPUT_PATH, ignore_errors=True)

        for f, content in generated.items():
            commit_file(f, content)


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


def serve(args: Namespace):
    thread: threading.Thread | None = None
    done = threading.Event()
    if args.watch:

        def watch_thread():
            for action, f in watch(INPUT_PATH, until=done):
                if action in (
                    FileAction.ADDED,
                    FileAction.MODIFIED,
                    FileAction.RENAMED_NEW_NAME,
                ):
                    try:
                        start = time.time()
                        commit_file(*process_file(f))
                        end = time.time()
                        print(f"regenerated {f} in {end - start:.3f}s", file=sys.stderr)
                    except ValueError as e:
                        print(f"failed to process file: {f}\n  {e}", file=sys.stderr)

        thread = threading.Thread(target=watch_thread)
        thread.start()

    with HTTPServer(
        ("", 8000), partial(SimpleHTTPRequestHandler, directory=OUTPUT_PATH)
    ) as httpd:
        host, port = httpd.socket.getsockname()[:2]
        url_host = f"[{host}]" if ":" in host else host
        print(f"http://{url_host}:{port}/")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            pass
        finally:
            done.set()
            if thread:
                thread.join()


if __name__ == "__main__":
    args = parse_args(main=main, test=test, serve=serve)
    if args.profile:
        cProfile.run("args.fn(args)", sort="cumtime")
    else:
        args.fn(args)
