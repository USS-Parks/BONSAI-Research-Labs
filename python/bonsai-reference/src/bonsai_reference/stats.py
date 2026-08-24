"""Independent integer statistical reference for BK-13.

Paired mean differences, deterministic bootstrap intervals, Holm adjustment,
and undeclared-exclusion detection. No post-hoc seed or metric selection.
"""

from __future__ import annotations

from dataclasses import dataclass, replace
from typing import Final

LCG_A: Final[int] = 1_664_525
LCG_C: Final[int] = 1_013_904_223
LCG_MASK: Final[int] = 0xFFFF_FFFF


class StatError(ValueError):
    """Stable statistical-plan failure."""

    def __init__(self, code: str) -> None:
        self.code = code
        super().__init__(code)


@dataclass(frozen=True, slots=True)
class SeedPair:
    pair_id: str
    outcome_a: int | None
    outcome_b: int | None
    failed_a: bool
    failed_b: bool


@dataclass(frozen=True, slots=True)
class OutcomeFamily:
    outcome_id: str
    pairs: tuple[SeedPair, ...]


@dataclass(frozen=True, slots=True)
class StatisticalPlan:
    preregistered_outcomes: tuple[str, ...]
    declared_pairs: tuple[str, ...]
    declared_exclusions: tuple[str, ...]
    bootstrap_resamples: int
    bootstrap_seed: int
    alpha_numerator: int
    alpha_denominator: int


@dataclass(frozen=True, slots=True)
class OutcomeRow:
    outcome_id: str
    complete_pairs: int
    missing_runs: int
    failed_runs: int
    excluded_runs: int
    effect_numerator: int | None
    effect_denominator: int | None
    ci_low_numerator: int | None
    ci_high_numerator: int | None
    ci_denominator: int | None
    p_numerator: int | None
    p_denominator: int | None
    holm_rejected: bool
    sensitivity_min_numerator: int | None
    sensitivity_max_numerator: int | None
    sensitivity_denominator: int | None
    detail_code: str | None


def aggregate(plan: StatisticalPlan, families: tuple[OutcomeFamily, ...]) -> tuple[OutcomeRow, ...]:
    """Aggregate preregistered paired outcomes and reject undeclared exclusions."""
    _validate_plan(plan)
    declared = set(plan.declared_pairs)
    excluded = set(plan.declared_exclusions)
    preregistered = set(plan.preregistered_outcomes)
    if declared & excluded:
        raise StatError("STAT_PLAN_INVALID")
    rows: list[OutcomeRow] = []
    p_values: list[tuple[str, int, int]] = []
    for family in families:
        if family.outcome_id not in preregistered:
            raise StatError("STAT_UNDECLARED_METRIC")
        diffs, missing, failed, excluded_runs = _complete_diffs(family.pairs, declared, excluded)
        if not diffs:
            rows.append(
                OutcomeRow(
                    outcome_id=family.outcome_id,
                    complete_pairs=0,
                    missing_runs=missing,
                    failed_runs=failed,
                    excluded_runs=excluded_runs,
                    effect_numerator=None,
                    effect_denominator=None,
                    ci_low_numerator=None,
                    ci_high_numerator=None,
                    ci_denominator=None,
                    p_numerator=None,
                    p_denominator=None,
                    holm_rejected=False,
                    sensitivity_min_numerator=None,
                    sensitivity_max_numerator=None,
                    sensitivity_denominator=None,
                    detail_code="STAT_PAIRS_UNAVAILABLE",
                )
            )
            continue
        effect = sum(diffs)
        low, high = _bootstrap_ci(diffs, plan.bootstrap_resamples, plan.bootstrap_seed)
        p_num, p_den = _bootstrap_p(diffs, plan.bootstrap_resamples, plan.bootstrap_seed)
        sens_min, sens_max, sens_den = _sensitivity(diffs)
        rows.append(
            OutcomeRow(
                outcome_id=family.outcome_id,
                complete_pairs=len(diffs),
                missing_runs=missing,
                failed_runs=failed,
                excluded_runs=excluded_runs,
                effect_numerator=effect,
                effect_denominator=len(diffs),
                ci_low_numerator=low,
                ci_high_numerator=high,
                ci_denominator=len(diffs),
                p_numerator=p_num,
                p_denominator=p_den,
                holm_rejected=False,
                sensitivity_min_numerator=sens_min,
                sensitivity_max_numerator=sens_max,
                sensitivity_denominator=sens_den,
                detail_code=None,
            )
        )
        p_values.append((family.outcome_id, p_num, p_den))
    rejected = _holm(p_values, plan.alpha_numerator, plan.alpha_denominator)
    return tuple(
        row if row.outcome_id not in rejected else replace(row, holm_rejected=True)
        for row in sorted(rows, key=lambda item: item.outcome_id)
    )


