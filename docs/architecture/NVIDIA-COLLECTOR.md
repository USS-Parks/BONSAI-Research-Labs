# NVIDIA accelerator collector

BM-11 feature-detects NVML or `nvidia-smi`. States are supported, not-supported, and no-permission. Absence never breaks CPU-only collection. Board-level energy is not treated as process-exclusive. AMD/Intel discrete collectors remain parked (P-04).
