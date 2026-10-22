<!-- unreleased -->
- Release (issue #1): a new workflow, `publish-pypi.yml`, publishes the package to PyPI when a `v*` tag is pushed. It uses trusted publishing (no token), runs the release gate first, and waits for the maintainer to approve the `pypi` environment.
