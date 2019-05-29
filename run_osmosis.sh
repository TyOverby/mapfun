#!/bin/sh

PBF=$1
NAME=$2
TOP=$3
LEFT=$4
BOTTOM=$5
RIGHT=$6

osmosis --read-pbf file=$PBF --bounding-box top=$TOP left=$LEFT bottom=$BOTTOM right=$RIGHT --write-xml $NAME

