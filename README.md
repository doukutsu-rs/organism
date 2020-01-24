# Organism

Organism is a high-accuracy [Organya] to raw audio conversion tool.\
It is written in Rust, and aims to be fast, easy to use,\
and produce [as accurate as possible](#accuracy) conversions.

# Why?

I absolutely adore Cave Story, and always wanted a proper "true to life" conversion of its music I
could listen to wherever.

However, the soundtrack never got an official release, and most if not all conversion tools prior to
[2018](#references) didn't get it right, and fixing them would have been far too difficult.

# Prior Art

Several attempts at achieving this were made before, with varrying issues.

- [in\_org], a plugin for Winamp. Winamp is largely dead, and the plugin had issues with percussion
  at low pitches. I tried fixing it, but to no avail.
- [Org2Raw], which was built from code from the Cave Story Engine 2 decompilation project, and while
  probably very accurate, is pretty limited (and I couldn't get it to work correctly...)
- [Org2XM], which worked well but got several technical details of the Organya format wrong, wasn't easy to use, and of course, required conversion from XM to WAV.

What most people ended up doing was just recording their soundcard output in Audacity, which works, but
isn't pretty by any means.

Organism aims to provide accurate and fast emulation of the Cave Story music engine, while having
well-documented, modern code, a simple command line interface and, in true unix fashion, be easily
composable with other programs, such as audio encoders (lame, oggenc, flac, ffmpeg), or audio
playback programs (aplay).

# Accuracy

The original Cave Story music playback engine is built on Microsoft's DirectSound APIs and Windows
MM timers.

Pretty much all playback relies on DirectSound sample buffers, which are fairly primitive and
can be emulated very accurately.

Timers are a lot more tricky, and not extremely consistent.\
Organism instead does millisecond timing using sample counting.\
Essentially, it counts how many samples it has processed to determine how much time has passed.

Another issue are the percussion samples. Cave Story used another format, PixTone, for them, which would require its own emulation as well.\
Pixel's Org Maker program, instead uses WAV samples, which are
a lot easier to deal with. PixTone support is planned for the future though.

### Source Code References

Alongside my own experimentation with the file format and various versions of Org Maker, these projects have provided a lot of insight into how Cave Story's music engine works.

[Org Maker 2], which was built using the original code by Pixel, which is largely still intact.

The [Cave Story Engine 2][CSE2] project, which aims to create a bit-perfect decompilation of the original game.

[Organya]: https://www.cavestory.org/download/music.php
[in\_org]: https://github.com/Yukitty/in_org
[Org2Raw]: https://www.cavestory.org/download/music-tools.php
[Org2XM]: https://github.com/Clownacy/org2xm
[Org Maker 2]: https://github.com/shbow/organya
[CSE2]: https://github.com/Clownacy/Cave-Story-Engine-2
