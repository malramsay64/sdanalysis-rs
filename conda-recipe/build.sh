#! /bin/sh
#
# build.sh
# Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
#
# Distributed under terms of the MIT license.
#


cargo build --release
cargo install --bin trajedy --path . --root $PREFIX
