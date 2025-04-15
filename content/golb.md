{% extends "base.html" %}

{% block content %}
<h1 class="title">{{ section.title }}</h1>
<p>Welcome to my golb!</p>

<p>It's like my blog, but with things that are a bit more… personal? Random? Spanish? Yeah!</p>

<hr>

<ul>
    {% for page in section.pages %}
    <li><a href="{{ page.permalink | safe }}">{{ page.title }}</a></li>
    {% endfor %}
</ul>
{% endblock %}
