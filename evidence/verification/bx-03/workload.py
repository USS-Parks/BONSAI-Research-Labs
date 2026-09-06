"""Bounded live BX-03 workloads; all activity stays inside the attached cgroup."""
import errno
import json
import os
import sys
import time


def expire():
    time.sleep(12)
    os._exit(0)


mode = sys.argv[1]
if mode == "tree":
    read_fd, write_fd = os.pipe()
    child = os.fork()
    if child == 0:
        grandchild = os.fork()
        if grandchild == 0:
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                pass
            expire()
        os.write(write_fd, str(grandchild).encode())
        expire()
    os.close(write_fd)
    grandchild = int(os.read(read_fd, 32))
    print(json.dumps({"pids": [os.getpid(), child, grandchild]}), flush=True)
    expire()
elif mode == "memory":
    read_fd, write_fd = os.pipe()
    child = os.fork()
    if child == 0:
        os.close(write_fd)
        os.read(read_fd, 1)
        allocation = bytearray(128 * 1024 * 1024)
        allocation[0] = 1
        expire()
    os.close(read_fd)
    print(json.dumps({"pids": [os.getpid(), child]}), flush=True)
    sys.stdin.readline()
    os.write(write_fd, b"1")
    expire()
elif mode == "pids":
    children = []
    denied = False
    for _ in range(12):
        try:
            child = os.fork()
        except OSError as error:
            if error.errno != errno.EAGAIN:
                raise
            denied = True
            break
        if child == 0:
            expire()
        children.append(child)
    print(json.dumps({"pids": [os.getpid(), *children], "fork_denied": denied}), flush=True)
    expire()
else:
    raise ValueError(mode)
