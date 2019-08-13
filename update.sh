#!/bin/sh
cargo build --release
cp -f target/release/ffxiv-materia-calc .
