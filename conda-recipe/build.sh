#! /bin/sh
#
# build.sh
# Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
#
# Distributed under terms of the MIT license.
#

C_INCLUDE_PATH=$PREFIX/include LIBRARY_PATH=$PREFIX/lib cargo install --path . --root $PREFIX
