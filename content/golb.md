<h1 class="title">My Golb</h1>

<p>Welcome to my golb!</p>

<p>It's like my blog, but with things that are a bit more… personal? Random? Spanish? Yeah!</p>

<hr>

<ul>

```python,inject
def inject(content):
    for page in sorted(content['golb'], key=lambda p: p.date, reverse=True):
        yield f'<li><a href="{page.permalink}">{page.title}</a></li>'
```

</ul>
