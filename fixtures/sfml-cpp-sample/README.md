# CodeHarbor SFML C++ Sample

This fixture is a tiny deterministic C++ SFML project for smoke-testing CodeHarbor's Ubuntu AMD64 workspace.

## Use It In CodeHarbor

1. Rebuild or recreate the Docker workspace image after the Dockerfile package changes.
2. Create an environment.
3. Set `Name` to `CodeHarbor SFML Sample`.
4. Set `Local folder path` to this fixture directory.
5. Start the environment.
6. Run `Run full evaluation`.

Expected results:

- `Build` creates `codeharbor_sfml_sample`.
- `Tests` runs `xvfb-run --auto-servernum ./codeharbor_sfml_sample --self-test`.
- `Tests` prints `SFML self-test passed`.
- A Markdown report appears in the `Reports` panel.
