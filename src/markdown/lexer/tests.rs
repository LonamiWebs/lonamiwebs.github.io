use super::*;

#[test]
fn test_escaping() {
    for c in "\\[<`*+=_-".chars() {
        let c_byte = c.to_string();
        let c_byte = c_byte.as_bytes();

        let text = format!("\\{c}\\text\\n\\{c}\\");
        let text = text.as_bytes();
        assert_eq!(
            lex(text).collect::<Vec<_>>(),
            vec![
                Token::Text(c_byte),
                Token::Text(b"text"),
                Token::Text(b"n"),
                Token::Text(c_byte),
            ],
        );
    }
}

#[test]
fn test_raw() {
    let text = b"text <pre>keep\nas-is\n\n  </pre>done<style></style>end<script unclosed";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Text(b"text "),
            Token::Raw(b"<pre>keep\nas-is\n\n  </pre>"),
            Token::Text(b"done"),
            Token::Raw(b"<style></style>"),
            Token::Text(b"end"),
            Token::Raw(b"<script unclosed"),
        ],
    );

    let text = b"<h1>h1</h1>\n<p>long\nparagraph</p>\n\n<h2 id=\"about\">h2</h2>\n<p>another paragraph</p>";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Raw(b"<h1>h1</h1>"),
            Token::Break { hard: false },
            Token::Raw(b"<p>long\nparagraph</p>"),
            Token::Break { hard: true },
            Token::Raw(b"<h2 id=\"about\">h2</h2>"),
            Token::Break { hard: false },
            Token::Raw(b"<p>another paragraph</p>"),
        ],
    );

    let text = b"<script></script>\n";

    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Raw(b"<script></script>"),
            Token::Break { hard: false }
        ]
    );

    let text = b"<span class=\"cls\">span</span>[^ref] text\n\n# heading";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Raw(b"<span class=\"cls\">span</span>"),
            Token::BeginReference { bang: false },
            Token::Text(b"^ref"),
            Token::EndReference {
                uri: b"^ref",
                alt: b"",
                lazy: true
            },
            Token::Text(b" text"),
            Token::Break { hard: true },
            Token::Heading(1),
            Token::Text(b"heading"),
        ]
    );

    let text = b"<details open><summary>summary</summary>\n\n> quote\n\n</details>";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Raw(b"<details open><summary>summary</summary>\n\n"),
            Token::Quote,
            Token::Indent(1),
            Token::Text(b"quote"),
            Token::Break { hard: true },
            Token::Raw(b"</details>"),
        ]
    );

    let text = b"&nbsp; & text ;";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![Token::Raw(b"&nbsp;"), Token::Text(b" & text ;"),]
    );
}

#[test]
fn test_html() {
    let text = b"<p>p *tag*</p><details>\n\ndetails *tag*\n\n</details>\n\ntext";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Raw(b"<p>p *tag*</p>"),
            Token::Raw(b"<details>\n\n"),
            Token::Text(b"details "),
            Token::Emphasis(1),
            Token::Text(b"tag"),
            Token::Emphasis(1),
            Token::Break { hard: true },
            Token::Raw(b"</details>"),
            Token::Break { hard: true },
            Token::Text(b"text"),
        ],
    );

    let text = b"<noscript>js</noscript>\n\n> quote";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Raw(b"<noscript>js</noscript>"),
            Token::Break { hard: true },
            Token::Quote,
            Token::Indent(1),
            Token::Text(b"quote"),
        ],
    );
}

