use crate::bnk::SoundBank;
use crate::org::Song as Organya;
use crate::stuff::*;
use crate::wav::*;

use std::mem::MaybeUninit;

pub struct PlaybackEngine {
    song: Organya,
    lengths: [u8; 8],
    track_buffers: [RenderBuffer; 16],
    output_format: WavFormat,
    play_pos: i32,
    frames_this_tick: usize,
    frames_per_tick: usize,
    pub loops: usize,
}

impl PlaybackEngine {
    pub fn new(song: Organya, samples: SoundBank) -> Self {

        let mut buffers: [MaybeUninit<RenderBuffer>; 16] = unsafe {
            MaybeUninit::uninit().assume_init()
        };

        for (inst, buf) in song.tracks[..8].iter().zip(buffers[..8].iter_mut()) {
            // FIXME: This fucking naming oh my god
            let wave = samples.get_wave(inst.inst.inst as usize)
                              .iter()
                              .map(|&x| x ^ 128) // WAVE100 is signed 8-bit, but we expect unsigned. Flipping the top bit might also work instead.
                              .collect();

            let format = WavFormat { channels: 1, sample_rate: 22050, bit_depth: 8 };

            *buf =
                MaybeUninit::new(
                    RenderBuffer::new_organya(
                        WavSample { format, data: wave }
                    )
                );
        }

        for (inst, buf) in song.tracks[8..].iter().zip(buffers[8..].iter_mut()) {
            *buf =
                MaybeUninit::new(
                    RenderBuffer::new(
                        // FIXME: *frustrated screaming*
                        samples.samples[inst.inst.inst as usize].clone()
                    )
                );
        }

        let frames_per_tick = (44100 / 1000) * song.time.wait as usize;

        PlaybackEngine {
            song,
            lengths: [0; 8],
            track_buffers: unsafe { std::mem::transmute(buffers) },
            play_pos: 0,
            output_format: WavFormat {
                channels: 2,
                sample_rate: 44100,
                bit_depth: 16
            },
            frames_this_tick: 0,
            frames_per_tick,
            loops: 1
        }
    }

    #[allow(unused)]
    pub fn set_position(&mut self, position: i32) {
        self.play_pos = position;
    }

    pub fn get_total_samples(&self) -> u32 {
        let ticks_intro = self.song.time.loop_range.start;
        let ticks_loop = self.song.time.loop_range.end - self.song.time.loop_range.start;
        let ticks_total = ticks_intro + ticks_loop + (ticks_loop * self.loops as i32);

        self.frames_per_tick as u32 * ticks_total as u32
    }

