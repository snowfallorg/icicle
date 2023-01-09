#!/usr/bin/env bash
export INPUT="$1"
export OUTPUT="$2"
sed 's/\(gettext\|_\)("\(.*\)")/\2/g' $INPUT > $OUTPUT
