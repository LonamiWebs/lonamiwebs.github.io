"""
Formatting pre-processor.
"""

import inspect
import itertools
import re
from typing import Self, Iterator
from pathlib import Path

from .conf import INPUT_PATH
from .lexer import lex
from .parser import parse_toml_ish
from .types import (
    Emphasis,
    Format,
    Fence,
    Group,
    Heading,
    Item,
    Metadata,
    Raw,
    Reference,
    Row,
    Separator,
)


class Entry:
    def __init__(self, path: Path) -> None:
        self.path = path
        self.meta: dict[bytes, list[bytes]] | None = None
        self.first_title: bytes | None = None

    def __iter__(self) -> Iterator[Self]:
        return itertools.chain(
            map(self.__class__, self.path.glob("*.md")),
            map(self.__class__, self.path.glob("*/index.md")),
        )

    def __getitem__(self, name: str) -> Self:
        return self.__class__(self.path / name)

    @property
    def permalink(self) -> str:
        f = self.path.relative_to(INPUT_PATH).with_suffix("").as_posix()
        return f"/{f}"

    @property
    def title(self) -> str:
        if meta := self.load_meta():
            if meta.get(b"title"):
                return meta[b"title"][0].decode()

        if self.first_title:
            return self.first_title.decode("utf-8")

        return self.path.name

    @property
    def date(self) -> str:
        if meta := self.load_meta():
            if meta.get(b"date"):
                return meta[b"date"][0].decode()

        return ""

    @property
    def category(self) -> str:
        if meta := self.load_meta():
            if meta.get(b"category"):
                return meta[b"category"][0].decode()

        return ""

    @property
    def tags(self) -> list[str]:
        if meta := self.load_meta():
            if meta.get(b"tags"):
                return [t.decode() for t in meta[b"tags"]]

        return []

    def load_meta(self) -> dict[bytes, list[bytes]] | None:
        if not self.meta and self.path.suffix == ".md":
            with self.path.open("rb") as fd:
                content = fd.read()
            text, formats = lex(content)
            for _, format in formats:
                if isinstance(format, Metadata):
                    self.meta = parse_toml_ish(format.content)
                    break

            heading_start: int | None = None
            for i, format in formats:
                if isinstance(format, Heading):
                    if heading_start is None:
                        heading_start = i
                    else:
                        self.first_title = text[heading_start:i]
                        break

        return self.meta


