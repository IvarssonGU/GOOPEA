#!/bin/bash

# Benchmark reverse_fip binaries
fip_bins=(output/reverse_fip/*)
hyperfine "${fip_bins[@]/#/./}" --export-csv benchmark_fip.json
echo "Benchmark for FIP complete. Results saved to benchmark_fip.json"

# Benchmark reverse_nofip binaries
nofip_bins=(output/reverse_nofip/*)
hyperfine "${nofip_bins[@]/#/./}" --export-csv benchmark_nofip.json
echo "Benchmark for NOFIP complete. Results saved to benchmark_nofip.json"
