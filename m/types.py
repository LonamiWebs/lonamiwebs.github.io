"""
Shared type definitions.
"""

import abc
from typing import Any, Literal


class Format(abc.ABC):
    def __repr__(self):
        values = ", ".join(
            f"{name}={getattr(self, name)}"
            for name in dir(self)
            if not name.startswith("_")
        )
        return f"{self.__class__.__name__}({values})"

    def __eq__(self, other: Any) -> bool:
        return self is other or repr(self) == repr(other)


class Metadata(Format):
    def __init__(self, *, content: bytes = b""):
        self.content = content


class Emphasis(Format):
    def __init__(self, *, strength: int = 1):
        self.strength = strength


class Reference(Format):
    def __init__(
        self, *, bang: bool = False, uri: bytes = b"", alt: bytes = b"", id: bytes = b""
    ):
        self.bang = bang
        self.uri = uri
        self.alt = alt
        self.id = id


class Heading(Format):
    def __init__(self, *, level: int = 1):
        self.level = level


class Item(Format):
    def __init__(self, *, ordered: bool = False):
        self.ordered = ordered


class Break(Format):
    pass


class Quote(Format):
    pass


class Row(Format):
    def __init__(self, *, content: bytes = b""):
        self.content = content


class Code(Format):
    def __init__(self, *, content: bytes = b""):
        self.content = content


class Fence(Format):
    def __init__(self, *, content: bytes = b"", type: bytes = b""):
        self.content = content
        self.type = type


class Separator(Format):
    def __init__(self, *, style: bytes = b""):
        self.style = style


class Group(Format):
    def __init__(
        self, *, type: Literal[b"", b"ordered-list", b"unordered-list", b"table"] = b""
    ):
        self.type = type


class Raw(Format):
    def __init__(self, *, content: bytes = b""):
        self.content = content
