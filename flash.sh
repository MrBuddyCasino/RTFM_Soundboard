#!/usr/bin/env bash
#python ~/git/stm32loader/stm32loader.py -ev -p /dev/tty.usbserial-A402QLIM -w target/thumbv7m-none-eabi/release/rust-blue-pill
xargo build --release && arm-none-eabi-gdb target/thumbv7m-none-eabi/release/rust-blue-pill