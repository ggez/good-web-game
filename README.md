# Good Web Game

good-web-game is a native wasm32-unknown-unknown implementation of some mininimal [ggez](https://github.com/ggez/ggez) subset on top of [miniquad](https://github.com/not-fl3/miniquad/). Built to run [zemeroth](https://github.com/ozkriff/zemeroth) in the web.

## Deprecation status

"good-web-game" is a last-resort option to port big ggez project to web or mobiles. However it is pretty much unmaintained and is more of a code reference than a ready to use thing.

If you are looking for maintained and supported minimal high-level engine on top of miniquad - check out [macroquad](https://github.com/not-fl3/macroquad/).
 
## Demo 

In action(0.1, pre-miniquad version): <https://ozkriff.itch.io/zemeroth>

![screen](https://i.imgur.com/TjvCNwa.jpg)

## Example

To build and run an example as a native binary:

```rust
cargo run --example astroblasto
```

To build and run a web version follow [miniquad instructions](https://github.com/not-fl3/miniquad/#wasm)

## Architecture

Here is how `good-web-game` fits into your rust-based game:

![software stack](about/gwg-stack.png?raw=true "good-web-game software stack")