    fn update_play_state(&mut self) {
        for i in 0..8 {
            // start a new note
            if let Some(note) =
                self.song.tracks[i].notes.iter().find(|x| x.pos == self.play_pos) {

                // FIXME: Add constants for dummy values
                // NOTE: No length dummy value. NaN (Not a Note) is represented by key == 255. Length is ignored in that case, but can be zero or one.
                if note.key != 255 {
                    let freq = org_key_to_freq(note.key, self.song.tracks[i].inst.freq as i16);
                    self.track_buffers[i].set_frequency(freq as u32);

                    self.lengths[i] = note.len;
                    self.track_buffers[i].playing = true;
                    self.track_buffers[i].looping = true;
                    let oct = note.key / 12;
                    self.track_buffers[i].organya_select_octave(oct as usize);
                }

                if note.vol != 255 {
                    let vol = org_vol_to_vol(note.vol);
                    self.track_buffers[i].set_volume(vol);
                }

                if note.pan != 255 {
                    let pan = org_pan_to_pan(note.pan);
                    self.track_buffers[i].set_pan(pan);
                }
            }

            if self.lengths[i] == 0 {
                // OrgMaker calls Play on the soundbuffer, without the looping flag. So the sample should play to the end.
                // https://github.com/shbow/organya/blob/master/source/OrgPlay.cpp#L36-L37
                // https://github.com/shbow/organya/blob/master/source/Sound.cpp#L364
                // https://docs.microsoft.com/en-us/previous-versions/windows/desktop/mt708933%28v%3dvs.85%29
                // in OrgMaker, this actually causes the buffers to temporarily play over each other.
                // since we only have one buffer per instrument, we can't do this.
                self.track_buffers[i].looping = false;
            }

            self.lengths[i] = self.lengths[i].saturating_sub(1);
        }

        for i in 8..16 {
            let notes = &self.song.tracks[i].notes;

            // start a new note
            // note (hah) that drums are unaffected by length and pi values. This is the only case we have to handle.
            if let Some(note) =
                notes.iter().find(|x| x.pos == self.play_pos) {

                // FIXME: Add constants for dummy values
                if note.key != 255 {
                    let freq = org_key_to_drum_freq(note.key);
                    self.track_buffers[i].set_frequency(freq as u32);
                    self.track_buffers[i].set_position(0);
                    self.track_buffers[i].playing = true;
                }

                if note.vol != 255 {
                    let vol = org_vol_to_vol(note.vol);
                    self.track_buffers[i].set_volume(vol);
                }

                if note.pan != 255 {
                    let pan = org_pan_to_pan(note.pan);
                    self.track_buffers[i].set_pan(pan);
                }
            }
        }

        let mut mute_mask = [true; 16];
        mute_mask[9] = true;

        self.track_buffers
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| x.playing = x.playing && mute_mask[i]);
    }

    pub fn render_to(&mut self, buf: &mut [u16]) -> usize {
        for (i, frame) in buf.iter_mut().enumerate() {
            if self.frames_this_tick == 0 {
                self.update_play_state()
            }

            mix(std::slice::from_mut(frame), self.output_format, &mut self.track_buffers);

            self.frames_this_tick += 1;

            if self.frames_this_tick == self.frames_per_tick {
                self.play_pos += 1;

                if self.play_pos == self.song.time.loop_range.end {
                    self.play_pos = self.song.time.loop_range.start;

                    if self.loops == 0 {
                        return i + 1;
                    }

                    self.loops -= 1;
                }

                self.frames_this_tick = 0;
            }
        }

        buf.len()
    }
}

// TODO: Create a MixingBuffer or something...
fn mix(dst: &mut [u16], dst_fmt: WavFormat, srcs: &mut [RenderBuffer]) {
    let freq = dst_fmt.sample_rate as f64;

    for buf in srcs {
        if buf.playing {
            // index into sound samples
            let advance = buf.frequency as f64 / freq;

            let vol = centibel_to_scale(buf.volume);

            let (pan_l, pan_r) =
                match buf.pan.signum() {
                    0 => (1.0, 1.0),
                    1 => (centibel_to_scale(-buf.pan), 1.0),
                    -1 => (1.0, centibel_to_scale(buf.pan)),
                    _ => unsafe { std::hint::unreachable_unchecked() }
                };

            fn clamp<T: Ord>(v: T, limit: T) -> T {
                if v > limit {
                    limit
                } else {
                    v
                }
            }

            // s1: sample 1
            // s2: sample 2
            // sp: previous sample (before s1)
            // sn: next sample (after s2)
            // mu: position to interpolate for
            fn cubic_interp(s1: f32, s2: f32, sp: f32, sn: f32, mu: f32) -> f32 {
                let mu2 = mu * mu;
                let a0 = sn - s2 - sp + s1;
                let a1 = sp - s1 - a0;
                let a2 = s2 - sp;
                let a3 = s1;

                a0*mu*mu2 + a1*mu2 + a2*mu + a3
            }

            #[allow(unused_variables)]

            for frame in dst.iter_mut() {
                let pos = buf.position as usize + buf.base_pos;
                // -1..1
                let s1 = (buf.sample.data[pos] as f32 - 128.0) / 128.0;
                let s2 = (buf.sample.data[clamp(pos + 1, buf.base_pos + buf.len - 1)] as f32 - 128.0) / 128.0;
                let s3 = (buf.sample.data[clamp(pos + 2, buf.base_pos + buf.len - 1)] as f32 - 128.0) / 128.0;
                let s4 = (buf.sample.data[pos.saturating_sub(1)] as f32 - 128.0) / 128.0;

                use std::f32::consts::PI;

                let r1 = buf.position.fract() as f32;
                let r2 = (1.0 - f32::cos(r1 * PI)) / 2.0;

                //let s = s1; // No interp
                //let s = s1 + (s2 - s1) * r1; // Linear interp
                //let s = s1 * (1.0 - r2) + s2 * r2; // Cosine interp
                let s = cubic_interp(s1, s2, s4, s3, r1); // Cubic interp
                // Ideally we want sinc/lanczos interpolation, since that's what DirectSound appears to use.

                // -128..128
                let sl = s * pan_l * vol * 128.0;
                let sr = s * pan_r * vol * 128.0;

                buf.position += advance;

                if buf.position as usize >= buf.len {
                    if buf.looping {
                        buf.position %= buf.len as f64;
                    } else {
                        buf.position = 0.0;
                        buf.playing = false;
                        break;
                    }
                }

                let [mut l, mut r] = frame.to_be_bytes();
                // -128..127
                let xl = (l ^ 128) as i8;
                let xr = (r ^ 128) as i8;

                // 0..255
                l = xl.saturating_add(sl as i8) as u8 ^ 128;
                r = xr.saturating_add(sr as i8) as u8 ^ 128;

                *frame = u16::from_be_bytes([l, r]);
            }
        }
    }
}

