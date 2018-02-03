#!/bin/bash

rm -rf blog
cp -r content /dev/shm
pushd /dev/shm

    mv content/{ribw,mdad} .
    cp content/style.css ribw
    cp content/style.css mdad

    pagong
    mv dist blog
    rm -r content

    mv ribw content
    pagong
    mv dist blog/ribw
    rm -r content

    mv mdad content
    pagong
    mv dist blog/mdad
    rm -r content

popd
mv /dev/shm/blog .
