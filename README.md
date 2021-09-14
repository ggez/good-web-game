# Good Web Game

[![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/jum3Fjek2A)

good-web-game is a wasm32-unknown-unknown implementation of a [ggez](https://github.com/ggez/ggez) subset on top of [miniquad](https://github.com/not-fl3/miniquad/). Originally built to run [zemeroth](https://github.com/ozkriff/zemeroth) in the web.

It has been recently updated to support much of the ggez 0.6.0 API. If you're already working with ggez you might use this library to port your game to the web (or perhaps even mobile).
Since it also runs well on desktop it also offers an alternative implementation of ggez, which might come in handy if you experience bugs in ggez, which you can't work around for some reason. Canvases with multisampling are currently buggy in classic ggez while they work fine in good-web-game, for example.

If you are looking for a properly maintained and supported minimal high-level engine on top of miniquad - check out [macroquad](https://github.com/not-fl3/macroquad/) instead.

## Status

"good-web-game" implements the most important parts of the ggez 0.6.0 API.

### Missing / Not available:

* filesystem with writing access (if you need it take a look at [`quad-storage`](https://github.com/optozorax/quad-storage))
* game pad support
* writing your own event loop (doesn't make much sense on callback-only platforms like HTML5)
* spatial audio (overall audio support is still relatively limited, but could be improved)
* resolution control in fullscreen mode
* setting window position / size (the latter is available on Windows, but buggy)
* screenshot function
* window icon
* and custom shader support (yes, this is a big one, but if you need it and are familiar with `miniquad` please
  consider starting a PR; `miniquad` has all the tools you need)
  
 
## Demo 

In action(0.1, pre-miniquad version): <https://ozkriff.itch.io/zemeroth>

![screen](https://i.imgur.com/TjvCNwa.jpg)

## Example

To build and run an example as a native binary:

```rust
cargo run --example 05_astroblasto
```

Building for web and mobile is currently a WIP (ironic, I know).
If you want to try your luck anyway the [miniquad instructions for WASM](https://github.com/not-fl3/miniquad/#wasm)
might be a good place to start.

## Architecture

Here is how `good-web-game` fits into your rust-based game:

![software stack](about/gwg-stack.png?raw=true "good-web-game software stack")
