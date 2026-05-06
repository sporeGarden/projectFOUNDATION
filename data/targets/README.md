# targets/

Validation target manifests. Each TOML file declares the expected numerical
results that NUCLEUS compositions must reproduce to validate a domain thread.

## How Targets Work

1. **Published values** are extracted from the original papers (e.g. "cell
   cycle duration = 9.0 +/- 0.5 hours" from Karr et al. 2012)
2. **Tolerance** specifies acceptable deviation
3. **Spring** identifies which spring's experiments validate the target
4. **Validated** flips to `true` when a NUCLEUS run matches the target
5. **BLAKE3** records the hash of the result artifact upon validation

Targets are the bridge between "the paper says X" and "NUCLEUS reproduces X."

## Naming Convention

`thread<NN>_<short>_targets.toml` — matches the thread numbering in
`lineage/THREAD_INDEX.toml`.
