#!/bin/sh
uname -srmo
id
command -v cargo
command -v uv
df -h / /mnt/c
cat /proc/self/cgroup
cat /sys/fs/cgroup/cgroup.controllers
ls -ld /sys/fs/cgroup
find /sys/fs/cgroup -maxdepth 3 -name cgroup.procs -writable 2>/dev/null | head -20
ls -l /home/basho/.cargo/bin/cargo /home/basho/.local/bin/uv 2>/dev/null
ls -ld /run/user/1000 /run/user/1000/bus 2>/dev/null