#[test]
fn test_metadata_ok() {
    for separator in ["---", "---------", "+++", "+++++++++"] {
        let text = format!("{separator}\nmeta\n{separator}");
        let text = text.as_bytes();
        assert_eq!(lex(text).collect::<Vec<_>>(), vec![Token::Meta(b"meta")]);

        let text = format!("{separator}\nmeta\n{separator}\ntext");
        let text = text.as_bytes();
        assert_eq!(
            lex(text).collect::<Vec<_>>(),
            vec![Token::Meta(b"meta"), Token::Text(b"text")]
        );

        let text = format!("{separator}\nmeta\n{separator}text");
        let text = text.as_bytes();
        assert_eq!(
            lex(text).collect::<Vec<_>>(),
            vec![
                Token::Text(separator.as_bytes()),
                Token::Break { hard: false },
                Token::Text(b"meta"),
                Token::Break { hard: false },
                Token::Text(format!("{separator}text").as_bytes())
            ]
        );
    }

    let text = b"-";
    assert_eq!(lex(text).collect::<Vec<_>>(), vec![Token::Separator(b'-')]);

    let text = format!("text\n+++\nmeta\n+++\ntext");
    let text = text.as_bytes();
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Text(b"text"),
            Token::Break { hard: false },
            Token::Text(b"+++"),
            Token::Break { hard: false },
            Token::Text(b"meta"),
            Token::Break { hard: false },
            Token::Text(b"+++"),
            Token::Break { hard: false },
            Token::Text(b"text")
        ]
    );
}

#[test]
fn test_separator() {
    for &c in b"*=_-" {
        for length in [1, 3, 10] {
            let separator = vec![c; length];
            let separator = String::from_utf8_lossy(&separator);
            let text = format!("start\n{separator}");
            let text = text.as_bytes();
            assert_eq!(
                lex(text).collect::<Vec<_>>(),
                vec![
                    Token::Text(b"start"),
                    Token::Break { hard: false },
                    Token::Separator(c)
                ]
            );

            let text = format!("start\n{separator}\ntext");
            let text = text.as_bytes();
            assert_eq!(
                lex(text).collect::<Vec<_>>(),
                vec![
                    Token::Text(b"start"),
                    Token::Break { hard: false },
                    Token::Separator(c),
                    Token::Break { hard: false },
                    Token::Text(b"text")
                ]
            );
        }
    }
}

#[test]
fn test_item() {
    for item in ["*", "-", "0.", "1."] {
        let text = format!("{item} text");
        let text = text.as_bytes();
        assert_eq!(
            lex(text).collect::<Vec<_>>(),
            vec![
                Token::BeginItem {
                    ordered: item.as_bytes()[0].is_ascii_digit(),
                },
                Token::Text(b"text"),
            ]
        );
    }

    let text = b"* star\n0. zero\n- dash\n1. one";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::BeginItem { ordered: false },
            Token::Text(b"star"),
            Token::Break { hard: false },
            Token::BeginItem { ordered: true },
            Token::Text(b"zero"),
            Token::Break { hard: false },
            Token::BeginItem { ordered: false },
            Token::Text(b"dash"),
            Token::Break { hard: false },
            Token::BeginItem { ordered: true },
            Token::Text(b"one"),
        ]
    );

    let text = b"* a\n* [b](u)\n* c\n\n  c\n* d";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::BeginItem { ordered: false },
            Token::Text(b"a"),
            Token::Break { hard: false },
            Token::BeginItem { ordered: false },
            Token::BeginReference { bang: false },
            Token::Text(b"b"),
            Token::EndReference {
                uri: b"u",
                alt: b"",
                lazy: false
            },
            Token::Break { hard: false },
            Token::BeginItem { ordered: false },
            Token::Text(b"c"),
            Token::Break { hard: true },
            Token::Indent(2),
            Token::Text(b"c"),
            Token::Break { hard: false },
            Token::BeginItem { ordered: false },
            Token::Text(b"d")
        ]
    );

    let text = b"* `code` text\n* text";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::BeginItem { ordered: false },
            Token::Code(b"code"),
            Token::Text(b" text"),
            Token::Break { hard: false },
            Token::BeginItem { ordered: false },
            Token::Text(b"text"),
        ]
    );

    let text = b"* text\n  * text";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::BeginItem { ordered: false },
            Token::Text(b"text"),
            Token::Break { hard: false },
            Token::Indent(2),
            Token::BeginItem { ordered: false },
            Token::Text(b"text"),
        ]
    );
}

