"""
HTML generator.
"""

import html
from .parser import parse_toml_ish
from .types import (
    Break,
    Code,
    Format,
    Group,
    Metadata,
    Emphasis,
    Raw,
    Reference,
    Heading,
    Item,
    Quote,
    Fence,
    Row,
    Separator,
)


def generate(text: bytes, formats: list[tuple[int, Format]]) -> bytes:
    segments: list[tuple[int, bytes]] = []

    open: list[Format] = []

    for i, f in formats:
        if isinstance(f, Metadata):
            title = parse_toml_ish(f.content)[b"title"][0]
            segments.append((i, b'<h1 class="title">' + title + b"</h1>"))

        elif isinstance(f, Emphasis):
            if f in open:
                segments.append(
                    (
                        i,
                        {1: b"</em>", 2: b"</strong>", 3: b"</em></strong>"}[
                            f.strength
                        ],
                    )
                )
                open.remove(f)
            else:
                segments.append(
                    (i, {1: b"<em>", 2: b"<strong>", 3: b"<strong><em>"}[f.strength])
                )
                open.append(f)

        elif isinstance(f, Reference):
            if f in open:
                segments.append((i, b"</a>"))
                open.remove(f)
            else:
                segments.append(
                    (
                        i,
                        b"<a "
                        + (b'id="' + f.id + b'" ' if f.id else b"")
                        + b'href="'
                        + f.uri
                        + b'">',
                    )
                )
                open.append(f)

        elif isinstance(f, Heading):
            if f in open:
                segments.append((i, b"</h" + str(f.level).encode() + b">"))
                open.remove(f)
            else:
                segments.append((i, b"<h" + str(f.level).encode() + b">"))
                open.append(f)

        elif isinstance(f, Item):
            # probably need an extra pass to merge elements.
            # maybe injector should be pre-processor and do this.
            # it is not injecting anything
            if f in open:
                segments.append((i, b"</li>"))
                open.remove(f)
            else:
                segments.append((i, b"<li>"))
                open.append(f)

        elif isinstance(f, Quote):
            if f in open:
                segments.append((i, b"</blockquote>"))
                open.remove(f)
            else:
                segments.append((i, b"<blockquote>"))
                open.append(f)

        elif isinstance(f, Fence):
            # also lang
            segments.append(
                (i, b"<pre>" + html.escape(f.content.decode()).encode() + b"</pre>")
            )

        elif isinstance(f, Code):
            # also lang
            segments.append(
                (i, b"<code>" + html.escape(f.content.decode()).encode() + b"</code>")
            )

        elif isinstance(f, Row):
            # probably need an extra pass to merge elements.
            # maybe injector should be pre-processor and do this.
            # it is not injecting anything
            segments.append(
                (
                    i,
                    b"<tr><td>"
                    + html.escape(f.content.decode()).encode()
                    + b"</td></tr>",
                )
            )

        elif isinstance(f, Separator):
            segments.append((i, b"<hr />"))

        elif isinstance(f, Break):
            pass
            # segments.append((i, b"<br>"))

        elif isinstance(f, Raw):
            segments.append((i, f.content))

        elif isinstance(f, Group):
            if f.type == b"ordered-list":
                if f in open:
                    segments.append((i, b"</ol>"))
                    open.remove(f)
                else:
                    segments.append((i, b"<ol>"))
                    open.append(f)
            elif f.type == b"unordered-list":
                if f in open:
                    segments.append((i, b"</ul>"))
                    open.remove(f)
                else:
                    segments.append((i, b"<ul>"))
                    open.append(f)
            elif f.type == b"table":
                if f in open:
                    segments.append((i, b"</tbody></table>"))
                    open.remove(f)
                else:
                    segments.append((i, b"<table><tbody>"))
                    open.append(f)
            else:
                raise RuntimeError(f"missing generator code for format: {f}")
        else:
            raise RuntimeError(
                f"missing generator code for format: {f.__class__.__name__}"
            )

    # if open:
    #     raise ValueError(f"not all formatting was closed")

    formatted = bytearray()
    remap = {
        # b"&": b"&amp;"[::-1],
        # b"<": b"&lt;"[::-1],
        # b">": b"&gt;"[::-1],
    }

    segments.sort(key=lambda t: t[0])  # why?

    for i in reversed(range(len(text))):
        c = text[i : i + 1]
        formatted += remap.get(c, c)
        while segments:
            p, a = segments[-1]
            if p != i:
                break
            formatted += a[::-1]
            segments.pop()

    formatted.reverse()
    return bytes(formatted)
