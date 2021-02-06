```meta
created: 2017-05-13
updated: 2019-02-16
```

Installing NixOS
================

Update
------

*Please see [my followup post with NixOS](../installing_nixos_2/index.html) for a far better experience with it*

----------------------------------------

Today I decided to install [NixOS](http://nixos.org/) as a recommendation, a purely functional Linux distribution, since [Xubuntu](https://xubuntu.org/) kept crashing. Here's my journey, and how I managed to install it from a terminal for the first time in my life. Steps aren't hard, but they may not seem obvious at first.

* Grab the Live CD, burn it on a USB stick and boot. I recommend using [Etcher](https://etcher.io/).
* Type `systemctl start display-manager` and wait.[^1]
* Open both the manual and the `konsole`.
* Connect to the network using the GUI.
* Create the disk partitions by using `fdisk`.

  You can list them with `fdisk -l`, modify a certain drive with `fdisk /dev/sdX` (for instance, `/dev/sda`) and follow the instructions.

  To create the file system, use `mkfs.ext4 -L <label> /dev/sdXY` and swap with `mkswap -L <label> /dev/sdXY`.

  The EFI partition should be done with `mkfs.vfat`.

* Mount the target to `/mnt` e.g. if the label was `nixos`, `mount /dev/disk/by-label/nixos /mnt`
* `mkdir /mnt/boot` and then mount your EFI partition to it.
* Generate a configuration template with `nixos-generate-config --root /mnt`, and modify it with `nano /etc/nixos/configuration.nix`.
* While modifying the configuration, make sure to add `boot.loader.grub.device = "/dev/sda"`
* More useful configuration things are:
  * Uncomment the whole `i18n` block.
  * Add some essential packages like `environment.systemPackages = with pkgs; [wget git firefox pulseaudio networkmanagerapplet];`.
  * If you want to use XFCE, add `services.xserver.desktopManager.xfce.enable = true;`, otherwise, you don't need `networkmanagerapplet` either. Make sure to add `networking.networkmanager.enable = true;` too.
  * Define some user for yourself (modify `guest` name) and use a UID greater than 1000. Also, add yourself to `extraGroups = ["wheel" "networkmanager"];` (the first to be able to `sudo`, the second to use network related things).

* Run `nixos-install`. If you ever modify that file again, to add more packages for instance (this is how they're installed), run `nixos-rebuild switch` (or use `test` to test but don't boot to it, or `boot` not to switch but to use on next boot.
* `reboot`.
* Login as `root`, and set a password for your user with `passwd <user>`. Done!

I enjoyed the process of installing it, and it's really cool that it has versioning and is so clean to keep track of which packages you install. But not being able to run arbitrary binaries by default is something very limitting in my opinion, though they've done a good job.

I'm now back to Xubuntu, with a fresh install.

Update
------

It is not true that "they don't allow running arbitrary binaries by default", as pointed out in their [manual, buildFHSUserEnv](https://nixos.org/nixpkgs/manual/#sec-fhs-environments):

> `buildFHSUserEnv` provides a way to build and run FHS-compatible lightweight sandboxes. It creates an isolated root with bound `/nix/store`, so its footprint in terms of disk space needed is quite small. This allows one to run software which is hard or unfeasible to patch for NixOS -- 3rd-party source trees with FHS assumptions, games distributed as tarballs, software with integrity checking and/or external self-updated binaries. It uses Linux namespaces feature to create temporary lightweight environments which are destroyed after all child processes exit, without root user rights requirement.

Thanks to [@bb010g](https://github.com/bb010g) for pointing this out.

Notes
-----

[^1]: The keyboard mapping is a bit strange. On my Spanish keyboard, the keys were as follows:

|Keyboard|Maps to|Shift
|---|---|---|
|'|-|_|
|´|'|"|
|`|[| |
|+|]| |
|¡|=| |
|-|/| |
|ñ|;| |
