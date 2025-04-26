import unittest
from ..minifier import minify_css, minify_html


class TestMinifier(unittest.TestCase):
    def test_css(self):
        self.assertEqual(
            minify_css(
                rb"""
@media (prefers-color-scheme: dark) {
    .foo:target {/*comment*/
        background-color: rgba(255, 127, 0, 0.1);
        transition: color 300ms, border-bottom 300ms;
    }
}
"""
            ),
            b"@media (prefers-color-scheme:dark){.foo:target {background-color:rgba(255,127,0,0.1);transition:color 300ms,border-bottom 300ms;}}",
        )

    def test_html(self):
        self.assertEqual(
            minify_html(
                rb"""
< ul  class="left  top  right" id=nav>
    <li><a href="/">my&nbsp;site</a></li>
<!-- ignore this -->
    <li><a href="/blog">some   blog</a></li>
    <li><a href="/golb"> other <b>words</b>  too </a></li>
</ul><pre>
keep
<!-- not this -->
all</pre>  <script>
this

too</script>
"""
            ),
            rb"""<ul class="left top right" id=nav><li><a href="/">my&nbsp;site</a></li><li><a href="/blog">some blog</a></li><li><a href="/golb"> other <b>words</b> too </a></li></ul><pre>
keep

all</pre><script>
this

too</script>""",
        )
