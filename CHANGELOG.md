# 0.4.1

* fixed a memory leak caused by drawable structs not releasing their resources
* fixed `miniquad` version

# 0.4.0

## Added

* added a re-export of miniquad for those who desire/need more control
* added `set`, `get_sprites` and `get_sprites_mut` to `SpriteBatch` for consistency with `ggez` 0.6.1
* added an "audio" feature, making audio optional 

## Changed

* updated audio to use `quad-snd` 0.2.2 - changing the audio API a little
* changed `05_astroblasto.rs` into just `astroblasto.rs` to allow for easier building on Android (as leading numbers seem to be disallowed there)

## Fixed

* fixed remaining clippy lints