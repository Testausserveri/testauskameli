#!/bin/sh
# This generates the install-dir containing all the runners

mkdir install-dir
find bin -type f -execdir ln -sr '{}' "../install-dir/{}-runner" \;
printf 'now run "export PATH=$PATH:$(pwd)/install-dir"\n'
