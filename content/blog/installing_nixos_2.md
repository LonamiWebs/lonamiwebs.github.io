+++
title = "Installing NixOS, Take 2"
date = 2019-02-15
updated = 2019-02-16
+++

This is my second take at installing NixOS, after a while being frustrated with Arch Linux and the fact that a few kernel upgrades ago, the system crashed randomly from time to time. `journalctl` did not have any helpful hints and I thought reinstalling could be worthwhile anyway.

This time, I started with more knowledge! The first step is heading to the [NixOS website](https://nixos.org) and downloading their minimal installation CD for 64 bits. I didn't go with their graphical live CD, because their [installation manual](https://nixos.org/nixos/manual) is a wonderful resource that guides you nicely.

Once you have downloaded their `.iso`, you should probably verify it's `sha256sum` and make sure that it matches. The easiest thing to do in my opinion is using an USB to burn the image in it. Plug it in and check its device name with `fdisk -l`. In my case, it was `/dev/sdb`, so I went ahead with it and ran `dd if=nixos.iso of=/dev/sdb status=progress`. Make sure to run `sync` once that's done.

If either `dd` or `sync` seem "stuck" in the end, they are just flushing the changes to disk to make sure all is good. This is normal, and depends on your drives.

Now, reboot your computer with the USB plugged in and make sure to boot into it. You should be welcome with a pretty screen. Just select the first option and wait until it logs you in as root. Once you're there you probably want to `loadkeys es` or whatever your keyboard layout is, or you will have a hard time with passwords, since the characters are all over the place.

In a clean disk, you would normally create the partitions now. In my case, I already had the partitions made (100MB for the EFI system, where `/boot` lives, 40GB for the root `/` partition with my old Linux installation, and 700G for `/home`), so I didn't need to do anything here. The manual showcases `parted`, but I personally use `fdisk`, which has very helpful help I check every time I use it.

**Important**: The `XY` in `/dev/sdXY` is probably different in your system! Make sure you use `fdisk -l` to see the correct letters and numbers!

With the partitions ready in my UEFI system, I formatted both `/` and `/boot` just to be safe with `mkfs.ext4 -L nixos /dev/sda2` and `mkfs.fat -F 32 -n boot /dev/sda1` (remember that these are the letters and numbers used in my partition scheme). Don't worry about the warning in the second command regarding lowercase letters and Windows. It's not really an issue.

Now, since we gave each partition a label, we can easily mount them through `mount /dev/disk/by-label/nixos /mnt` and, in UEFI systems, be sure to `mkdir -p /mnt/boot` and `mount /dev/disk/by-label/boot /mnt/boot`. I didn't bother setting up swap, since I have 8GB of RAM in my laptop and that's really enough for my use case.

With that done, we will now ask the configuration wizard to do some work for us (in particular, generate a template) with `nixos-generate-config --root /mnt`. This generates a very well documented file that we should edit right now (and this is important!) with whatever editor you prefer. I used `vim`, but you can change it for `nano` if you prefer.

On to the configuration file, we need to enable a few things, so `vim /mnt/etc/nixos/configuration.nix` and start scrolling down. We want to make sure to uncomment:

```
# We really want network!
networking.wireless.enable = true;

# This "fixes" the keyboard layout. Put the one you use.
i18n = {
consoleKeyMap = "es";
}

# Timezones are tricky so let's get this right.
time.timeZone = "Europe/Madrid";

# We *really* want some base packages installed, such as
# wpa_supplicant, or we won't have a way to connect to the
# network once we install...
environment.systemPackages = with pkgs; [
wpa_supplicant wget curl vim neovim cmus mpv firefox git tdesktop
];

# Printing is useful, sure, enable CUPS
services.printing.enable = true;

# We have speakers, let's make use of them.
sound.enable = true;
hardware.pulseaudio.enable = true;

# We want the X11 windowing system enabled, in Spanish.
services.xserver.enable = true;
services.xserver.layout = "es";

# I want a desktop manager in my laptop.
# I personally prefer XFCE, but the manual shows plenty
# of other options, such as Plasma, i3 WM, or whatever.
services.xserver.desktopManager.xfce.enable = true;
services.xserver.desktopManager.default = "xfce";

# Touchpad is useful (although sometimes annoying) in a laptop
services.xserver.libinput.enable = true;

# We don't want to do everything as root!
users.users.lonami = {
isNormalUser = true;
uid = 1000;
home = "/home/lonami";
extraGroups = [ "wheel" "networkmanager" "audio" ];
};
```

*(Fun fact, I overlooked the configuration file until I wrote this and hadn't noticed sound/pulseaudio was there. It wasn't hard to find online how to enable it though!)*

Now, let's modify `hardware-configuration.nix`. But if you have `/home` in a separate partition like me, you should run `blkid` to figure out its UUID. To avoid typing it out myself, I just ran `blkid >> /mnt/etc/nixos/hardware-configuration.nix` so that I could easily move it around with `vim`:

```
# (stuff...)

fileSystems."/home" =
{ device = "/dev/disk/by-uuid/d344c686-cae7-4dd3-840e-308eddf86608";
fsType = "ext4";
};

# (more stuff...)
```

Note that, obviously, you should put your own partition's UUID there. Modifying the configuration is where I think the current NixOS' manual should have made more emphasis, at this step of the installation. They do detail it below, but that was already too late in my first attempt. Anyway, you can boot from the USB and run `nixos-install` as many times as you need until you get it working!

But before installing, we need to configure the network since there are plenty of things to download. If you want to work from WiFi, you should first figure out the name of your network card with `ip link show`. In my case it's called `wlp3s0`. So with that knowledge we can run `wpa_supplicant -B -i wlp3s0 -c <(wpa_passphrase SSID key)`. Be sure to replace both `SSID` and `key` with the name of your network and password key, respectively. If they have spaces, surround them in quotes.

Another funny pitfall was typing `wpa_supplicant` in the command above twice (instead of `wpa_passphrase`). That sure spit out a few funny errors! Once you have ran that, wait a few seconds and `ping 1.1.1.1` to make sure that you can reach the internet. If you do, `^C` and let's install NixOS!

```
nixos-install
```

Well, that was pretty painless. You can now `reboot` and enjoy your new, functional system.

Afterword
---------

The process of installing NixOS was really painless once you have made sense out of what things mean. I was far more pleased this time than in my previous attempt, despite the four attempts I needed to have it up and running.

However not all is so good. I'm not sure where I went wrong, but the first time I tried with `i3` instead of `xfce`, all I was welcome with was a white, small terminal in the top left corner. I even generated a configuration file with `i3-config-wizard` to make sure it could detect my Mod1/Mod4 keys (which, it did), but even after rebooting, my commands weren't responding. For example, I couldn't manage to open another terminal with `Mod1+Enter`. I'm not even sure that I was in `i3`…

In my very first attempt, I pressed `Alt+F8` as suggested in the welcome message. This took me an offline copy of the manual, which is really nicely done. Funny enough, though, I couldn't exit `w3m`. Both `Q` and `B` to quit and take me back wouldn't work. Somehow, it kept throwing me back into `w3m`, so I had to forcibly shutdown.

In my second attempt, I also forgot to configure network, so I had no way to download `wpa_supplicant` without having `wpa_supplicant` itself to connect my laptop to the network! So, it was important to do that through the USB before installing it (which comes with the program preinstalled), just by making sure to add it in the configuration file.

Some other notes, if you can't reach the internet, don't add any DNS in `/etc/resolv.conf`. This should be done declaratively in `configuration.nix`.

In the end, I spent the entire afternoon playing around with it, taking breaks and what-not. I still haven't figured out why `nvim` was printing the literal escape character when going from normal to insert mode in the `xfce4-terminal` (and other actions also made it print this "garbage" to the console), why sometimes the network can reach the internet (and only some sites!) and sometimes not, and how to setup dualboot.

But despite all of this, I think it was a worth installing it again. One sure sees things from a different perspective, and gets the chance to write another blog post!

If there's something I overlooked or that could be done better, or maybe you can explain it differently, please be sure to [contact me](https://lonami.dev/contact) to let me know!

Update
------

Well, that was surprisingly fast feedback. Thank you very much [@bb010g](https://bb010g.keybase.pub/) for it! As they rightfully pointed out, one can avoid adding `/home` manually to `hardware-configuration.nix` if you mount it before generating the configuration files. However, the installation process doesn't need `/home` mounted, so I didn't do it.

The second weird issue with `w3m` is actually a funny one. `Alt+F8` *switches to another TTY*! That's why quitting the program wouldn't do anything. You'd still be in a different TTY! Normally, this is `Ctrl+Alt+FX`, so I hadn't even thought that this is what could be happening. Anyway, the solution is not quitting the program, but rather going back to the main TTY with `Alt+F1`. You can switch back and forth all you need to consult the manual.

More suggestions are having [`home-manager`](https://github.com/rycee/home-manager) manage the graphical sessions, since it should be easier to deal with than the alternatives.

Despite having followed the guide and having read it over and over several times, it seems like my thoughts in this blog post may be a bit messy. So I recommend you also reading through the guide to have two versions of all this, just in case.

Regarding network issues, they use `connman` so that may be worth checking out.

Regarding terminal issues with `nvim` printing the literal escape character, I was told off for not having checked what my `$TERM` was. I hadn't really looked into it much myself, just complained about it here, so sorry for being annoying about that. A quick search in the `nixpkgs` repository lets us find [neovim/default.nix](https://github.com/NixOS/nixpkgs/blob/release-18.09/pkgs/applications/editors/neovim/default.nix), with version 0.3.1. Looking at [Neovim's main repository](https://github.com/neovim/neovim) we can see that this is a bit outdated, but that is fine.

If only I had bothered to look at [Neovim's wiki](https://github.com/neovim/neovim/wiki/FAQ#nvim-shows-weird-symbols-2-q-when-changing-modes), (which they found through [Neovim's GitHub issues](https://github.com/neovim/neovim/issues/7749)) I would've seen that some terminals just don't support the program properly. The solution is, of course, to use a different terminal emulator with better support or to disable the `guicursor` in Neovim's config.

This is a pretty good life lesson. 30 seconds of searching, maybe two minutes and a half for also checking XFCE issues, are often more than enough to troubleshoot your issues. The internet is a big place and more people have surely came across the problem before, so make sure to look online first. In my defense I'll say that it didn't bother me so much so I didn't bother looking for that soon either.