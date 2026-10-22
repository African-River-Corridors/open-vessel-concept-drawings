#!/usr/bin/env python3
"""Move every changelog fragment into CHANGELOG.md (issue #560). See changes/README.md."""
import runpy
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
runpy.run_module("changelog_fragments", run_name="__main__")
