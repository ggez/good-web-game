# Good Web Game

good-web-game is a native wasm32-unknown-unknown implementation of some mininimal [ggez](https://github.com/ggez/ggez) subset on top of [miniquad](https://github.com/not-fl3/miniquad/). Built to run [zemeroth](https://github.com/ozkriff/zemeroth) in the web.

In action(0.1, pre-miniquad version): <https://ozkriff.itch.io/zemeroth>

![screen](https://i.imgur.com/TjvCNwa.jpg)

## Example

1) To build and run an example as a native binary that uses ggez:

    ```rust
    cargo run --example astroblasto
    ```

2) To build and run a web version of an example:

    ```rust
    rustup target add wasm32-unknown-unknown
    cargo build --example astroblasto --target wasm32-unknown-unknown
    cp target/wasm32-unknown-unknown/debug/examples/astroblsato.wasm js
    cd js/ #  and launch http server with wasm MIME, maybe check index.html to match>
    ```
