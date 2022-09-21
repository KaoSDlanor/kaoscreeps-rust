### Installation
```sh
cargo install --git https://github.com/rustyscreeps/cargo-screeps.git --branch bindgen
cp example-screeps.toml screeps.toml
```

### Building
 - Set your auth token in `screeps.toml`
 - Run `cargo screeps build` to build the .webp file or `cargo screeps upload` to build and upload