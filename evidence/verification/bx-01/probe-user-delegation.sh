#!/bin/sh
export XDG_RUNTIME_DIR=/run/user/1000
systemd-run --user --wait --pipe --collect --property=Delegate=yes --unit=bonsai-bx01-probe /bin/sh -c 'cat /proc/self/cgroup; p=$(sed -n "s/^0:://p" /proc/self/cgroup); ls -ld "/sys/fs/cgroup$p"; cat "/sys/fs/cgroup$p/cgroup.controllers"; test -w "/sys/fs/cgroup$p/cgroup.procs" && echo user-delegation-writable'
