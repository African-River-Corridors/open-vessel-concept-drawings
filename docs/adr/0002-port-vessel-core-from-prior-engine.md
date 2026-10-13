# Port the vessel core from a prior engine; don't rebuild it

A prior private feasibility engine already held a tested vessel core: cargo, Type C tanks and
pressure vessels, hull, hydrostatics, weights, capacity, gas separation, protective location,
resistance, squat and electric energy, with its tests. We port that core and its tests into this
repo and reshape its interfaces, instead of starting fresh. Modules that are not vessel design stay
behind.

## Consequences

The ported code carries defaults from its first use (e.g. Cb 0.85, a 28 m tank-length cap) that
must become inputs. Fabrication rates are already data (`examples/barge_cost_rates.example.yaml`).
The ported drawings were bespoke SVG; GAs move to a separate drawing tool (ADR 0003).
