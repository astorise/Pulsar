# cognitive-gc

`cognitive-gc` is the cleanup component for stale or contradictory agent memory. It is responsible for keeping shared state compact enough for repeated autonomous runs.

The crate builds as both an `rlib` for native tests and a `cdylib` for WebAssembly component packaging.