def _validate_plan(plan: StatisticalPlan) -> None:
    if (
        not plan.preregistered_outcomes
        or not plan.declared_pairs
        or plan.bootstrap_resamples < 2
        or plan.alpha_numerator <= 0
        or plan.alpha_denominator == 0
        or len(set(plan.preregistered_outcomes)) != len(plan.preregistered_outcomes)
        or len(set(plan.declared_pairs)) != len(plan.declared_pairs)
        or len(set(plan.declared_exclusions)) != len(plan.declared_exclusions)
    ):
        raise StatError("STAT_PLAN_INVALID")


def _complete_diffs(
    pairs: tuple[SeedPair, ...],
    declared: set[str],
    excluded: set[str],
) -> tuple[list[int], int, int, int]:
    diffs: list[int] = []
    missing = 0
    failed = 0
    excluded_runs = 0
    for pair in sorted(pairs, key=lambda item: item.pair_id):
        if pair.pair_id not in declared and pair.pair_id not in excluded:
            raise StatError("STAT_UNDECLARED_EXCLUSION")
        if pair.pair_id in excluded:
            excluded_runs += 1
            continue
        if pair.failed_a or pair.failed_b:
            failed += 1
            continue
        if pair.outcome_a is None or pair.outcome_b is None:
            missing += 1
            continue
        diffs.append(pair.outcome_a - pair.outcome_b)
    return diffs, missing, failed, excluded_runs


def _lcg(state: int) -> int:
    return (state * LCG_A + LCG_C) & LCG_MASK


def _bootstrap_sums(diffs: list[int], resamples: int, seed: int) -> list[int]:
    state = seed & LCG_MASK
    n = len(diffs)
    sums: list[int] = []
    for _ in range(resamples):
        total = 0
        for _ in range(n):
            state = _lcg(state)
            total += diffs[state % n]
        sums.append(total)
    return sums


def _bootstrap_ci(diffs: list[int], resamples: int, seed: int) -> tuple[int, int]:
    sums = sorted(_bootstrap_sums(diffs, resamples, seed))
    low_index = (25 * (resamples - 1)) // 1000
    high_index = (975 * (resamples - 1)) // 1000
    return sums[low_index], sums[high_index]


def _bootstrap_p(diffs: list[int], resamples: int, seed: int) -> tuple[int, int]:
    observed = sum(diffs)
    count = 0
    for total in _bootstrap_sums(diffs, resamples, seed):
        if observed == 0 or (observed > 0 and total <= 0) or (observed < 0 and total >= 0):
            count += 1
    return count + 1, resamples + 1


def _sensitivity(diffs: list[int]) -> tuple[int | None, int | None, int | None]:
    if len(diffs) < 2:
        return None, None, None
    leave_one = [sum(diffs) - value for value in diffs]
    return min(leave_one), max(leave_one), len(diffs) - 1


def _holm(p_values: list[tuple[str, int, int]], alpha_num: int, alpha_den: int) -> set[str]:
    ordered = sorted(p_values, key=lambda item: (item[1] * 10_000 // item[2], item[0]))
    rejected: set[str] = set()
    count = len(ordered)
    for index, (outcome_id, p_num, p_den) in enumerate(ordered):
        remaining = count - index
        if p_num * alpha_den * remaining <= p_den * alpha_num:
            rejected.add(outcome_id)
        else:
            break
    return rejected


def diagnostic_plan() -> StatisticalPlan:
    return StatisticalPlan(
        preregistered_outcomes=("latency", "reward"),
        declared_pairs=("p1", "p2", "p3", "p4"),
        declared_exclusions=("p_skip",),
        bootstrap_resamples=20,
        bootstrap_seed=1,
        alpha_numerator=5,
        alpha_denominator=100,
    )


def diagnostic_families() -> tuple[OutcomeFamily, ...]:
    reward = OutcomeFamily(
        outcome_id="reward",
        pairs=(
            SeedPair("p1", 10, 6, False, False),
            SeedPair("p2", 8, 7, False, False),
            SeedPair("p3", 12, 5, False, False),
            SeedPair("p4", 9, None, False, True),
            SeedPair("p_skip", 100, 0, False, False),
        ),
    )
    latency = OutcomeFamily(
        outcome_id="latency",
        pairs=(
            SeedPair("p1", 3, 4, False, False),
            SeedPair("p2", 2, 5, False, False),
            SeedPair("p3", 1, 6, False, False),
            SeedPair("p4", None, 2, True, False),
            SeedPair("p_skip", 0, 50, False, False),
        ),
    )
    return (latency, reward)
