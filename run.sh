#!/bin/bash

qemu-system-x86_64 -no-reboot -no-shutdown -serial stdio -drive format=raw,file=$HOME/study/myos/target/debug/build/myos-3b491c3e4241445c/out/os-bios.img