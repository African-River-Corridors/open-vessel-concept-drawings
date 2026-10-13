Status: Superseded by ADR 0009 (one Drawing Format Standard, two implementations).

# Draw General Arrangements with technical_drawings_for_agents

GAs are produced through the open-source
[`technical_drawings_for_agents`](https://github.com/African-River-Corridors/technical_drawings_for_agents)
tool (formerly `sankofa-draw`; engineering drawings as code: DXF + SVG, version-controlled), not with
the bespoke SVG renderer ported from the prior engine. It gives DXF for drafters. Only the output layer
imports it; the calculation core has no drawing dependency. Title blocks carry the African River
Corridors logo for now.
