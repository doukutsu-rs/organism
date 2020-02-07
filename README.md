# Organism

Organism is an [Organya] to wave audio conversion program.\
It is written in Rust, and aims to be fast, easy to use,
and produce [highly accurate](#accuracy) conversions.

Right now, the code works, and is roughly on par with [in\_org] in terms of sound quality ([demo](https://www.youtube.com/watch?v=j_btVvNkWnM)).\
However, the code is pretty messy and requires a lot of refactoring, as well as some tweaks to mixing.\
By no means is this a finished product yet!

To use it, run `cargo run -- <organya file> | aplay -q -fcd -fU8`.

## Prior Art

Programs that have

- [in\_org], a plugin for Winamp. It has issues with playing drum tracks at low pitches.

- [Org2Raw], built with code from the [Cave Story Engine 2][CSE2] project. I can't find the source code for it, and it's rather clunky to use.

- [Org2XM], which converts Organya files to FastTracker modules. Requires two-step conversion and the original code is pretty outdated. I might build my own some day.

- Recording Org Maker or Cave Story in Audacity. Requires lots of manual effort and depends on hardware.


## Accuracy

Several aspects of the Organya format and Pixel's original code need to be considered when talking about accuracy.

#### Format

Org Maker and Cave Story do not verify that Organya files are valid aside from checking the magic number.

This can cause all sorts of mayhem, so Organism will validate all files beforehand. (TODO!)

#### Sound

Pixel's engine is built on Microsoft's (now deprecated) DirectSound APIs.

All playback happens through DirectSound buffers, and frequency, volume, and pan
controls can all be emulated very easily.

#### Timing

Timing is done using Windows Multimedia Timers.\
This means that playback speed is never 100% consistent, though fluctuations are minimal.

Since Organism does not provide real-time playback, timing is done by counting
the number of processed samples.

#### Percussion

While Org Maker and Org View use regular Wave files for the percussion samples,
Cave Story uses Pixel's own format, PixTone, which will require its own emulation.

PixTone support may be added in the future.

#### Fidelity

Due to several oddities in Pixel's code, the melody instruments often produce a 'pop' sound at the start and end of notes.

The details of this are fuzzy, so this is *not* emulated.

## Code References

Alongside my own experimentation with existing tools, these projects have provided a lot of insight into how Organya works.

[Org Maker 2], which was built using the original code by Pixel, which is largely still intact.

[Cave Story Engine 2][CSE2], which aims to create a bit-perfect decompilation of the original game.

[Organya]: https://www.cavestory.org/download/music.php
[in\_org]: https://github.com/Yukitty/in_org
[Org2Raw]: https://www.cavestory.org/download/music-tools.php
[Org2XM]: https://github.com/Clownacy/org2xm
[Org Maker 2]: https://github.com/shbow/organya
[CSE2]: https://github.com/Clownacy/Cave-Story-Engine-2

## License

Organism is released under the MPL-2.0 license. See [LICENSE](./LICENSE) for more information.
