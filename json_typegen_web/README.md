# `json_typegen_web`

Web interface for `json_typegen` using Web Assembly

See [main project repo](https://github.com/evestera/json_typegen) for more extensive README

Need [wasm-pack](https://github.com/rustwasm/wasm-pack) installed.

## Development

For development, run e.g.

```sh
(cd .. && watchexec -e rs -- wasm-pack build --target web json_typegen_wasm)
```

and

```sh
npm run dev
```

If you're not touching the Rust code, you can build the wasm package just once with

```sh
(cd ../json_typegen_wasm && wasm-pack build --target web)
```
