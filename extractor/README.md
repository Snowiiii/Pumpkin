# The Extractor

The extractor generates JSON files placed in `./run/pumpkin_extractor_output/`.
When adding an extractor, !!!ensure the extractor output is deterministic!!! and
be sure to update the `file_map.dat` file.
`file_map.dat` maps where the generated files should be moved.
The left side is the source file relative to the extractor output directory.
The right side is the destination file relative to the project root.

When modifying the extractors, or the values of the extractors changes, be sure
to run `../contrib/pin_asset_hashes.sh` and push the result to the repo.
