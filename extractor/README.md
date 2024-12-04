# The Extractor

The extractor generates JSON files placed in `./run/pumpkin_extractor_output/`.
When adding an extractor, be sure to update the `file_map.dat` file;
a text file that maps where the generated files should be moved.
The left side is the source file relative to this directory. The right side is
the destination file relative to the project root.
