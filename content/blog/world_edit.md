+++
title = "WorldEdit Commands"
date = 2018-07-11
updated = 2018-07-11
+++

[WorldEdit](https://dev.bukkit.org/projects/worldedit) is an extremely powerful tool for modifying entire worlds within [Minecraft](https://minecraft.net), which can be used as either a mod for your single-player worlds or as a plugin for your [Bukkit](https://getbukkit.org/) servers.

This command guide was written for Minecraft 1.12.1, version [6.1.7.3](https://dev.bukkit.org/projects/worldedit/files/2460562), but should work for newer versions too. All WorldEdit commands can be used with a double slash (`//`) so they don't conlict with built-in commands. This means you can get a list of all commands with `//help`. Let's explore different categories!

Movement
--------

In order to edit a world properly you need to learn how to move in said world properly. There are several straightforward commands that let you move:

* `//ascend` goes up one floor.
* `//descend` goes down one floor.
* `//thru` let's you pass through walls.
* `//jumpto` to go wherever you are looking.

Information
-----------

Knowing your world properly is as important as knowing how to move within it, and will also let you change the information in said world if you need to.

* `//biomelist` shows all known biomes.
* `//biomeinfo` shows the current biome.
* `//setbiome` lets you change the biome.

Blocks
------

You can act over all blocks in a radius around you with quite a few commands. Some won't actually act over the entire range you specify, so 100 is often a good number.

### Filling

You can fill pools with `//fill water 100` or caves with `//fillr water 100`, both of which act below your feet.

### Fixing

If the water or lava is buggy use `//fixwater 100` or `//fixlava 100` respectively.

Some creeper removed the snow or the grass? Fear not, you can use `//snow 10` or `//grass 10`.

### Emptying

You can empty a pool completely with `//drain 100`, remove the snow with `//thaw 10`, and remove fire with `//ex 10`.

### Removing

You can remove blocks above and below you in some area with the `//removeabove N` and `//removebelow N`. You probably want to set a limit though, or you could fall off the world with `//removebelow 1 10` for radius and depth. You can also remove near blocks with `//removenear block 10`.

### Shapes

Making a cylinder (or circle) can be done with through `//cyl stone 10`, a third argument for the height. The radius can be comma-separated to make a ellipses instead, such as `//cyl stone 5,10`.

Spheres are done with `//sphere stone 5`. This will build one right at your center, so you can raise it to be on your feet with `//sphere stone 5 yes`. Similar to cylinders, you can comma separate the radius `x,y,z`.

Pyramids can be done with `//pyramic stone 5`.

All these commands can be prefixed with "h" to make them hollow. For instance, `//hsphere stone 10`.

Regions
-------

### Basics

Operating over an entire region is really important, and the first thing you need to work comfortably with them is a tool to make selections. The default wooden-axe tool can be obtained with `//wand`, but you must be near the blocks to select. You can use a different tool, like a golden axe, to use as your "far wand" (wand usable over distance). Once you have one in your hand type `//farwand` to use it as your "far wand". You can select the two corners of your region with left and right click. If you have selected the wrong tool, use `//none` to clear it.

If there are no blocks but you want to use your current position as a corner, use `//pos1` or 2.

If you made a region too small, you can enlarge it with `//expand 10 up`, or `//expand vert` for the entire vertical range, etc., or make it smaller with `//contract 10 up` etc., or `//inset` it to contract in both directions. You can use short-names for the cardinal directions (NSEW).

Finally, if you want to move your selection, you can `//shift 1 north` it to wherever you need.

### Information

You can get the `//size` of the selection or even `//count torch` in some area. If you want to count all blocks, get their distribution `//distr`.

### Filling

With a region selected, you can `//set` it to be any block! For instance, you can use `//set air` to clear it entirely. You can use more than one block evenly by separting them with a comma `//set stone,dirt`, or with a custom chance `//set 20%stone,80%dirt`.

You can use `//replace from to` instead if you don't want to override all blocks in your selection.

You can make an hollow set with `//faces`, and if you just want the walls, use `//walls`.

### Cleaning

If someone destroyed your wonderful snow landscape, fear not, you can use `//overlay snow` over it (although for this you actually have `//snow N` and its opposite `//thaw`).

If you set some rough area, you can always `//smooth` it, even more than one time with `//smooth 3`. You can get your dirt and stone back with `//naturalize` and put some plants with `//flora` or `//forest`, both of which support a density or even the type for the trees. If you already have the dirt use `//green` instead. If you want some pumpkins, with `//pumpkins`.

### Moving

You can repeat an entire selection many times by stacking them with `//stack N DIR`. This is extremely useful to make things like corridors or elevators. For instance, you can make a small section of the corridor, select it entirely, and then repeat it 10 times with `//stack 10 north`. Or you can make the elevator and then `//stack 10 up`. If you need to also copy the air use `//stackair`.

Finally, if you don't need to repeat it and simply move it just a bit towards the right direction, you can use `//move N`. The default direction is "me" (towards where you are facing) but you can set one with `//move 1 up` for example.

### Selecting

You can not only select cuboids. You can also select different shapes, or even just points:

* `//sel cuboid` is the default.
* `//sel extend` expands the default.
* `//sel poly` first point with left click and right click to add new points.
* `//sel ellipsoid` first point to select the center and right click to select the different radius.
* `//sel sphere` first point to select the center and one more right click for the radius.
* `//sel cyl` for cylinders, first click being the center.
* `//sel convex` for convex shapes. This one is extremely useful for `//curve`.

Brushes
-------

Brushes are a way to paint in 3D without first bothering about making a selection, and there are spherical and cylinder brushes with e.g. `//brush sphere stone 2`, or the shorter form `//br s stone`. For cylinder, one must use `cyl` instead `sphere`.

There also exists a brush to smooth the terrain which can be enabled on the current item with `//br smooth`, which can be used with right-click like any other brush.

Clipboard
---------

Finally, you can copy and cut things around like you would do with normal text with `//copy` and `//cut`. The copy is issued from wherever you issue the command, so when you use `//paste`, remember that if you were 4 blocks apart when copying, it will be 4 blocks apart when pasting.

The contents of the clipboard can be flipped to wherever you are looking via `//flip`, and can be rotated via the `//rotate 90` command (in degrees).

To remove the copy use `//clearclipboard`.