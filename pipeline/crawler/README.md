# crawler

Fetches the crates.io db-dump and `.crate` source tarballs for the RDRAND
prevalence study.

## Build

```
cargo build --release
```

## Use

```
# 1. Download + extract the daily db-dump (~2 GB compressed).
./target/release/crawler db-dump

# 2. Fetch the latest non-yanked stable version of every crate.
#    Add --limit N for smoke tests.
./target/release/crawler fetch --concurrency 6 --limit 20
```

Both subcommands honor `--scratch <path>`, the `CRATES_SCRATCH_DIR` env var,
or fall back to `~/scratch/crates-io/`.

## Outputs

Under the scratch dir:

```
db-dump.tar.gz                # raw archive
db-dump/<dated-dir>/...       # extracted CSVs
tarballs/<aa>/<bb>/<name>/<name>-<version>.crate
manifest.csv                  # name,version,sha256,size_bytes,fetched_at
snapshot.toml                 # url, sha256, fetched_at of the db-dump
```

`manifest.csv` is append-only and idempotent: re-running `fetch` only
downloads crates not already recorded with a matching tarball on disk. The
fetcher verifies each `.crate` against the `checksum` column from
`versions.csv` before accepting it.

## Notes

- User-Agent identifies the requester per crates.io's crawler policy.
- "Latest" means most recently created non-yanked, non-prerelease version
  per crate. This is a deliberate first cut; a semver-aware refinement
  belongs in a follow-up if needed.
