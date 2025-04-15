<style>
.golb {
    transform: scaleY(-1);
    transition: transform 300ms;
}

.golb:hover {
    transform: scaleY(1);
}
</style>

<h1>Lonami's Site</h1>
<p>Welcome to my personal website! This page has had several redesigns
over time, but now I've decided to let it be as minimalist as possible
(proud to be under 32KB!).</p>

<h2 id="about">About me</h2>
<p>Spanish male
<span id="age">born in 1998</span>.
I have been programming
<span id="programming">since 2012</span>
and it is my passion.</p>

<p>I enjoy nature, taking pictures, playing video-games,
drawing vector graphics, or just chatting online.</p>

<p>I can speak perfect Spanish, read and write perfect English
and Python, and have programmed in C#, Java, JavaScript, Rust,
some C and C++, and designed pages like this with plain HTML
and CSS.</p>

<p>On the Internet, I'm often known as <i>Lonami</i>, although
my real name is simply my nick name, put the other way round.</p>

<h2 id="projects">Project highlights</h2>
<ul>
<li>
<a href="https://github.com/LonamiWebs/Telethon/">Telethon</a>:
Python implementation of the Telegram's API.
</li>
<li>
<a href="klooni">1010! Klooni</a>: libGDX simple puzzle
game based on the original <i>1010!</i>.
</li>
<li>
<a href="https://github.com/LonamiWebs/Stringlate/">Stringlate</a>:
Android application that makes it easy to translate other FOSS apps.
</li>
</ul>
<p>These are only my <i>Top 3</i> projects, the ones I consider to be
the most successful. If you're curious about what else I've done, feel
free to check out my
<a href="https://github.com/LonamiWebs/">GitHub</a>.</p>

<h2 id="more-links">More links</h2>
<dl>
    <dt><a href="/blog"><img src="/img/blog.svg" alt="" /> My blog</a></dt>
    <dd>Sometimes I blog about things, whether it's games, techy stuff, or random life stuff.</dd>

    <dt><a href="/golb"><img src="/img/blog.svg" class="golb" alt="" /> My golb</a></dt>
    <dd>What? You don't know what a golb is? It's like a blog, but less conventional.</dd>

    <dt><a href="https://github.com/LonamiWebs"><img src="/img/github.svg" alt="" /> My GitHub</a></dt>
    <dd>By far what I'm most proud of. I love releasing my projects as open source. There is no reason not to!</dd>

    <dt><a href="/utils"><img src="/img/utils.svg" alt="" /> Several Utilities</a></dt>
    <dd>Random things I've put online because I keep forgetting about them.</dd>

    <dt><a href="/stopwatch.html"><img src="/stopwatch.svg" width="24" height="24" alt="" /> stopwatch</a> &amp; <a href="/canvas.html">canvas</a></dt>
    <dd>An extremely simple JavaScript-based stopwatch &amp; canvas for sketching.</dd>

    <dt><a href="donate"><img src="/img/donate.svg" alt="" /> Donate</a></dt>
    <dd>Some people like what I do and want to compensate me for it, but I'm fine with compliments if you can't afford a donation!</dd>

    <dt><a href="humans.txt"><img src="/img/humans.svg" alt="" /> humans.txt</a></dt>
    <dd><a href="http://humanstxt.org/">We are humans, not robots.</a></dd>
</dl>

<h2 id="contact">Contact</h2>
<p>You can send me a private email to
<a href="mailto:totufals@hotmail.com">totufals[at]hotmail[dot]com</a>
and I will try to reply as soon as I can. Please don't use the email
if you need help with a specific project, this is better discussed in
a different public place where everyone can benefit from it.</p>

<script type="text/javascript">
    now = (new Date()).getFullYear();
    document.getElementById("age").innerHTML = "aged " + (now - 1999);
    document.getElementById("programming").innerHTML = "for " + (now - 2012) + " years";
</script>