def preprocess(
    text: bytes, formats: list[tuple[int, Format]]
) -> tuple[bytes, list[tuple[int, Format]]]:
    context = {"content": Entry(Path("content"))}

    new_formats: list[tuple[int, Format]] = []

    list_item_open = False
    list_group = False

    table_group = False

    reference_open = False
    reference_footnote = False
    reference_footnotes: set[bytes] = set()

    skip_n = 0

    for i, (p, f) in enumerate(formats):
        if skip_n:
            skip_n -= 1
            continue

        if isinstance(f, Emphasis):
            np, nf = formats[i + 1] if (i + 1) < len(formats) else (-1, None)
            if isinstance(nf, Emphasis):
                new_formats.append((p, f))
                new_formats.append((np, nf))
                skip_n = 1
            else:
                new_formats.append((p, Raw(content=b"*" * f.strength)))

        elif isinstance(f, Reference):
            reference_open = not reference_open
            if reference_open:
                np, nf = next(
                    pf for pf in formats[i + 1 :] if isinstance(pf[1], Reference)
                )
                if not f.uri and text[p : p + 1].startswith(b"^"):
                    ref_text = text[p + 1 : np]
                    fn = (
                        lambda h, r: (b"#" if h else b"")
                        + b"fn"
                        + (b"ref" if r else b"")
                        + b":"
                        + ref_text
                    )
                    if ref_text in reference_footnotes:
                        reference_footnotes.discard(ref_text)
                        # TODO remove the text... somehow, yikes..!
                        new_formats.append(
                            (
                                p,
                                Reference(uri=fn(True, False), id=fn(False, True)),
                            )
                        )
                        new_formats.append(
                            (p, Raw(content=b"<sup>" + ref_text + b"</sup>"))
                        )
                        new_formats.append(
                            (
                                p,
                                Reference(uri=fn(True, False), id=fn(False, True)),
                            )
                        )
                    else:
                        reference_footnotes.add(ref_text)
                        # TODO remove the text... somehow, yikes..!
                        new_formats.append(
                            (p, Reference(uri=fn(True, True), id=fn(False, False)))
                        )
                        new_formats.append(
                            (p, Raw(content=b"<sup>" + ref_text + b"</sup>"))
                        )
                        new_formats.append(
                            (
                                p,
                                Reference(uri=fn(True, True), id=fn(False, False)),
                            )
                        )
                else:
                    new_formats.append((p, f))
                    new_formats.append((np, nf))

        elif isinstance(f, Item):
            g = Group(type=b"ordered-list" if f.ordered else b"unordered-list")
            np, nf = next(
                (pf for pf in formats[i + 1 :] if isinstance(pf[1], Item)), (-1, None)
            )
            list_item_open = not list_item_open
            if list_group:
                if list_item_open or (p + 1 == np):
                    new_formats.append((p, f))
                else:
                    new_formats.append((p, f))
                    new_formats.append(
                        (
                            p,
                            g,
                        )
                    )
                    list_group = False
            else:
                list_group = True
                new_formats.append((p, g))
                new_formats.append((p, f))

        elif isinstance(f, Row):
            np, nf = formats[i + 1] if (i + 1) < len(formats) else (-1, None)
            if table_group:
                if isinstance(nf, Row) and text[p:np] == b"\n":
                    new_formats.append((p, f))
                else:
                    new_formats.append((p, f))
                    new_formats.append((p, Group(type=b"table")))
                    table_group = False
            else:
                table_group = True
                new_formats.append((p, Group(type=b"table")))
                new_formats.append((p, f))

        elif isinstance(f, Separator):
            if (
                m := re.compile(rb"(^|\n)[^\n]+\n$").search(text, 0, endpos=p)
            ) and p == m.end():
                uf = Heading(level=1 if f.style == b"=" else 2)
                new_formats.append((m.start(), uf))
                new_formats.append((m.end() - 1, uf))
            else:
                # this feels awfully similar to a second lex step, specially if we keep text..
                new_formats.append((p, f))

        elif isinstance(f, Fence) and b",inject" in f.type:
            g = {}
            exec(f.content, g)
            i = g["inject"]
            kwargs = {}
            for param in inspect.signature(i).parameters:
                try:
                    kwargs[param] = context[param]
                except KeyError:
                    raise ValueError(
                        f"cannot inject value for unknown parameter name: {param}"
                    )

            new_formats.append(
                (p, Raw(content=b"\n".join(map(str.encode, i(**kwargs)))))
            )
        else:
            new_formats.append((p, f))

    return text, new_formats


def template_replacer(path: Path, content: bytes):
    def replacer(match: re.Match[bytes]) -> bytes:
        if match[1] == b"TITLE":
            if path.parts[0] == "index.md":
                return b"Lonami's Site"
            elif path.parts[0] == "blog.md":
                return b"Lonami's Blog"
            elif path.parts[0] == "golb.md":
                return b"Lonami's Golb"
            elif path.parts[0] in ("blog", "golb"):
                return f"{Entry(INPUT_PATH / path).title} | Lonami's Blog".encode()
            else:
                raise RuntimeError(
                    f"unknown path to use for replacing template title: {path}"
                )
        elif match[1] == b"CONTENT":
            return content
        elif match[1] == b"ROOT":
            return b"class=selected" if path.parts[0] == "index.md" else b""
        elif match[1] == b"BLOG":
            return b"class=selected" if path.parts[0] in ("blog.md", "blog") else b""
        elif match[1] == b"GOLB":
            return b"class=selected" if path.parts[0] in ("golb.md", "golb") else b""
        else:
            raise RuntimeError(f"unknown template variable: {match[1]}")

    return replacer
