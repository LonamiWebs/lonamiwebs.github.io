"""
Non-compliant non-perfect minifiers (whitespace rules not respected).
"""


def minify_css(css: bytes) -> bytes:
    minified = bytearray()

    in_comment = False
    for i in range(len(css)):
        cur = css[i : i + 1]
        if cur == b"\r":
            continue

        if in_comment:
            if css[i - 1 : i + 1] == b"*/":
                in_comment = False
        elif css[i : i + 2] == b"/*":
            in_comment = True
        elif not cur.isspace() or minified[-1:] not in b" \t\n,;:{}()":
            minified += cur

    return bytes(minified)


def minify_html(html: bytes) -> bytes:
    minified = bytearray()

    in_other: bytes | None = None
    in_comment = False
    in_pre = False
    in_tag = False
    maybe_space = False
    for i in range(len(html)):
        cur = html[i : i + 1]
        if cur == b"\r":
            continue

        if in_other:
            if html[i - (len(in_other) - 1) : i + 1] == in_other:
                in_other = None
            minified += cur
        elif html[i : i + 6] == b"<style":
            in_other = b"</style>"
            minified += cur
        elif html[i : i + 7] == b"<script":
            in_other = b"</script>"
            minified += cur
        elif in_comment:
            if html[i - 2 : i + 1] == b"-->":
                in_comment = False
        elif html[i : i + 4] == b"<!--":
            in_comment = True
        elif in_pre:
            if html[i - 5 : i + 1] == b"</pre>":
                in_pre = False
            minified += cur
        elif html[i : i + 4] == b"<pre":
            in_pre = True
            minified += cur
        elif in_tag:
            if cur == b">":
                in_tag = False
                minified += cur
            elif not cur.isspace() or minified[-1:] not in b" \t\n<":
                minified += cur
        elif cur == b"<":
            in_tag = True
            maybe_space = False
            minified += cur
        else:
            if cur.isspace() and minified[-1:] == b">":
                maybe_space = True
            elif not cur.isspace() and maybe_space:
                maybe_space = False
                minified += b" "
                minified += cur
            elif not cur.isspace() or minified[-1:] not in b" \t\n":
                minified += cur

    return bytes(minified)
