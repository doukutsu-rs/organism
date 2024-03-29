# Organism

Organism is an [Organya] to wave audio conversion program.\
It is written in Rust, and aims to be fast, easy to use,
and produce [highly accurate](#accuracy) conversions.

Right now, the program works, but the interface is incredibly primitive. See [usage](#usage) for more information.

If you wanna hear how it sounds, you can find a demo [here](https://www.youtube.com/watch?v=5VxJYq-yoa0).

## Installation

```sh
git clone https://gitdab.com/LunarLambda/organism.git
cd organism
cargo install --path .
```

## Usage

After installing, you can render Organya files like so:

```sh
organism organya_file loops "wav" > output_file
```

If you omit the third argument (which must be the string `wav`), then only raw PCM data will be output.

Alternatively, you can run organism without installing it, by using the included `run` script, which always uses wav output:

If you don't specify the number of loops, it defaults to 1.

```sh
# Output to wav file
./run organya_file > output.wav

# Output to flac file
./run organya_file | flac -sV8fo output.flac -

# Play
./run organya_file | aplay -q
```

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
