#!/usr/bin/bash

if [ $# -lt 2 ]; then
    echo "./$0 <appid> <[dev|prod]>"
    exit 1
fi

APPID=$1
STAGE=$2

cargo lambda build --release
if [ $? -eq 0 ]; then
  cargo lambda deploy --profile icci --binary-name $APPID-api $APPID-$STAGE
fi
