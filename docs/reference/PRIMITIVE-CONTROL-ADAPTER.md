# Primitive tabular control adapter

BE-02 implements a minimal sample-average tabular controller for primitive discrete actions. It consumes one observation, chooses one action, consumes one reward, updates exactly one state-action statistic, and discards the transition. Initial exploration and mean-return comparison are deterministic integer operations.

The certification fixes batch size one, replay capacity zero, primitive actions only, and no learned features, options, planning, or privileged diagnostic input. Exact environment-step, update, parameter-touch, and work-item counts are external-accounting inputs. The diagnostic bandit learning curve is a protocol fixture, not a scientific comparator or C-level result.
