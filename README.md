## Build

### Javascript

Install wasm-pack: `cargo install wasm-pack`

```bash
cd js
yarn build
```

## Compiling Polylang to Miden

You can use the `compile` binary to compile Polylang functions to Miden. Compile outputs the generated Miden assembly to stdout, you can pipe it to `miden-run` to run it.

### Example of compiling and running a contract function

```bash
 $ cargo run --bin compile -- contract:Account function:setName <<<'contract Account { id: string; name: string; function setName(newName: string) { this.name = newName; } }'

 $ cargo run --bin compile -- contract:Account function:setName <<<'contract Account { id: string; name: string; function setName(newName: string) { this.name = newName; } }' \
  | cargo run -p miden-run -- \
    --this-json '{ "id": "id1", "name": "John" }' \
    --advice-tape-json '["Tom"]'

# Output: this_json: {"id":"id1","name":"Tom"}
```

### Example of compiling and running a standalone function

```bash
 $ cargo run --bin compile -- function:main <<<'function main() { }' | cargo run -p miden-run
```

## Test

```bash
cargo test && (cd parser && cargo test)
```

## Contribution

Contributions of all sorts (bug reports, enhancement requests etc.) are welcome. For more information on contribution tips and guidelines, please see the [Contributing](CONTRIBUTING.md) page.
