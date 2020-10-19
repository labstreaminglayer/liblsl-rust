# lsl-sys

This is the low-level auto-generated binding to the [liblsl] library.
You probably don't want to use this library directly; instead, check out [liblsl-rust].

### Getting the source code

```
git clone --recurse-submodules https://github.com/intheon/liblsl-rust
``` 

### Regenerating C FFI bindings

You only need to run this if you want to update the autogenerated header bindings
(`src/generated.rs`), e.g., to pull in new declarations from an updated liblsl version.

For this you need the `bindgen` tool, which you can obtain as described here
[here](https://rust-lang.github.io/rust-bindgen/command-line-usage.html).

Then, run the following commands to regenerate the bindings:
```
cd lsl-sys

# (prefix with liblsl license text)
echo "/* $(cat liblsl/LICENSE) */" > src/generated.rs

# (append bindings to file)
bindgen liblsl/include/lsl_c.h \
    --whitelist-function "^lsl_.*" \
    --whitelist-var "^lsl_.*" \
    --whitelist-type "^lsl_.*" \
    >> src/generated.rs

```

[liblsl]: https://github.com/sccn/liblsl
[liblsl-rust]: https://github.com/intheon/liblsl-rust