# CodeHarbor CSFML C Sample

This fixture is a tiny deterministic C CSFML project for smoke-testing CodeHarbor's Ubuntu AMD64 workspace.

## Use It In CodeHarbor

1. Rebuild or recreate the Docker workspace image after the Dockerfile package changes.
2. Create an environment.
3. Set `Name` to `CodeHarbor CSFML Sample`.
4. Set `Local folder path` to this fixture directory.
5. Start the environment.
6. Run `Run full evaluation`.

Expected results:

- `Build` creates `codeharbor_csfml_sample`.
- `Tests` runs `xvfb-run --auto-servernum ./codeharbor_csfml_sample --self-test`.
- `Tests` prints `CSFML self-test passed`.
- A Markdown report appears in the `Reports` panel.
