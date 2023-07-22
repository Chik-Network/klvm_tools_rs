Build
-----

Clone GitHub repository
```bash
git clone https://github.com/Chik-Network/klvm_tools_rs
cd klvm_tools_rs/wasm
```

Use `wasm-pack` to build the wasm `pkg` file used with npm. Install it with:

```bash
cargo install wasm-pack
```

Then build with

```bash
# Make sure you're at <klvm_tools_rs root>/wasm
wasm-pack build --release --target=nodejs
```

Test
-----
Prerequisite:
- NodeJS >= 16
- Wasm files built by `wasm-pack` command exist at `<klvm_tools_rs root>/wasm/pkg/`

```bash
# Make sure you're at <klvm_tools_rs root>/wasm
node ./tests/index.js
```
