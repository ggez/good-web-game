<img align="right" width="25%" src="about/logo.svg">

# Good Web Game

[![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/jum3Fjek2A)
[![Docs Status](https://docs.rs/good-web-game/badge.svg)](https://docs.rs/good-web-game)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ggez/good-web-game/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/good-web-game.svg)](https://crates.io/crates/good-web-game)

good-web-game is a wasm32-unknown-unknown implementation of a [ggez](https://github.com/ggez/ggez) subset on top of [miniquad](https://github.com/not-fl3/miniquad/). Originally built to run [Zemeroth](https://github.com/ozkriff/zemeroth) on the web.

It currently supports most of the ggez 0.7.0 API. If you're already working with ggez you might use this library to port your game to the web (or even mobile).
Since it also runs well on desktop it also offers an alternative implementation of ggez, which might always come in handy.

If you are just looking for a well supported, more serious, minimal high-level engine on top of miniquad you might want to take a look at [macroquad](https://github.com/not-fl3/macroquad/).

## Supported Platforms

The idea behind good-web-game is to offer a way to easily port ggez games to the web and even to mobile platforms. As it's mostly a subset of ggez, porting from good-web-game to ggez is even simpler.

<p align="center" width="100%">
    <img width="90%" src="about/supported_platforms.svg">
</p>

Note that we don't give any guarantees for iOS / macOS support, as we currently simply don't have Macs lying around to test it on. In theory, it _should_ work though.

## Status

"good-web-game" implements most of the ggez 0.7.0 API.

### Differences

* boilerplate code differs slightly, [as shown here](https://github.com/PSteinhaus/PSteinhaus.github.io/tree/main/ggez/web-examples#ggez-animation-example)
* audio API differs somewhat due to use of `quad-snd` instead of `rodio` for easy portability
* if you want to run on the web, shaders have to be written in GLSL100, due to support for WebGL1
* API for creation of shaders and their corresponding uniform structs differs slightly, but the workflow remains the same, see [the `shader` example](examples/shader.rs)

### Missing / Not available:

* ggez (and therefore good-web-game) usually loads files in a blocking fashion, which doesn't work on WASM
  * loading files asynchronously is possible through [`load_file_async`](https://docs.rs/good-web-game/0.5.0/good_web_game/filesystem/fn.load_file_async.html) everywhere though
* filesystem with writing access (if you need it take a look at [`quad-storage`](https://github.com/optozorax/quad-storage))
* writing your own event loop (doesn't make much sense on callback-only platforms like HTML5)
* spatial audio (overall audio support is still relatively limited)
* resolution control in fullscreen mode
* setting window position / size (the latter is available on Windows, but buggy)
* screenshot function
* window icon
* gamepad support on WASM (as `gilrs` depends on wasm-bindgen)
 
## Demo 

Running Zemeroth: <https://not-fl3.github.io/miniquad-samples/zemeroth.html>

![screen](https://i.imgur.com/TjvCNwa.jpg)

You can also check out [astroblasto running on the web](https://psteinhaus.github.io/gwg-example/) ([source](https://github.com/PSteinhaus/PSteinhaus.github.io/tree/main/gwg-example)).

## Building for different platforms

To build and run an example as a native binary:

```
cargo run --example astroblasto
```

### WebAssembly

```
rustup target add wasm32-unknown-unknown
cargo build --example astroblasto --target wasm32-unknown-unknown
```

And then use the following .html to load .wasm:

<details><summary>index.html</summary>

```html
<html lang="en">

<head>
    <meta charset="utf-8">
    <title>TITLE</title>
    <style>
        html,
        body,
        canvas {
            margin: 0px;
            padding: 0px;
            width: 100%;
            height: 100%;
            overflow: hidden;
            position: absolute;
            background: black;
            z-index: 0;
        }
    </style>
</head>

<body>
    <canvas id="glcanvas" tabindex='1'></canvas>
    <!-- For now this is just the same js glue macroquad uses: https://github.com/not-fl3/macroquad/tree/master/js -->
    <script src="https://psteinhaus.github.io/js/js_bundle.js"></script>
    <script>load("astroblasto.wasm");</script> <!-- Your compiled wasm file -->
</body>

</html>
```
</details>

To run it you need a server. An easy way to start one:

```
cargo install basic-http-server
basic-http-server .
```

### Android


Recommended way to build for android is using Docker.<br/>
miniquad uses a slightly modifed version of `cargo-apk`

```
docker run --rm -v (your project folder):/root/src -w /root/src notfl3/cargo-apk cargo quad-apk build --example astroblasto
```

APK file will be in `target/android-artifacts/(debug|release)/apk`

With "log-impl" enabled all log calls will be forwarded to the adb console.
No code modifications for Android required.

Note that all examples starting with numbers (for example `03_drawing.rs`) won't install correctly. You need to remove the leading numbers: `drawing.rs`

### iOS

See miniquad iOS [sample project](https://github.com/Gordon-F/miniquad_ios_example).

## On blurry graphics

You may run into somewhat blurry graphics. This is caused by high-dpi rendering:

When run on a system with a scaling factor unequal to 1 the graphics may appear blurry, due to the drawbuffer being scaled up, to achieve a window of the size requested by your OS.
This size is usually "the size you specified in `Conf`" * "your OS scaling factor".

To avoid this set `Conf::high_dpi` to `true`. This leads to the drawbuffer being the size of your actual physical window. It also means though that you can't be sure how big your drawable space will actually be, as this will then depend on where the program is being run.

We aim towards changing this, so that windows are always created with the physical size specified in `Conf`, but that's not directly supported by miniquad currently.

## Architecture

Here is how `good-web-game` fits into your rust-based game:

![software stack](about/gwg-stack.png?raw=true "good-web-game software stack")
