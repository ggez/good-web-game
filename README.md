# Good Web Game

good-web-game is a native wasm32-unknown-unknown implementation of some mininimal [ggez](https://github.com/ggez/ggez) subset using WebGL1 and 2d canvas. Built to run [zemeroth](https://github.com/ozkriff/zemeroth) in the web.

In action: <https://ozkriff.itch.io/zemeroth>

![screen](https://i.imgur.com/TjvCNwa.jpg)

## Example

```rust
cargo install cargo-web
cd examples/simple
cargo web build
cargo web start
```

Then open `127.0.0.1:8000` in your browser.
