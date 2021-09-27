<img align="right" width="25%" src="about/logo.svg">

# Good Web Game

[![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/jum3Fjek2A)
[![Docs Status](https://docs.rs/good-web-game/badge.svg)](https://docs.rs/good-web-game)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ggez/good-web-game/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/good-web-game.svg)](https://crates.io/crates/good-web-game)

good-web-game is a wasm32-unknown-unknown implementation of a [ggez](https://github.com/ggez/ggez) subset on top of [miniquad](https://github.com/not-fl3/miniquad/). Originally built to run [Zemeroth](https://github.com/ozkriff/zemeroth) on the web.

It has been recently updated to support much of the ggez 0.6.1 API. If you're already working with ggez you might use this library to port your game to the web (or even mobile).
Since it also runs well on desktop it also offers an alternative implementation of ggez, which might always come in handy.

If you are just looking for a well supported minimal high-level engine on top of miniquad you might want to take a look at [macroquad](https://github.com/not-fl3/macroquad/).

## Supported Platforms

The idea behind good-web-game is to offer a way to easily port ggez games to the web and even to mobile platforms. As it's mostly a subset of ggez, porting from good-web-game to ggez is even simpler.

<p align="center" width="100%">
    <img width="90%" src="about/supported_platforms.svg">
</p>

Note that we don't give any guarantees for iOS / macOS support, as we currently simply don't have Macs lying around to test it on. It _should_ work just fine though.

## Status

"good-web-game" implements most of the ggez 0.6.1 API.

### Differences

* boilerplate code differs slightly, [as shown here](https://github.com/PSteinhaus/PSteinhaus.github.io/tree/main/ggez/web-examples#ggez-animation-example)
* shaders have to be written in GLSL100, due to support for WebGL1
* API for creation of shaders and their corresponding uniform structs differs slightly, but the workflow remains the same, see [the `shader` example](examples/shader.rs)

### Missing / Not available:

* ggez (and therefore good-web-game) loads files in a blocking fashion, which doesn't work on Wasm (and is also currently not supported on mobile)
* filesystem with writing access (if you need it take a look at [`quad-storage`](https://github.com/optozorax/quad-storage))
* writing your own event loop (doesn't make much sense on callback-only platforms like HTML5)
* spatial audio (overall audio support is still relatively limited, but could be improved)
* resolution control in fullscreen mode
* setting window position / size (the latter is available on Windows, but buggy)
* screenshot function
* window icon
* gamepad support on WASM (as `gilrs` depends on wasm-bindgen)
 
## Demo 

Running Zemeroth: <https://not-fl3.github.io/miniquad-samples/zemeroth.html>

![screen](https://i.imgur.com/TjvCNwa.jpg)

You can also check out [astroblasto running on the web](https://psteinhaus.github.io/gwg-example/) ([source](https://github.com/PSteinhaus/PSteinhaus.github.io/tree/main/gwg-example)).

## Example

To build and run an example as a native binary:

```rust
cargo run --example 05_astroblasto
```

If you want to build for WASM take a look at the [miniquad instructions for WASM](https://github.com/not-fl3/miniquad/#wasm).

There you'll also find infos on building for [Android](https://github.com/not-fl3/miniquad/#android) / [iOS](https://github.com/not-fl3/miniquad/#ios).

## On blurry graphics

You may run into somewhat blurry graphics. This is caused by high-dpi rendering:

When run on a system with a scaling factor unequal to 1 the graphics may appear blurry, due to the drawbuffer being scaled up, to achieve a window of the size requested by your OS.
This size is usually "the size you specified in `Conf`" * "your OS scaling factor".

To avoid this set `Conf::high_dpi` to `true`. This leads to the drawbuffer being the size of your actual physical window. It also means though that you can't be sure how big your drawable space will actually be, as this will then depend on where the program is being run.

We aim towards changing this, so that windows are always created with the physical size specified in `Conf`, but that's not directly supported by miniquad currently.

## Architecture

Here is how `good-web-game` fits into your rust-based game:

![software stack](about/gwg-stack.png?raw=true "good-web-game software stack")
