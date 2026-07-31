#!/usr/bin/bash

upver

cargo lambda build --release
if [ $? -eq 0 ]; then
  cargo lambda deploy
fi
