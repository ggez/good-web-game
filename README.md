# Good Web Game

good-web-game is a native wasm32-unknown-unknown implementation of some mininimal [ggez](https://github.com/ggez/ggez) subset using WebGL1 and 2d canvas. Built to run [zemeroth](https://github.com/ozkriff/zemeroth) in the web.

In action: <https://ozkriff.itch.io/zemeroth>

![screen](https://i.imgur.com/TjvCNwa.jpg)

## Example

1) To build and run an example as a native binary that uses ggez:

    ```rust
    cd examples/simple
    cargo run
    ```

2) To build and run a web version of an example:

    ```rust
    cargo install cargo-web
    cd examples/simple
    cargo web start
    ```

    Then open `http://localhost:8000` in your browser.
