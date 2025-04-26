"""
Bespoke Markdown-esque lexer.
"""

import re
import sys

from .types import (
    Break,
    Code,
    Format,
    Metadata,
    Emphasis,
    Reference,
    Heading,
    Item,
    Quote,
    Fence,
    Row,
    Separator,
)


class Lexer:
    def __init__(self, data: bytes) -> None:
        self.data = data
        self.len = len(data)
        self.pos = -1
        self.skips: list[tuple[range, Format | None]] = []
        self.kept = bytearray()
        self.formats: list[tuple[int, Format]] = []

    def next(self) -> bool:
        self.pos += 1

        while self.skips:
            for skip, _ in self.skips:
                if self.pos in skip:
                    self.pos = skip.stop
                    break
            else:
                for t in list(self.skips):
                    skip, format = t
                    if skip.stop <= self.pos:
                        self.skips.remove(t)
                        if format:
                            self.format(format)
                break

        return self.pos < self.len

    def try_match(self, regexp: bytes) -> re.Match[bytes] | None:
        return re.compile(regexp).match(self.data, self.pos)

    def line_col_at(self, pos: int) -> tuple[int, int]:
        l = 1
        c = 0
        nl = b"\n"[0]
        for i in range(pos):
            if self.data[i] == nl:
                l += 1
                c = 0
            else:
                c += 1
        return l, c

    def find_at(
        self, pos: int, regexp: bytes, *, endpos: int = sys.maxsize
    ) -> re.Match[bytes]:
        m = self.try_find_at(pos, regexp, endpos=endpos)
        if not m:
            l, c = self.line_col_at(pos)
            raise ValueError(
                f"expected to find regexp after {l}:{c}: {regexp.decode()}"
            )
        return m

    def try_find_at(
        self, pos: int, regexp: bytes, *, endpos: int = sys.maxsize
    ) -> re.Match[bytes] | None:
        return re.compile(regexp).search(self.data, pos, endpos)

    @property
    def cur(self) -> bytes:
        return self.byte(self.pos)

    @property
    def prev(self) -> bytes:
        return self.byte(self.pos - 1)

    def skip(self, span: tuple[int, int], format: Format | None = None) -> None:
        self.skips.append((range(*span), format))

    def byte(self, pos: int) -> bytes:
        try:
            return self.data[pos : pos + 1]
        except KeyError:
            return b""

    def keep(self) -> None:
        self.kept += self.cur

    def format(self, format: Format) -> None:
        self.formats.append((len(self.kept), format))


def lex(data: bytes) -> tuple[bytes, list[tuple[int, Format]]]:
    p = Lexer(data)
    while p.next():
        if m := p.try_match(rb"\\([\\\[<`*+=_-])"):
            p.skip(m.span())
            p.kept += m[1]

        elif m := p.try_match(rb"<(pre|script|style)"):
            m2 = p.find_at(m.end(), rb"</" + m[1] + rb">")
            p.skip((m.start(), m2.end()))
            p.kept += p.data[m.start() : m2.end()]

        elif m := p.try_match(rb"</?\w+"):
            m2 = p.try_find_at(m.end(), rb"\n\n")
            end = m2.start() if m2 else len(p.data)
            p.skip((m.start(), end))
            p.kept += p.data[m.start() : end]

        elif (m := p.try_match(rb"([+-]{3,})\n")) and p.pos == 0:
            m2 = p.find_at(m.end(), rb"\n" + re.escape(m[1]) + rb"(\n|$)")

            f = Metadata(content=p.data[m.end() : m2.start()])
            p.format(f)
            p.skip((m.start(), m2.end()))

        elif (m := p.try_match(rb"([*=_-])+(\n|$)")) and p.prev in (b"", b"\n"):
            f = Separator(style=m[1])
            p.format(f)
            p.skip((m.start(), m.end() - len(m[2])))

        elif (m := p.try_match(rb"(\*|-|\d+(\.))\s+")) and p.prev in (b"", b"\n"):
            p.skip(m.span())

            m2 = p.find_at(m.end(), rb"\n|$")

            f = Item(ordered=bool(m[2]))
            p.format(f)
            p.skip((m2.start(), m2.start()), f)

        elif (m := p.try_match(rb"\*{1,3}")) and (not p.prev or p.prev not in rb"*"):
            p.skip(m.span())

            f = Emphasis(strength=len(m[0]))
            p.format(f)

        elif m := p.try_match(rb"(\!)?\["):
            p.skip(m.span())

            me = p.find_at(m.end(), rb"\n|$")
            m2 = p.find_at(m.end(), rb"\]", endpos=me.start())

            if p.byte(m2.end()) == b"(":
                m3 = p.find_at(m2.end(), rb"\)", endpos=me.start())
                uri, *rest = p.data[m2.end() + 1 : m3.start()].split(maxsplit=1)
            else:
                m3 = m2
                uri = b""
                rest = None

            f = Reference(bang=bool(m[1]), uri=uri, alt=rest[0] if rest else b"")
            p.format(f)
            p.skip((m2.start(), m3.end()), f)

        elif (m := p.try_match(rb"(#+)\s*")) and p.prev in (b"", b"\n"):
            p.skip(m.span())

            m2 = p.find_at(m.end(), rb"\n|$")

            f = Heading(level=len(m[1]))
            p.format(f)
            p.skip((m2.start(), m2.start()), f)

        elif m := p.try_match(rb"(```+)([^\n]*)\n"):
            m2 = p.find_at(m.end(), m[1])

            f = Fence(content=p.data[m.end() : m2.start()], type=m[2])
            p.format(f)
            p.skip((m.start(), m2.end()))

        elif m := p.try_match(rb"`"):
            me = p.find_at(m.end(), rb"\n|$")
            m2 = p.find_at(m.end(), rb"`", endpos=me.start())

            f = Code(content=p.data[m.end() : m2.start()])
            p.format(f)
            p.skip((m.start(), m2.end()))

        elif (m := p.try_match(rb">\s*")) and p.prev in (b"", b"\n"):
            p.skip(m.span())

            m2 = p.find_at(m.end(), rb"\n|$")

            f = Quote()
            p.format(f)
            p.skip((m2.start(), m2.start()), f)

        elif (m := p.try_match(rb"\|")) and p.prev in (b"", b"\n"):
            m2 = p.find_at(m.end(), rb"\n|$")

            f = Row(content=p.data[m.start() : m2.start()])
            p.format(f)
            p.skip((m.start(), m2.start()))

        elif m := p.try_match(rb"\n\n+"):
            p.skip(m.span())

            f = Break()
            p.format(f)
            p.keep()  # preprocessor relies on some linebreaks

        else:
            p.keep()

    return bytes(p.kept), p.formats
