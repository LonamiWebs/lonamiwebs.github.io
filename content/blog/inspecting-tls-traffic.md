+++
title = "Downloading Minecraft modpacks without using adware launchers"
date = 2023-08-05
[taxonomies]
category = ["network"]
tags = ["windows", "minecraft", "python", "tls"]
+++

Every now and then, I get the urge to download and install a [Minecraft](https://www.minecraft.net/) modpack. Mind you, it's often the case that I go through the installation process, and only then realize I don't *actually* want to play it, because I might not be in the mood to do so.

[CurseForge](https://www.curseforge.com/) has pretty much become the de-facto place to find and download mods. Or at least, that's the impression I get when looking for Minecraft mods.

There's just one tiny problem. They really, *really* want you to use their desktop application, either from the [Overwolf](https://www.overwolf.com/) launcher or the "standalone".

It used to be the case that you were able to go into the Files of a specific mod project and download a single ZIP with all the `.jar` mods. I can no longer find this. Some projects offer a "server" download, which is very handy, because it has all the mods, but it might not work directly for the client installation of your Minecraft launcher.

It seems your only choice is to painstakingly download the mods one by one, or use their ad-filled launchers. I haven't done a lot of analysis on these launchers, but it's probably safe to call them spyware. At least, the amount of ad-vendors they list when you open it to accept the privacy policy (because of course there's one, it's really just a webpage bundled into an executable file!) is concerning to me (the scrollbar is *long*).

I don't like either option, so I'm making up a third and a fourth.

> This post is about me figuring the whole thing out, so it's a bit all over the place. You've been warned!

Automating the browser
----------------------

My first instinct was to automate the browser. Using something like [Selenium from Python](https://selenium-python.readthedocs.io) didn't sound too bad, since I'm familiar with Python already.

It shouldn't be too bad, either. It is possible to download a ZIP file for the modpack which contains some configuration overrides along with the list of mods. Nice!

In theory, all we would need to do is grab the link for the modpack we desire, click on the Download button, inspect the ZIP, and figure out the next URLs. Cool!

Except, the download starts automatically, so you probably want a [custom download folder](https://stackoverflow.com/a/29777967). [Experimenting with preference values at runtime is just as annoying](https://stackoverflow.com/q/46470473). As far as I can tell, there's no clean way to detect these, so one would instead watch the directory and figure out when new files are added. And there are cookie banners that block the view and can appear at any time.

Overall, automating web interaction sounds extremely annoying. It could be done, but it's not really fun to me. And the idea of running *an entire web browser* to download a few files does not sit right with me.

Inspecting the app's traffic
----------------------------

Now, to do this, of course we will need to install one of those two launchers. It sucks, but it's only temporary. So go ahead and install one (or wait until we have the rest of the tools in place). You could use a virtual machine for this, but I cannot be bothered.

We'll obviously need a tool that can help us reading the network traffic of the launcher. I've lost track on how many times I've installed and uninstalled [Wireshark](https://www.wireshark.org/). It's really great, but I use it very sparingly and I am terrible at it, so I uninstall it after I'm done. Perhaps one day I'll finally get the hang of it and be able to do cool things.

Unfortunately, [Wireshark alone won't do](https://wiki.wireshark.org/TLS). The launcher, like most "applications" nowadays is really just a browser in disguise connecting and accessing resources via HTTPS. It's that "S" in "HTTPS" that bothers me. Because the connection is secure, the traffic is encrypted, and I cannot be bothered to figure out how the application uses TLS to try and have it spit the secrets.

The [Wireshark wiki does include some tips](https://wiki.wireshark.org/TLS). It would've been wise of me to at least try these first, such as running the `CurseForge.exe` from Git Bash with `SSLKEYLOGFILE=lol.log`, and it *actually* creates the file! But this is not what I did, so we won't go this route. It's also less widely applicable, so it's nice to explore more "general" solutions.

I had recently read the [announcement post for Clipper](https://jade.fyi/blog/announcing-clipper/), claiming to offer "TLS-transparent HTTP debugging for native apps". Sounds neat, that's just what I want! Except, [the easy way to build it is with Nix](https://github.com/lf-/clipper#development). I could boot into my Linux partition and try to use CurseForge there. I could even try WSL. But neither are in a workable state, so those options are out.

After [asking around the internet some more](https://stackoverflow.com/q/57306192), one might discover [PolarProxy](https://www.netresec.com/?page=PolarProxy). Sounds like it could work! After download, we can run it with:

```sh
PolarProxy -p 443,80 -o pcaplog -f proxyflows.log -x polarproxy.cer --httpconnect 10443 --certhttp 10080 --pcapoverip 57012
```

Where:

* `-p` sets the LISTEN-PORT "TCP port to bind proxy to" and DECRYPTED-PORT "TCP server port to use for decrypted traffic in PCAP". I'm not entirely sure what this does, but it was in their examples.
* `-o` sets the "output directory for hourly rotated PCAP files". Make sure you create that folder first! It makes it convenient to keep the traffic saved as we can later analyse it at our leisure.
* `-f` to "[l]og flow metadata for proxied sessions". Again, not sure what this is for. Probably not needed, but doesn't hurt.
* `-x` to "[e]xport DER encoded public CA certificate". We need this, because we want to Install Certificate to the Local Machine and to place it in the "Trusted Root Certification Authorities". This is explained in the same [PolarProxy](https://www.netresec.com/?page=PolarProxy) page. You shouldn't need this for subsequent runs though.
* `--httpconnect` to "[r]un local HTTP CONNECT proxy server". I had to use this when configuring proxy settings.
* `--certhttp` to "host the X.509 root CA cert". Probably not needed.
* `--pcapoverip` to "[s]erve decrypted TLS data as PCAP-over-IP". This makes it so that one can run Wireshark with `wireshark -k -i TCP@127.0.0.1:57012`, which is convenient to do it live.

We can then Change proxy settings in our System settings by enabling Use a proxy server with the address shown from the `ipconfig` command and this same port. (You probably could use `127.0.0.1` too.)

Now! With all that done, if we run the CurseForge application, we should be generating some PCAP files or, alternatively, getting the decrypted traffic in Wireshark! Go ahead and download the modpack you want from it, and once it's done, our PCAP should be complete. You can then stop the capture to reduce the amount of information we need to go through.

Understanding the app's traffic
-------------------------------

Great, now we have a packet capture with a lot of data in it. How do we go about finding the right things in there?

We can then type `http` into the filter to get suggestions for `http.request.full_uri or http2.request.full_uri`. Using that, we can filter the packets and see some paths in the Info column, as it will display anything with a full URI in it.

You should see GET requests to places like `/monsdk/electron/...` (how unsurprising) and, more interestingly, to `/v1/minecraft/modloader`! Clicking here we can see that it's making the request to the host api.curseforge.com! You can go ahead and colorize the TCP conversation to see related messages. If you're confident that it looks promising, you can also Follow the HTTP conversation entirely (which is a bit slower because it needs to run a new filter).

If you look into the HTTP data Wireshark decoded, you will spot... bingo! The `x-api-key` header! You can also see the `User-Agent` among other headers, so we can fully recreate this on our own now!

Now that we know this `x-api-key` is used, we can filter by it with `http contains "API ID PASTED HERE"`, and we will get all the traffic sent using that API key. In my capture, it seems like there are two conversations.

Feel free to go ahead and right-click the columns to change your Column Preferences to display other information you might want to go through. Simply set the Field to anything you want from the `http` namespace. And be sure to check the source and destination to know the direction a message is going in!

In my case, the second conversation is the one requesting the modpack I am interested in. Following that conversation and then displaying only the outgoing messages, it is very easy to see all the GET and POST requests being made:

```
GET /v1/mods/MODID
GET /v1/mods/MODID/files/FILEID
POST /v1/mods
GET /v1/mods/ANOTHER-MODID/files/ANOTHER-FILEID
...
```

It seems like first it will request information about the mod (or modpack) you are interested in. This presumably returns a list of files, and likely the latest one is chosen. After a POST to `/v1/mods`, a long trail of `GET` to various files is made. We can save the conversation to a file to more easily analyze it (such as disabling word-wrap to easily see where requests start and end).

[You might be able to use other tools to export the request-response](https://stackoverflow.com/q/8903815), but I just post-processed the text file a bit ot make it nicer to read (as there is no newline after response bodies). Nothing a simple `([^\n])((?:GET|POST) /)` regexp can't fix.

After this, the rest is history! We managed to read through the encrypted traffic to extract the API key, which [can be used to write a custom downloader](https://github.com/Lonami/curseforge-downloader). I won't go into the details, because the API could change any time, so that's left as an exercise to the reader. Enjoy!
