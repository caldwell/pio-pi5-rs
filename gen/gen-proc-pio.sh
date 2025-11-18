#!/bin/bash

set -x
curl -s -L https://github.com/raspberrypi/utils/raw/refs/heads/master/piolib/include/hardware/regs/proc_pio.h \
    | "$(dirname $0)"/gen-proc-pio.rb > src/proc-pio.rs.new
mv src/proc-pio.rs{.new,}