#[test]
fn test_emphasis() {
    let text = b"*1* **2** ***3***";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Emphasis(1),
            Token::Text(b"1"),
            Token::Emphasis(1),
            Token::Text(b" "),
            Token::Emphasis(2),
            Token::Text(b"2"),
            Token::Emphasis(2),
            Token::Text(b" "),
            Token::Emphasis(3),
            Token::Text(b"3"),
            Token::Emphasis(3),
        ]
    );
}

#[test]
fn test_reference() {
    for bang in [false, true] {
        for lazy in [false, true] {
            for alt in [false, true] {
                let a = if bang { "!" } else { "" };
                let (b, c) = if lazy { ("[", "]") } else { ("(", ")") };
                let d = if alt { " \"alt\"" } else { "" };

                let text = format!("{a}[text]{b}url{d}{c}");
                let text = text.as_bytes();
                assert_eq!(
                    lex(text).collect::<Vec<_>>(),
                    vec![
                        Token::BeginReference { bang },
                        Token::Text(b"text"),
                        Token::EndReference {
                            uri: b"url",
                            alt: if alt { b"alt" } else { b"" },
                            lazy
                        }
                    ],
                );
            }
        }
    }

    let text = b"[text]]()";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::BeginReference { bang: false },
            Token::Text(b"text"),
            Token::EndReference {
                uri: b"text",
                alt: b"",
                lazy: true
            },
            Token::Text(b"]()")
        ]
    );
}

#[test]
fn test_footnote() {
    let text = b"text![^1]:\n\n[^1]: footnote";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Text(b"text!"),
            Token::BeginReference { bang: false },
            Token::Text(b"^1"),
            Token::EndReference {
                uri: b"^1",
                alt: b"",
                lazy: true
            },
            Token::Text(b":"),
            Token::Break { hard: true },
            Token::BeginDefinition(b"^1"),
            Token::Text(b"footnote"),
        ],
    );
}

#[test]
fn test_heading() {
    for length in 1..7 {
        let text = format!("{:#^length$} heading\ntext", "");
        let text = text.as_bytes();

        assert_eq!(
            lex(text).collect::<Vec<_>>(),
            vec![
                Token::Heading(length as u8),
                Token::Text(b"heading"),
                Token::Break { hard: false },
                Token::Text(b"text"),
            ]
        );
    }
}

#[test]
fn test_fence() {
    let text = b"```lang\npre```\ntext";

    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Fence {
                lang: b"lang",
                text: b"pre"
            },
            Token::Text(b"text"),
        ]
    );
    let text = b"```````lang spaces\npre\n```\nfalse```````";

    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![Token::Fence {
            lang: b"lang spaces",
            text: b"pre\n```\nfalse"
        }]
    );
}

#[test]
fn test_code() {
    let text = b"`co\\de <tag> end`text";

    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![Token::Code(b"co\\de <tag> end"), Token::Text(b"text")]
    );
}

#[test]
fn test_quote() {
    let text = b"> quote";

    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![Token::Quote, Token::Indent(1), Token::Text(b"quote")]
    );

    let text = b"> * list";

    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Quote,
            Token::Indent(1),
            Token::BeginItem { ordered: false },
            Token::Text(b"list")
        ]
    );
}

#[test]
fn test_break() {
    let text = b"\n\nleading";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![Token::Break { hard: true }, Token::Text(b"leading")]
    );

    let text = b"trailing\n\n";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![Token::Text(b"trailing"), Token::Break { hard: true },]
    );

    let text = b"mid\ndle";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Text(b"mid"),
            Token::Break { hard: false },
            Token::Text(b"dle")
        ]
    );

    let text = b"mid\n  \ndle";
    assert_eq!(
        lex(text).collect::<Vec<_>>(),
        vec![
            Token::Text(b"mid"),
            Token::Break { hard: true },
            Token::Text(b"dle")
        ]
    );
}
