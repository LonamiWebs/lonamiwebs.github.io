"""
hacky script to convert saved wordpress sites to markdown for use in https://github.com/expectocode/pagong
"""
import bs4
import os
import sys
import re
from pathlib import Path
import urllib.parse
import dateutil.parser
import shutil

def header(tag_name):
    if m := re.match(r'h([1-6])', tag_name):
        return int(m[1])

def rewrite_img_src(src):
    if '//' in src:
        return src
    else:
        return src.split('/')[-1]

def handle(tag, pre=False, list_ty=None):
    if isinstance(tag, bs4.NavigableString):
        tag = str(tag)
        if pre:
            yield tag
        else:
            value = re.sub(r'\s+', ' ', tag)
            if not value.isspace():
                yield value
        return

    if tag.name == 'div':
        pass
    elif level := header(tag.name):
        yield '\n\n' + '#' * level + ' '
    elif tag.name == 'p':
        pass
    elif tag.name == 'em':
        yield '_'
    elif tag.name == 'strong':
        yield '**'
    elif tag.name == 'a':
        yield '['
    elif tag.name == 'code':
        if not pre:
            yield '`'
    elif tag.name == 'ul':
        list_ty = list_ty or []
        list_ty.append(None)
    elif tag.name == 'li':
        if not list_ty[-1]:
            yield '\n* '
        else:
            yield f'\n{list_ty[-1]}. '
            list_ty[-1] += 1
    elif tag.name == 'pre':
        pre = True
        yield '\n```\n'
    elif tag.name == 'figure':
        yield '\n'
    elif tag.name == 'img':
        yield f'![{tag["alt"]}]({rewrite_img_src(tag["src"])})'
    elif tag.name == 'hr':
        yield '\n\n----------\n\n'
    elif tag.name == 'ol':
        list_ty = list_ty or []
        list_ty.append(1)
    elif tag.name == 'br':
        yield '\n'
    elif tag.name == 'table':
        # bruh i ain't gonna parse tables
        yield tag.prettify()
        return
    elif tag.name == 'blockquote':
        yield '\n> '
    elif tag.name == 's':
        yield '~~'
    elif tag.name == 'figcaption':
        yield '\n_'
    elif tag.name == 'video':
        yield f'<video controls="controls" src="{rewrite_img_src(tag["src"])}"></video>'
    elif tag.name == 'cite':
        yield f'-- '
    elif tag.name in ('sub', 'sup'):
        yield f'<{tag.name}>'
    else:
        print('wtf is', tag.name)
        quit()

    for child in tag.children:
        yield from handle(child, pre=pre, list_ty=list_ty)

    if tag.name == 'div':
        pass
    elif header(tag.name):
        yield '\n\n'
    elif tag.name == 'p':
        yield '\n\n'
    elif tag.name == 'em':
        yield '_'
    elif tag.name == 'strong':
        yield '**'
    elif tag.name == 'a':
        yield f']({tag["href"]})'
    elif tag.name == 'code':
        if not pre:
            yield '`'
    elif tag.name == 'ul':
        list_ty.pop()
        yield '\n'
    elif tag.name == 'li':
        pass
    elif tag.name == 'pre':
        yield '\n```\n\n'
    elif tag.name == 'figure':
        yield '\n\n'
    elif tag.name == 'img':
        pass
    elif tag.name == 'hr':
        pass
    elif tag.name == 'ol':
        list_ty.pop()
        yield '\n'
    elif tag.name == 'br':
        pass
    elif tag.name == 'table':
        pass
    elif tag.name == 'blockquote':
        yield '\n'
    elif tag.name == 's':
        yield '~~'
    elif tag.name == 'figcaption':
        yield '_\n'
    elif tag.name == 'video':
        pass
    elif tag.name == 'cite':
        pass
    elif tag.name in ('sub', 'sup'):
        yield f'</{tag.name}>'


def iter_local_img(file: Path, tag):
    if isinstance(tag, bs4.NavigableString):
        return

    if tag.name == 'img':
        src = tag["src"]
        if '//' not in src:
            f = file.parent / urllib.parse.unquote(src)
            if f.is_file():
                yield f, rewrite_img_src(src)

    for child in tag.children:
        yield from iter_local_img(file, child)


def main():
    try:
        indir = Path(sys.argv[1])
        outroot = Path(sys.argv[2])
    except IndexError:
        print('usage:', sys.argv[0], '<IN DIR>', '<OUT DIR>')
        exit(1)

    outroot.mkdir(exist_ok=True)

    for file in indir.iterdir():
        if not file.is_file() or not file.name.endswith('.html'):
            continue

        with file.open(encoding='utf-8') as fd:
            soup = bs4.BeautifulSoup(fd.read(), 'html.parser')

        name = soup.find('link', rel='canonical')
        if name:
            name = name['href']
        else:
            name = soup.find(id='cancel-comment-reply-link')['href'].split('#')[0]
        name = name.rstrip('/').split('/')[-1]

        outdir = outroot / name
        title = soup.find(class_='entry-title').text
        _author = soup.find(class_='entry-author').text  # i'd rather not write this
        published = dateutil.parser.isoparse(soup.find(class_='published')['datetime']).replace(' ', 'T')  # ISO 8601
        updated = dateutil.parser.isoparse(soup.find(class_='updated')['datetime']).replace(' ', 'T')
        content = soup.find(class_='entry-content')

        outdir.mkdir(exist_ok=True)
        with open(outdir / 'post.md', 'w', encoding='utf-8') as fd:
            fd.write(f'''```meta
title: {title}
published: {published}
updated: {updated}
```
''')

            # hacky way to avoid the excessive amount of newlines except in pre blocks
            lines = ''.join(handle(content)).split('\n')
            pre = False
            empty = False
            for line in lines:
                if line.startswith('```'):
                    fd.write(line)
                    fd.write('\n')
                    pre = not pre
                    continue

                if not line or line.isspace():
                    empty = True
                else:
                    if empty:
                        fd.write('\n')
                        empty = False
                    fd.write(line)
                    fd.write('\n')

        for src, dst in iter_local_img(file, content):
            shutil.copy(src, outdir / dst)

main()
