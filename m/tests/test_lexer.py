import unittest
from ..lexer import lex
from ..types import (
    Metadata,
    Separator,
    Item,
    Emphasis,
    Reference,
    Heading,
    Fence,
    Code,
    Quote,
    Row,
    Break,
)


class TestLexer(unittest.TestCase):
    def test_escaping(self):
        with self.subTest("happy path"):
            for c in (b"\\", b"[", b"<", b"`", b"*", b"+", b"=", b"_", b"-"):
                res = lex(b"\\" + c + b"\\text\\n\\" + c)
                exp = c + b"\\text\\n" + c, []
                self.assertEqual(res, exp)

    def test_html(self):
        with self.subTest("happy path"):
            res = lex(b"<p>p *tag*</p><details>\n\ndetails *tag*\n\n</details>\n\ntext")
            exp = b"<p>p *tag*</p><details>\ndetails tag\n</details>\ntext", [
                (23, Break()),
                (32, Emphasis(strength=1)),
                (35, Emphasis(strength=1)),
                (35, Break()),
                (46, Break()),
            ]
            self.assertEqual(res, exp)

    def test_metadata(self):
        with self.subTest("happy path"):
            res = lex(b"---\nmeta\n---")
            exp = b"", [(0, Metadata(content=b"meta"))]
            self.assertEqual(res, exp)

            res = lex(b"---\nmeta\n---\ntext")
            exp = b"text", [(0, Metadata(content=b"meta"))]

            self.assertEqual(res, exp)

            res = lex(b"---------\nmeta\n---------\ntext")
            exp = b"text", [(0, Metadata(content=b"meta"))]

            self.assertEqual(res, exp)

            res = lex(b"+++\nmeta\n+++")
            exp = b"", [(0, Metadata(content=b"meta"))]

            self.assertEqual(res, exp)

            res = lex(b"+++\nmeta\n+++\ntext")
            exp = b"text", [(0, Metadata(content=b"meta"))]

            self.assertEqual(res, exp)

            res = lex(b"+++++++++\nmeta\n+++++++++\ntext")
            exp = b"text", [(0, Metadata(content=b"meta"))]

            self.assertEqual(res, exp)

        with self.subTest("false terminator"):
            with self.assertRaises(ValueError):
                res = lex(b"+++\nmeta\n+++false")

            res = lex(b"+++\nmeta\n+++false\n+++\ntext")
            exp = b"text", [(0, Metadata(content=b"meta\n+++false"))]

            self.assertEqual(res, exp)

        with self.subTest("mismatching terminator"):
            with self.assertRaises(ValueError):
                res = lex(b"+++\nmeta\n---")

            with self.assertRaises(ValueError):
                res = lex(b"---\nmeta\n+++")

            with self.assertRaises(ValueError):
                res = lex(b"+++++\nmeta\n+++")

            with self.assertRaises(ValueError):
                res = lex(b"---\nmeta\n-----")

    def test_separator(self):
        with self.subTest("happy path"):
            for c in (b"*", b"=", b"_", b"-"):
                for length in (1, 3, 10):
                    res = lex(b"start\n" + c * length)
                    exp = b"start\n", [(6, Separator(style=c))]
                    self.assertEqual(res, exp)

                    res = lex(b"start\n" + c * length + b"\ntext")
                    exp = b"start\n\ntext", [(6, Separator(style=c))]
                    self.assertEqual(res, exp)

    def test_item(self):
        with self.subTest("happy path"):
            for c in (b"*", b"-", b"0.", b"1."):
                res = lex(c + b" text")
                exp = (
                    b"text",
                    [
                        (0, Item(ordered=c[:1].isdigit())),
                        (4, Item(ordered=c[:1].isdigit())),
                    ],
                )
                self.assertEqual(res, exp)

            res = lex(b"* star\n0. zero\n- dash\n1. one")
            exp = (
                b"star\nzero\ndash\none",
                [
                    (0, Item(ordered=False)),
                    (4, Item(ordered=False)),
                    (5, Item(ordered=True)),
                    (9, Item(ordered=True)),
                    (10, Item(ordered=False)),
                    (14, Item(ordered=False)),
                    (15, Item(ordered=True)),
                    (18, Item(ordered=True)),
                ],
            )
            self.assertEqual(res, exp)

    def test_emphasis(self):
        with self.subTest("happy path"):
            res = lex(b"*1* **2** ***3***")
            exp = (
                b"1 2 3",
                [
                    (0, Emphasis(strength=1)),
                    (1, Emphasis(strength=1)),
                    (2, Emphasis(strength=2)),
                    (3, Emphasis(strength=2)),
                    (4, Emphasis(strength=3)),
                    (5, Emphasis(strength=3)),
                ],
            )
            self.assertEqual(res, exp)

    def test_reference(self):
        with self.subTest("happy path"):
            res = lex(b'[t](1) ![e](2) [x](3 "a") ![t](4 "b")')
            exp = (
                b"t e x t",
                [
                    (0, Reference(bang=False, uri=b"1")),
                    (1, Reference(bang=False, uri=b"1")),
                    (2, Reference(bang=True, uri=b"2")),
                    (3, Reference(bang=True, uri=b"2")),
                    (4, Reference(bang=False, uri=b"3", alt=b'"a"')),
                    (5, Reference(bang=False, uri=b"3", alt=b'"a"')),
                    (6, Reference(bang=True, uri=b"4", alt=b'"b"')),
                    (7, Reference(bang=True, uri=b"4", alt=b'"b"')),
                ],
            )
            self.assertEqual(res, exp)

    def test_heading(self):
        with self.subTest("happy path"):
            for length in range(1, 7):
                res = lex(b"#" * length + b" heading\ntext")
                exp = (
                    b"heading\ntext",
                    [(0, Heading(level=length)), (7, Heading(level=length))],
                )
                self.assertEqual(res, exp)

    def test_fence(self):
        with self.subTest("happy path"):
            res = lex(b"```lang\npre```\ntext")
            exp = (
                b"\ntext",
                [(0, Fence(content=b"pre", type=b"lang"))],
            )
            self.assertEqual(res, exp)

            res = lex(b"```````lang spaces\npre\n```\nfalse```````")
            exp = (
                b"",
                [(0, Fence(content=b"pre\n```\nfalse", type=b"lang spaces"))],
            )
            self.assertEqual(res, exp)

    def test_code(self):
        with self.subTest("happy path"):
            res = lex(b"`co\\de`")
            exp = (
                b"",
                [(0, Code(content=b"co\\de"))],
            )
            self.assertEqual(res, exp)

    def test_quote(self):
        with self.subTest("happy path"):
            res = lex(b"> quote")
            exp = (
                b"quote",
                [(0, Quote()), (5, Quote())],
            )
            self.assertEqual(res, exp)

    def test_row(self):
        with self.subTest("happy path"):
            res = lex(b"|1|2|3|")
            exp = (
                b"",
                [(0, Row(content=b"|1|2|3|"))],
            )
            self.assertEqual(res, exp)

    def test_break(self):
        with self.subTest("happy path"):
            res = lex(b"\n\n")
            exp = (
                b"\n",
                [(0, Break())],
            )
            self.assertEqual(res, exp)
