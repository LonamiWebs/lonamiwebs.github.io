import unittest
from ..parser import parse_toml_ish


class TestParser(unittest.TestCase):
    def test_escaping(self):
        res = parse_toml_ish(
            rb"""
title = "Some, title"
date = 1234-56-78
[taxonomies]
category = ["cat"]
tags = ["t", "a", "g"]
"""
        )
        exp = {
            b"title": [b"Some, title"],
            b"date": [b"1234-56-78"],
            b"category": [b"cat"],
            b"tags": [b"t", b"a", b"g"],
        }
        self.assertEqual(res, exp)
