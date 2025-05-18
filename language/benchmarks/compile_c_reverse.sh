#!/bin/bash

for src_dir in reverse_fip reverse_nofip; do
  out_dir="output/$src_dir"
  mkdir -p "$out_dir"

  for c_file in "$src_dir"/*.c; do
    base=$(basename "$c_file" .c)
    out_bin="${out_dir}/${base}"

    gcc -Wno-int-conversion "$c_file" -o "$out_bin"

    echo "Compiled $c_file -> $out_bin"
  done
done
