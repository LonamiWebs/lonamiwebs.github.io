#!/bin/bash

for f in *.svg
do
    svgcleaner $f ../$f
done
