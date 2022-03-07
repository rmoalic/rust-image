# Toy image library

A project made to learn about rust

Convert png to ppm.

## Run

```sh
$ RUST_LOG=trace cargo run --release test.png
```

Result in ```out.ppm```

## Reference
- PNG: https://www.w3.org/TR/PNG
- PNG Filters: https://www.w3.org/TR/PNG-Filters.html
- Test suite: http://www.schaik.com/pngsuite
- PPM: https://en.wikipedia.org/wiki/Netpbm#File_formats
- Adler32: https://fr.wikipedia.org/wiki/Adler-32
