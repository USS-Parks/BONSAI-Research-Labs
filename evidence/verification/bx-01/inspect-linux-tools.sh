#!/bin/sh
find /sys/fs/cgroup -maxdepth 8 -name cgroup.procs -writable 2>/dev/null | head -20
ls -ld /home/basho/.rustup /home/basho/.cargo /usr/bin/cargo /usr/bin/rustc /usr/local/cargo/bin/cargo 2>/dev/null
env XDG_RUNTIME_DIR=/run/user/1000 systemctl --user show --property=ControlGroup --property=Delegate --property=ActiveState
