# Compressed Vec

`CompressedVec` compresses `serde` serializable structs via `flate2` and `serde_json` into buckets to conserve RAM.

It was originally written to save Episodes of Game Engine Action Replays which would hold a few GB of RAM even when they don't need to be accessed.
This makes accessing your data more expensive computation wise but saves RAM. It hasn't been optimized at all for speed yet and only implements the `Vec` functions I needed for my project.
Pull requests to bring functionality more on par with the original `Vec` are more than welcome!

# Warning
This vector is intended as append only and afterwards sequential reads via single-threaded access.
Everything outside of the indeded purposes might fail.