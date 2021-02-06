+++
title = "My new computer"
date = 2020-06-19
updated = 2020-07-03
+++

This post will be mostly me ranting about setting up a new laptop, but I also just want to share my upgrade. If you're considering installing Arch Linux with dual-boot for Windows, maybe this post will help. Or perhaps you will learn something new to troubleshoot systems in the future. Let's begin!

Last Sunday, I ordered a Asus Rog Strix G531GT-BQ165 for 900€ (on a 20% discount) with the following specifications:

* Intel® Core i7-9750H (6 cores, 12MB cache, 2.6GHz up to 4.5GHz, 64-bit)
* 16GB RAM (8GB*2) DDR4 2666MHz
* 512GB SSD M.2 PCIe® NVMe
* Display 15.6" (1920x1080/16:9) 60Hz
* Graphics NVIDIA® GeForce® GTX1650 4GB GDDR5 VRAM
* LAN 10/100/1000
* Wi-Fi 5 (802.11ac) 2x2 RangeBoost
* Bluetooth 5.0
* 48Wh battery with 3 cells
* 3 x USB 3.1 (GEN1)

I was mostly interested in a general upgrade (better processor, disk, more RAM), although the graphics card is a really nice addition which will allow me to take some time off on more games. After using it for a bit, I really love the feel of the keyboard, and I love the lack of numpad! (No sarcasm, I really don't like numpads.)

This is an upgrade from my previous laptop (Asus X554LA-XX822T), which I won in a competition before entering university in a programming challenge. It has served me really well for the past five years, and had the following specifications:

* Intel® Core™ i5-5200U
* 4GB RAM DDR3L 1600MHz (which I upgraded to have 8GB)
* 1TB HDD
* Display 15.6" (1366x768/16:9)
* Intel® HD Graphics 4400
* LAN 10/100/1000
* Wifi 802.11 bgn
* Bluetooth 4.0
* Battery 2 cells
* 1 x USB 2.0
* 2 x USB 3.0

Prior to this one, I had a Lenovo (also won in the same competition of the previous year), and prior to that (just for the sake of history), it was HP Pavilion, AMD A4-3300M processor, which unfortunately ended with heating problems. But that's very old now.

## Laptop arrival

The laptop arrived 2 days ago at roughly 19:00, which I put charged for 3 hours as the book said. The day after, nightmares began!

Trying to boot it the first two times was fun, as it comes with a somewhat loud sound on boot. I don't know why they would do this, and I immediately turned it off in the BIOS.

## Installation journey

I spent all of yesterday trying to setup Windows and Arch Linux (and didn't even finish, it took me this morning too and even now it's only half functional). I absolutely *hate* the amount of partitions the Windows installer creates on a clean disk. So instead, I first went with Arch Linux, and followed the [installation guide on the Arch wiki](https://wiki.archlinux.org/index.php/Installation_guide). Pre-installation, setting up the wireless network, creating the partitions and formatting them went all good. I decided to avoid GRUB at first and go with rEFInd, but alas I missed a big warning on the wiki and after reboot (I would later find out) it was not mounting root properly, so all I had was whatever was in the Initramfs. Reboot didn't work, so I had to hold the power button.

Anyway, once the partitions were created, I went to install Windows (there was a lot of back and forth burning different `.iso` images on the USB, which was a bit annoying because it wasn't the fastest thing in the world). This was pretty painless, and the process was standard: select advanced to let me choose the right partition, pick the one, say "no" to everything in the services setup, and done. But this was the first Windows `.iso` I tried. It was an old revision, and the drivers were causing issues when running (something weird about their `.dll`, manually installing the `.ini` driver files seemed to work?). The Nvidia drivers didn't want to be installed on such an old revision, after updating everything I could via Windows updates. So back I went to burning a newer Windows `.iso` and going through the same process again…

Once Windows was ready and I verified that I could boot to it correctly, it was time to have a second go at Arch Linux. And I went through the setup at least three times, getting it wrong every single time, formatting root every single time, redownloading the packages every single pain. If only had I known earlier what the issue was!

Why bother with Arch? I was pretty happy with Linux Mint, and I lowkey wanted to try NixOS, but I had used Arch before and it's a really nice distro overall (up-to-date, has AUR, quite minimal, imperative), except for trying to install rEFInd while chrooted…

In the end I managed to get something half-working, I still need to properly configure WiFi and pulseaudio in my system but hey it works.

I like to be able to dual-boot Windows and Linux because Linux is amazing for productivity, but unfortunately, some games only work fine on Windows. Might as well have both systems and use one for gaming, while the other is my daily driver.

## Setting up Arch Linux

This is the process I followed to install Arch Linux in the end, along with a brief explanation on what I think the things are doing and why we are doing them. I think the wiki could do a better job at this, but I also know it's hard to get it right for everyone. Something I do dislike is the link colour, after opening a link it becomes gray and it's a lot easier to miss the fact that it is a link in the first place, which was tough when re-reading it because some links actually matter a lot. Furthermore, important information may just be a single line, also easy to skim over. Anyway, on to the installation process…

The first thing we want to do is configure our keyboard layout or else the keys won't correspond to what we expect:

```sh
loadkeys es
```

Because we're on a recent system, we want to verify that UEFI works correctly. If we see files listed, then it works fine:

```sh
ls /sys/firmware/efi/efivars
```

The next thing we want to do is configure the WiFi, because I don't have any ethernet cable nearby. To do this, we check what network interfaces our laptop has (we're looking for the one prefixed with "w", presumably for wireless, such as "wlan0" or "wlo1"), we set it up, scan for available wireless network, and finally connect. In my case, the network has WPA security so we rely on `wpa_supplicant` to connect, passing the SSID (network name) and password:

```sh
ip link
ip link set <IFACE> up
iw dev <IFACE> scan | less
wpa_supplicant -B -i <IFACE> -c <(wpa_passphrase <SSID> <PASS>)
```

After that's done, pinging an IP address like "1.1.1.1" should Just Work™, but to be able to resolve hostnames, we need to also setup a nameserver. I'm using Cloudflare's, but you could use any other:

```sh
echo nameserver 1.1.1.1 > /etc/resolv.conf
ping archlinux.org
^C
```

If the ping works, then network works! If you still have issues, you may need to [manually configure a static IP address](https://wiki.archlinux.org/index.php/Network_configuration#Static_IP_address) and add a route with the address of your, well, router. This basically shows if we have any address, adds a static address (so people know who we are), shows what route we have, and adds a default one (so our packets know where to go):

```sh
ip address show
ip address add <YOUR ADDR>/24 broadcast + dev <IFACE>
ip route show
ip route add default via <ROUTER ADDR> dev <IFACE>
```

Now that we have network available, we can enable NTP to synchronize our system time (this may be required for network operations where certificates have a validity period, not sure; in any case nobody wants a wrong system time):

```sh
timedatectl set-ntp true
```

After that, we can manage our disk and partitions using `fdisk`. We want to define partitions to tell the system where it should live. To determine the disk name, we first list them, and then edit it. `fdisk` is really nice and reminds you at every step that help can be accessed with "m", which you should constantly use to guide you through.

```sh
fdisk -l
fdisk /dev/<DISK>
```

The partitions I made are the following:

* A 100MB one for the EFI system.
* A 32GB one for Linux' root `/` partition.
* A 200GB one for Linux' home `/home` partition.
* The rest was unallocated for Windows because I did this first.

I like to have `/home` and `/` separate because I can reinstall root without losing anything from home (projects, music, photos, screenshots, videos…).

After the partitions are made, we format them in FAT32 and EXT4 which are good defaults for EFI, root and home. They need to have a format, or else they won't be usable:

```sh
mkfs.fat -F32 /dev/<DISK><PART1>
mkfs.ext4 /dev/<DISK><PART2>
mkfs.ext4 /dev/<DISK><PART3>
```

Because the laptop was new, there was no risk to lose anything, but if you're doing a install on a previous system, be very careful with the partition names. Make sure they match with the ones in `fdisk -l`.

Now that we have usable partitions, we need to mount them or they won't be accessible. We can do this with `mount`:

```sh
mount /dev/<DISK><PART2> /mnt
mkdir /mnt/efi
mount /dev/<DISK><PART1> /mnt/efi
mkdir /mnt/home
mount /dev/<DISK><PART3> /mnt/home
```

Remember to use the correct partitions while mounting. We mount everything so that the system knows which partitions we care about, which we will let know about later on.

Next step is to setup the basic Arch Linux system on root, which can be done with `pacstrap`. What follows the directory is a list of packages, and you may choose any you wish (at least add `base`, `linux` and `linux-firmware`). These can be installed later, but I'd recommend having them from the beginning, just in case:

```sh
pacstrap /mnt base linux linux-firmware sudo vim-minimal dhcpcd wpa_supplicant man-db man-pages intel-ucode grub efibootmgr os-prober ntfs-3g
```

Because my system has an intel CPU, I also installed `intel-ucode`.

Next up is generating the `fstab` file, which we tell to use UUIDs to be on the safe side through `-U`. This file is important, because without it the system won't know what partitions exist and will happily only boot with the initramfs, without anything of what we just installed at root. Not knowing this made me restart the entire installation process a few times.

```sh
genfstab -U /mnt >> /mnt/etc/fstab
```

After that's done, we can change our root into our mount point and finish up configuration. We setup our timezone (so DST can be handled correctly if needed), synchronize the hardware clock (to persist the current time to the BIOS), uncomment our locales (exit `vim` by pressing ESC, then type `:wq` and press enter), generate locale files (which some applications need), configure language and keymap, update the hostname of our laptop and what indicate what `localhost` means…

```sh
ln -sf /usr/share/zoneinfo/<REGION>/<CITY> /etc/localtime
hwclock --systohc
vim /etc/locale.gen
locale-gen
echo LANG=es_ES.UTF-8 > /etc/locale.conf
echo KEYMAP=es > /etc/vconsole.conf
echo <HOST> /etc/hostname
cat <<EOF > /etc/hosts
127.0.0.1 localhost
::1 localhost
127.0.1.1 <HOST>.localdomain <HOST>
EOF
```

Really, we could've done all of this later, and the same goes for setting root's password with `passwd` or creating users (some of the groups you probably want are `power` and `wheel`).

The important part here is installing GRUB (which also needed the `efibootmgr` package):

```sh
grub-install --target=x86_64-efi --efi-directory=/efi --bootloader-id=GRUB
```

If we want GRUB to find our Windows install, we also need the `os-prober` and `ntfs-3g` packages that we installed earlier with `pacstrap`, and with those we need to mount the Windows partition somewhere. It doesn't matter where. With that done, we can generate the GRUB configuration file which lists all the boot options:

```sh
mkdir /windows
mount /dev/<DISK><PART5> /windows
grub-mkconfig -o /boot/grub/grub.cfg
```

(In my case, I installed Windows before completing the Arch install, which created an additional partition in between).

With GRUB ready, we can exit the chroot and reboot the system, and if all went well, you should be greeted with a choice of operating system to use:

```sh
exit
reboot
```

If for some reason you need to find what mountpoints were active prior to rebooting (to `unmount` them for example), you can use `findmnt`.

Before GRUB I tried rEFInd, which as I explained had issues with for missing a warning. Then I tried systemd-boot, which did not pick up Arch at first. That's where the several reinstalls come from, I didn't want to work with a half-worked system so I mostly redid the entire process quite a few times.

## Migrating to the new laptop

I had a external disk formatted with NTFS. Of course, after moving every file I cared about from my previous Linux install caused all the permissions to reset. All my `.git` repositories, dirty with file permission changes! This is going to take a while to fix, or maybe I should just `git config core.fileMode false`. Here is a [lovely command](https://stackoverflow.com/a/2083563) to sort them out on a per-repository basis:

```sh
git diff --summary | grep --color 'mode change 100644 => 100755' | cut -d' ' -f7- | xargs -d'\n' chmod -x
```

I never realized how much I had stored over the years, but it really was a lot. While moving things to the external disk, I tried to do some cleanup, such as removing some build artifacts which needlessly occupy space, or completely skipping all the binary application files. If I need those I will install them anyway. The process was mostly focused on finding all the projects and program data that I did care about, or even some game saves. Nothing too difficult, but definitely time consuming.

## Tuning Arch

Now that our system is ready, install `pacman-contrib` to grab a copy of the `rankmirrors` speed. It should help speed up the download of whatever packages you want to install, since it will help us [rank the mirrors by download speed](https://wiki.archlinux.org/index.php/Mirrors#List_by_speed). Making a copy of the file is important, otherwise whenever you try to install something it will fail saying it can't find anything.

```sh
cp /etc/pacman.d/mirrorlist /etc/pacman.d/mirrorlist.backup
sed -i 's/^#Server/Server/' /etc/pacman.d/mirrorlist.backup
rankmirrors -n 6 /etc/pacman.d/mirrorlist.backup | tee /etc/pacman.d/mirrorlist
```

This will take a while, but it should be well worth it. We're using `tee` to see the progress as it goes.

Some other packages I installed after I had a working system in no particular order:

* `xfce4` and `xorg-server`. I just love the simplicity of XFCE.
* `xfce4-whiskermenu-plugin`, a really nice start menu.
* `xfce4-pulseaudio-plugin` and `pavucontrol`, to quickly adjust the audio with my mouse.
* `xfce4-taskmanager`, a GUI alternative I generally prefer to `htop`.
* `pulseaudio` and `pulseaudio-alsa` to get nice integration with XFCE4 and audio mixing.
* `firefox`, which comes with fonts too. A really good web browser.
* `git`, to commit ~~crimes~~ code.
* `code`, a wonderful editor which I used to write this blog entry.
* `nano`, so much nicer to write a simple commit message.
* `python` and `python-pip`, my favourite language to toy around ideas or use as a calculator.
* `telegram-desktop`, for my needs on sharing memes.
* `cmus` and `mpv`, a simple terminal music player and media player.
* `openssh`, to connect into any VPS I have access to.
* `base-devel`, necessary to build most projects I'll find myself working with (or even compiling some projects Rust which I installed via `rustup`).
* `flac`, `libmad`, `opus`, and `libvorbis`, to be able to play more audio files.
* `inkscape`, to make random drawings.
* `ffmpeg`, to convert media or record screen.
* `xclip`, to automatically copy screenshots to my clipboard.
* `gvfs`, needed by Thunar to handle mounting and having a trash (perma-deletion by default can be nasty sometimes).
* `noto-fonts`, `noto-fonts-cjk`, `noto-fonts-extra` and `noto-fonts-emoji`, if you don't want missing gliphs everywhere.
* `xfce4-notifyd` and `libnotify`, for notifications.
* `cronie`, to be able to `crontab -e`. Make sure to `system enable cronie`.
* `xarchiver` (with `p7zip`, `zip`, `unzip` and `unrar`) to uncompress stuff.
* `xreader` to read `.pdf` files.
* `sqlitebrowser` is always nice to tinker around with SQLite databases.
* `jre8-openjdk` if you want to run Java applications.
* `smartmontools` is nice with a SSD to view your disk statistics.

After that, I configured my Super L key to launch `xfce4-popup-whiskermenu` so that it opens the application menu, pretty much the same as it would on Windows, moved the panels around and configured them to my needs, and it feels like home once more.

I made some mistakes while [configuring systemd-networkd](https://wiki.archlinux.org/index.php/Systemd-networkd) and accidentally added a service that was incorrect, which caused boot to wait for it to timeout before completing. My boot time was taking 90 seconds longer because of this! [The solution was to remove said service](https://www.reddit.com/r/archlinux/comments/4nv9yi/my_arch_greets_me_now_with_a_start_job/), so this is something to look out for.

In order to find what was taking long, I had to edit the [kernel parameters](https://wiki.archlinux.org/index.php/kernel_parameters) to remove the `quiet` option. I prefer seeing the output on what my computer is doing anyway, because it gives me a sense of progress and most importantly is of great value when things go wrong. Another interesting option is `noauto,x-systemd.automount`, which makes a disk lazily-mounted. If you have a slow disk, this could help speed things up.

If you see a service taking long, you can also use `systemd-analyze blame` to see what takes the longest, and `systemctl list-dependencies` is also helpful to find what services are active.

My `locale charmap` was spitting out a bunch of warnings:

```sh
$ locale charmap
locale: Cannot set LC_CTYPE to default locale: No such file or directory
locale: Cannot set LC_MESSAGES to default locale: No such file or directory
locale: Cannot set LC_ALL to default locale: No such file or directory
ANSI_X3.4-1968
```

…ANSI encoding? Immediately I added the following to `~/.bashrc` and `~/.profile`:

```sh
export LC_ALL=en_US.UTF-8
export LANG=en_US.UTF-8
export LANGUAGE=en_US.UTF-8
```

For some reason, I also had to edit `xfce4-terminal`'s preferences in advanced to change the default character encoding to UTF-8. This also solved my issues with pasting things into the terminal, and also proper rendering! I guess pastes were not working because it had some characters that could not be encoded.

To have working notifications, I added the following to `~/.bash_profile` after `exec startx`:

```sh
systemctl --user start xfce4-notifyd.service
```

I'm pretty sure there's a better way to do this, or maybe it's not even necessary, but this works for me.

Some of the other things I had left to do was setting up `sccache` to speed up Rust builds:

```sh
cargo install sccache
echo export RUSTC_WRAPPER=sccache >> ~/.bashrc
```

Once I had `cargo` ready, installed `hacksaw` and `shotgun` with it to perform screenshots.

I also disabled the security delay when downloading files in Firefox because it's just annoying, in `about:config` setting `security.dialog_enable_delay` to `0`, and added the [Kill sticky headers](https://alisdair.mcdiarmid.org/kill-sticky-headers/) to my bookmarks (you may prefer [the updated version](https://github.com/t-mart/kill-sticky)).

The `utils-linux` comes with a `fstrim` utility to [trim the SSD weekly](https://wiki.archlinux.org/index.php/Solid_state_drive#Periodic_TRIM), which I want enabled via `systemctl enable fstrim.timer` (you may also want to `start` it if you don't reboot often). For more SSD tips, check [How to optimize your Solid State Drive](https://easylinuxtipsproject.blogspot.com/p/ssd.html).

If the sound is funky prior to reboot, try `pulseaudio --kill` and `pulseaudio --start`, or delete `~/.config/pulse`.

I haven't been able to get the brightness keys to work yet, but it's not a big deal, because scrolling on the power manager plugin of Xfce does work (and also `xbacklight` works, or writing directly to `/sys/class/backlight/*`).

## Tuning Windows

On the Windows side, I disabled the annoying Windows defender by running (<kbd>Ctrl+R</kbd>) `gpedit.msc` and editing:

* *Computer Configuration > Administrative Templates > Windows Components > Windows Defender » Turn off Windows Defender » Enable*
* *User Configuration > Administrative Templates > Start Menu and Taskbar » Remove Notifications and Action Center » Enable*

I also updated the [`hosts` file](https://github.com/WindowsLies/BlockWindows/raw/master/hosts) (located at `%windir%\system32\Drivers\etc\hosts`) with the hope that it will stop some of the telemetry.

Last, to have consistent time on Windows and Linux, I changed the following registry key for a `qword` with value `1`:

```
HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\TimeZoneInformation\RealTimeIsUniversal
```

(The key might not exist, but you can create it if that's the case).

All this time, my laptop had the keyboard lights on, which have been quite annoying. Apparently, they also can cause [massive FPS drops](https://www.reddit.com/r/ValveIndex/comments/cm6pos/psa_uninstalldisable_aura_sync_lighting_if_you/). I headed over to [Asus Rog downloads](https://rog.asus.com/downloads/), selected Aura Sync…

```md
# Not Found

The requested URL /campaign/aura/us/Sync.html was not found on this server.

Additionally, a 404 Not Found error was encountered while trying to use an ErrorDocument to handle the request.
```

…great! I'll just find the [Aura site](https://www.asus.com/campaign/aura/global/) somewhere else…

```md
# ASUS

# We'll be back.

Hi, our website is temporarily closed for service enhancements.

We'll be back shortly.Thank you for your patience!
```

Oh come on. After waiting for the next day, I headed over, downloaded their software, tried to install it and it was an awful experience. It felt like I was purposedly installing malware. It spammed and flashed a lot of `cmd`'s on screen as if it was a virus. It was stuck at 100% doing that and then, Windows blue-screened with `KERNEL_MODE_HEAP_CORRUPTION`. Amazing. How do you screw up this bad?

Well, at least rebooting worked. I tried to [uninstall Aura, but of course that failed](https://answers.microsoft.com/en-us/windows/forum/all/unable-to-uninstall-asus-aura-sync-utility/e9bec36c-e62f-4773-80be-88fb68dace16). Using the [troubleshooter to uninstall programs](https://support.microsoft.com/en-us/help/17588/windows-fix-problems-that-block-programs-being-installed-or-removed) helped me remove most of the crap that was installed.

After searching around how to disable the lights (because [my BIOS did not have this setting](https://rog.asus.com/forum/showthread.php?112786-Option-to-Disable-Aura-Lights-on-Strix-G-series-(G531GT)-irrespective-of-OSes)), I stumbled upon ["Armoury Crate"](https://rog.asus.com/us/innovation/armoury_crate/). Okay, fine, I will install that.

The experience wasn't much better. It did the same thing with a lot of consoles flashing on screen. And of course, it resulted in another blue-screen, this time `KERNEL_SECURITY_CHECK_FAILURE`. To finish up, the BSOD kept happening as I rebooted the system. ~~Time to reinstall Windows once more.~~ After booting and crashing a few more times I could get into secure mode and perform the reinstall from there, which saved me from burning the `.iso` again.

Asus software might be good, but the software is utter crap.

After trying out [rogauracore](https://github.com/wroberts/rogauracore) (which didn't list my model), it worked! I could disable the stupid lights from Linux, and [OpenRGB](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/home) also works on Windows which may be worth checking out too.

Because `rougauracore` helped me and they linked to [hw-probe](https://github.com/linuxhw/hw-probe/blob/master/README.md#appimage), I decided to [run it on my system](https://linux-hardware.org/?probe=0e3e48c501), with the hopes it is useful for other people.

## Closing words

I hope the installation journey is at least useful to someone, or that you enjoyed reading about it all. If not, sorry!