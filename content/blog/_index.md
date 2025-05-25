+++
title = "My Blog"
date =
+++

<p id="welcome" onclick="pls_stop()">Welcome to my blog!</p>

Here I occasionally post new entries, mostly tech related. Perhaps it's tips for a new game I'm playing, perhaps it has something to do with FFI, or perhaps I'm fighting the borrow checker (just kidding, I'm over that. Mostly).

---

<script>
    const WELCOME_EN = 'Welcome to my blog!'
    const WELCOME_ES = '¡Bienvenido a mi blog!'
    const APOLOGIES = "ok sorry i'll stop"
    const REWRITE_DELAY = 5000
    const CHAR_DELAY = 30
    const welcome = document.getElementById('welcome')

    let deleting = true
    let english = false
    let stopped = false

    const pls_stop = () => {
        stopped = true
        welcome.innerHTML = APOLOGIES
    }

    const begin_rewrite = () => {
        if (stopped) {
            // now our visitor is angry :(
        } else if (deleting) {
            if (welcome.innerHTML == '…') {
                deleting = false
            } else {
                welcome.innerHTML = welcome.innerHTML.slice(0, -1) || '…'
            }
            setTimeout(begin_rewrite, CHAR_DELAY)
        } else {
            let text = english ? WELCOME_EN : WELCOME_ES
            welcome.innerHTML = text.slice(0, welcome.innerHTML.length + 1)
            deleting = welcome.innerHTML.length == text.length
            english ^= deleting
            setTimeout(begin_rewrite, deleting ? REWRITE_DELAY : CHAR_DELAY)
        }
    }

    setTimeout(begin_rewrite, REWRITE_DELAY)
</script>