pub fn centibel_to_scale(a: i32) -> f32 {
    f32::powf(10.0, a as f32 / 2000.0)
}

pub struct RenderBuffer {
    pub position: f64,
    pub frequency: u32,
    pub volume: i32,
    pub pan: i32,
    pub sample: WavSample,
    pub playing: bool,
    pub looping: bool,
    pub base_pos: usize,
    pub len: usize,
}

impl RenderBuffer {
    pub fn new(sample: WavSample) -> RenderBuffer {
        RenderBuffer {
            position: 0.0,
            frequency: sample.format.sample_rate,
            volume: 0,
            pan: 0,
            len: sample.data.len(),
            sample,
            playing: false,
            looping: false,
            base_pos: 0,
        }
    }

    pub fn new_organya(mut sample: WavSample) -> RenderBuffer {
        let wave = sample.data.clone();
        sample.data.clear();

        for size in &[256_usize,256,128,128,64,32,16,8] {
            let step = 256 / size;
            let mut acc = 0;

            for _ in 0..*size {
                sample.data.push(wave[acc]);
                acc += step;

                if acc >= 256 {
                    acc = 0;
                }
            }
        }

        RenderBuffer::new(sample)
    }

    #[inline]
    pub fn organya_select_octave(&mut self, octave: usize) {
        const OFFS: &[usize] = &[0x000, 0x100,
                                 0x200, 0x280,
                                 0x300, 0x340,
                                 0x360, 0x370];
        const LENS: &[usize] = &[256_usize,256,128,128,64,32,16,8];
        self.base_pos = OFFS[octave];
        self.len = LENS[octave];
        self.position %= self.len as f64;
    }

    #[inline]
    pub fn set_frequency(&mut self, frequency: u32) {
        //assert!(frequency >= 100 && frequency <= 100000);
        //dbg!(frequency);
        self.frequency = frequency;
    }

    #[inline]
    pub fn set_volume(&mut self, volume: i32) {
        assert!(volume >= -10000 && volume <= 0);

        self.volume = volume;
    }

    #[inline]
    pub fn set_pan(&mut self, pan: i32) {
        assert!(pan >= -10000 && pan <= 10000);

        self.pan = pan;
    }

    #[inline]
    #[allow(unused)]
    pub fn set_position(&mut self, position: u32) {
        assert!(position < self.sample.data.len() as u32 / self.sample.format.bit_depth as u32);

        self.position = position as f64;
    }
}
