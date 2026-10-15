"""Drive the GA drawings from ``barge_capacity`` — the current model — not the legacy one.

WHY THIS EXISTS. A GA drawer (e.g. an SVG GA module) reads ``models.BargeInputs`` /
``BargeResults``, which is the LEGACY ATB barge model: it sizes its own hull and tanks. ``barge_capacity`` is the
standardised module the rest of the package uses, and the two DISAGREE — on a CN II motor
vessel the legacy solver returns a 9.95 m moulded depth, 5.01 m draft and 12.80 m tank OD
where ``barge_capacity`` gives 5.4 m, 4.40 m and 11.25 m.

Drawing the legacy numbers on a pack whose tonnages come from ``barge_capacity`` would be the
exhibit-vs-model drift that is easy to miss and costly to find late. So this adapter builds the
drawing's own structures FROM the capacity result: the picture is then a view of the current
model rather than a second model that happens to look similar.

It converts geometry ONLY. Nothing here recomputes a capacity, a weight or a draft — every
number it passes through was produced by ``barge_capacity`` or ``journey_derating``.
"""
from __future__ import annotations

from typing import Optional, Tuple

from .models import BargeInputs, BargeResults
from .barge_capacity import BargeCapacityInputs, BargeCapacityResult
from .._bridge import call as _call


def is_gas(res: BargeCapacityResult) -> bool:
    return res.phase == "liquid_gas"


def to_ga(inp: BargeCapacityInputs, res: BargeCapacityResult,
          laden_draft_m: Optional[float] = None, motor_vessel: bool = True):
    """(BargeInputs, BargeResults) for ``svg_drawings``, built from a capacity result.

    ``laden_draft_m`` overrides the full-load draft with the channel-limited one from
    ``journey_derating`` — that is the draft the vessel actually floats at on this river, and
    it is the one a reader of the pack needs to see.

    ``motor_vessel=True`` sets ``tug_length=0``, which ``svg_drawings`` reads as
    self-propelled: aft machinery and accommodation inside the hull, no pushed tug.
    """
    kwargs = {"motor_vessel": motor_vessel}
    if laden_draft_m is not None:
        kwargs["laden_draft_m"] = laden_draft_m
    return _call("ga_adapter", "to_ga", {"inputs": inp, "result": res, "kwargs": kwargs}, Tuple[BargeInputs, BargeResults])
