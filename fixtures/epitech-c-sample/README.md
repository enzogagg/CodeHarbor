# CodeHarbor Epitech C Sample

This fixture is a tiny deterministic C project for manually smoke-testing CodeHarbor.

## Use It In CodeHarbor

1. Create an environment.
2. Set `Name` to `CodeHarbor Sample`.
3. Set `Local folder path` to this fixture directory.
4. Start the environment.
5. Run `Run full evaluation`.

Expected results:

- `Build` creates `codeharbor_sample`.
- `Tests` prints `self-test passed`.
- `Valgrind` can run against `codeharbor_sample` after it is detected.
- `History`, `Artifacts`, `Docker`, and `Reports` update in the app.
