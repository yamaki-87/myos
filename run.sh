#!/bin/bash

qemu-system-x86_64 -no-reboot -no-shutdown -serial stdio -drive format=raw,file=$HOME/study/myos/target/debug/build/myos-5f2ca893d0b9eedb/out/os-bios.img