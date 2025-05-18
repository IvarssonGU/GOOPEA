#!/bin/bash

for dir in FIP NOFIP; do
  lower_dir="${dir,,}"                     # Lowercase directory name
  out_c_dir="reverse_${lower_dir}"        # Where .c files go
  out_bin_dir="output/${out_c_dir}"       # Where binaries go

  mkdir -p "$out_c_dir" "$out_bin_dir"    # Ensure output directories exist

  for file in "$dir"/*; do
    base=$(basename "$file")
    c_file="${out_c_dir}/${base}.c"
    bin_file="${out_bin_dir}/${base}"

    # Run cargo to generate .c file
    cargo run -- -f "$file" > "$c_file"
    echo "Generated C file: $file -> $c_file"

    # Compile the .c file with gcc

    gcc -Wno-int-conversion "$c_file" -o "$bin_file"
    echo "Compiled binary: $c_file -> $bin_file"
  done
done
