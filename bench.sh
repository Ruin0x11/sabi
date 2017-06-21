#!/bin/bash
list=( target/release/sabi-* )
target="${list[0]}"
valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --simulate-cache=yes "$target" --ignored --bench "$*"
