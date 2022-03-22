#![allow(dead_code, unused_imports)]
use crate::bnk::SoundBank;
use crate::org::Song as Organya;
use crate::stuff::*;
use crate::wav::*;

use std::mem::MaybeUninit;

pub struct PlaybackEngine {
    song: Organya,
    mute: [bool; 16],
    lengths: [u8; 8],
    swaps: [usize; 8],
    keys: [u8; 8],
    track_buffers: [RenderBuffer; 136],
    output_format: WavFormat,
    play_pos: i32,
    frames_this_tick: usize,
    frames_per_tick: usize,
    frames_done: u32,
    pub loops: usize,
    pub extra: u32,
}

impl PlaybackEngine {
    pub fn new(song: Organya, samples: SoundBank) -> Self {

        // Octave 0 Track 0 Swap 0
        // Octave 0 Track 1 Swap 0
        // ...
        // Octave 1 Track 0 Swap 0
        // ...
        // Octave 0 Track 0 Swap 1
        // octave * 8 + track + swap
        // 128..136: Drum Tracks
        let mut buffers: [MaybeUninit<RenderBuffer>; 136] = unsafe {
            MaybeUninit::uninit().assume_init()
        };

        // track
        for i in 0..8 {
            let sound_index = song.tracks[i].inst.inst as usize;

            // WAVE100 uses 8-bit signed audio, but wav audio wants 8-bit unsigned.
            // On 2s complement system, we can simply flip the top bit
            // No need to cast to u8 here because the sound bank data is one big &[u8].
            let sound = samples.get_wave(sound_index)
                               .iter()
                               .map(|&x| x ^ 128)
                               .collect();

            let format = WavFormat { channels: 1, sample_rate: 22050, bit_depth: 8 };

            let rbuf = RenderBuffer::new_organya(WavSample { format, data: sound });

            // octave
            for j in 0..8 {
                // swap
                for &k in &[0, 64] {
                    buffers[i + (j * 8) + k] = MaybeUninit::new(rbuf.clone());
                }
            }
        }

        for (inst, buf) in song.tracks[8..].iter().zip(buffers[128..].iter_mut()) {
            *buf =
                MaybeUninit::new(
                    RenderBuffer::new(
                        // FIXME: *frustrated screaming*
                        samples.samples[inst.inst.inst as usize].clone()
                    )
                );
        }

        let frames_per_tick = ((44100.0 / 1000.0) * song.time.wait as f32) as usize;

        PlaybackEngine {
            song,
            mute: [false; 16],
            lengths: [0; 8],
            swaps: [0; 8],
            keys: [255; 8],
            track_buffers: unsafe { std::mem::transmute(buffers) },
            play_pos: 0,
            output_format: WavFormat {
                channels: 2,
                sample_rate: 44100,
                bit_depth: 16
            },
            frames_this_tick: 0,
            frames_per_tick,
            frames_done: 0,
            loops: 1,
            extra: 0,
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

        self.frames_per_tick as u32 * ticks_total as u32 + (self.extra * self.output_format.sample_rate)
    }

    fn get_active_buffer_for_track(&self, track: usize) -> usize {
        ((self.keys[track] / 12) * 8 + track as u8 + self.swaps[track] as u8) as usize
    }

    fn track_is_playing(&self, track: usize) -> bool {
        self.keys[track] != 255
    }

    fn track_start_playing(&mut self, track: usize, key: u8) {
        self.keys[track] = key;
    }

    fn track_stop_playing(&mut self, track: usize) {
        self.keys[track] = 255;
    }

    fn swap_buffers_for_track(&mut self, track: usize) {
        self.swaps[track] ^= 64;
    }

    fn note_ended(&self, track: usize) -> bool {
        self.lengths[track] == 0
    }

    fn track_kill_note(&mut self, track: usize) {
        let j = self.get_active_buffer_for_track(track);

        if self.song.tracks[track].inst.pipi == 0 {
            self.track_buffers[j].looping = false;
        }
    }

    fn track_play_note(&mut self, track: usize) {
        let j = self.get_active_buffer_for_track(track);

        self.track_buffers[j].playing = true;
        self.track_buffers[j].looping = true;
    }

    fn display_buffer(&self, buf: usize) -> String {
        let buf_ns = buf & 63;
        format!("Track {}, Octave {}, Buffer {}", buf_ns % 8, buf_ns / 8, buf >> 6)
    }

    fn update_play_state(&mut self) {
        // self.mute[0] = true;
        // self.mute[1] = true;
        // self.mute[2] = true;
        // self.mute[3] = true;
        // self.mute[4] = true;
        // self.mute[5] = true;
        // self.mute[6] = true;
        // self.mute[7] = true;

        // self.mute[8] = true;
        // self.mute[9] = true;
        // self.mute[10] = true;
        // self.mute[11] = true;
        // self.mute[12] = true;
        // self.mute[13] = true;
        // self.mute[14] = true;
        // self.mute[15] = true;

        // For every wave track...
        for track in 0..8 {
            if self.mute[track] { continue; }

            // Do we have a note for the current X pos?
            if let Some(&note) =
                self.song.tracks[track].notes.iter().find(|x| x.pos == self.play_pos) {

                // New note (Pitch of 255 is a dummy value for volume/pan adjustments)
                if note.key != 255 {
                    if self.track_is_playing(track) {
                        self.track_kill_note(track);

                        let freq = org_key_to_freq(note.key, self.song.tracks[track].inst.freq as i16);
                        let l = self.get_active_buffer_for_track(track);
                        self.track_buffers[l].set_frequency(freq as u32);

                        self.swap_buffers_for_track(track);
                    }

                    // Set last playing key
                    self.track_start_playing(track, note.key);
                    self.track_play_note(track);

                    let l = self.get_active_buffer_for_track(track);
                    let freq = org_key_to_freq(note.key, self.song.tracks[track].inst.freq as i16);
                    self.track_buffers[l].set_frequency(freq as u32);
                    self.track_buffers[l].organya_select_octave(note.key as usize/12, self.song.tracks[track].inst.pipi != 0);

                    self.lengths[track] = note.len;
                }

                // Why is this behind this check?
                //
                // The effect is that a note event immediately following
                // A note can affect the last played note, but otherwise is
                // ignored...
                if self.track_is_playing(track) {
                    let j = self.get_active_buffer_for_track(track);

                    if note.vol != 255 {
                        let vol = org_vol_to_vol(note.vol);
                        self.track_buffers[j].set_volume(vol);
                    }

                    if note.pan != 255 {
                        let pan = org_pan_to_pan(note.pan);
                        self.track_buffers[j].set_pan(pan);
                    }
                }
            }

            // Play lengths
            if self.note_ended(track) {
                if self.track_is_playing(track) {
                    self.track_kill_note(track);
                    self.track_stop_playing(track);
                }
            }

            self.lengths[track] = self.lengths[track].saturating_sub(1);
        }

        // Drum notes
        for i in 8..16 {
            if self.mute[i] { continue; }

            let j = i + 120;

            let notes = &self.song.tracks[i].notes;

            // start a new note
            // note (hah) that drums are unaffected by length and pi values. This is the only case we have to handle.
            if let Some(note) =
                notes.iter().find(|x| x.pos == self.play_pos) {

                // FIXME: Add constants for dummy values
                if note.key != 255 {
                    let freq = org_key_to_drum_freq(note.key);
                    self.track_buffers[j].set_frequency(freq as u32);
                    self.track_buffers[j].set_position(0);
                    self.track_buffers[j].playing = true;
                }

                if note.vol != 255 {
                    let vol = org_vol_to_vol(note.vol);
                    self.track_buffers[j].set_volume(vol);
                }

                if note.pan != 255 {
                    let pan = org_pan_to_pan(note.pan);
                    self.track_buffers[j].set_pan(pan);
                }
            }
        }
    }

    pub fn render_to(&mut self, buf: &mut [u32]) -> usize {
        for (i, frame) in buf.iter_mut().enumerate() {
            if self.frames_this_tick == 0 {
                self.update_play_state()
            }

            mix(std::slice::from_mut(frame), self.output_format, &mut self.track_buffers);

            self.frames_done += 1;
            self.frames_this_tick += 1;

            if self.frames_this_tick == self.frames_per_tick {
                self.play_pos += 1;

                if self.play_pos == self.song.time.loop_range.end {
                    self.play_pos = self.song.time.loop_range.start;

                    // if self.loops == 0 {
                    //     // return i + 1;
                    // }
                    // else {
                    //     self.loops -= 1;
                    // }
                }

                self.frames_this_tick = 0;
            }

            if self.frames_done == self.get_total_samples() {
                return i + 1;
            }
        }

        buf.len()
    }
}

// TODO: Create a MixingBuffer or something...
fn mix(dst: &mut [u32], dst_fmt: WavFormat, srcs: &mut [RenderBuffer]) {
    let freq = dst_fmt.sample_rate as f64;

    for (_j, buf) in srcs.into_iter().enumerate() {
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

            use std::f32::consts::PI;

            fn sinc(x: f32) -> f32
            {
                if x.abs() <= f32::EPSILON
                {
                    return 1.0;
                }

                let y = x * PI;

                y.sin() / y
            }

            fn lanczos(x: f32, a: f32) -> f32
            {
                if x.abs() >= a
                {
                    return 0.0;
                }

                sinc(x) * sinc(x / a)
            }

            fn lanczos_interp(s1: f32, s2: f32, s3: f32, s4: f32, r: f32) -> f32
            {
                // assuming floor(x) = 0
                (s4 * lanczos(r - -1.0, 2.0)) +
                (s1 * lanczos(r, 2.0)) +
                (s2 * lanczos(r - 1.0, 2.0)) +
                (s3 * lanczos(r - 2.0, 2.0))
            }

            fn lanczos_interp6(s1: f32, s2: f32, s3: f32, s4: f32, s5: f32, s6: f32, r: f32) -> f32
            {
                // assuming floor(x) = 0
                (s5 * lanczos(r - -2.0, 3.0)) +
                (s4 * lanczos(r - -1.0, 3.0)) +
                (s1 * lanczos(r, 3.0)) +
                (s2 * lanczos(r - 1.0, 3.0)) +
                (s3 * lanczos(r - 2.0, 3.0)) +
                (s6 * lanczos(r - 3.0, 3.0))
            }

            #[allow(unused_variables)]

            for (i, frame) in dst.iter_mut().enumerate() {
                let pos = buf.position as usize + buf.base_pos;
                // -1..1

                // x
                let s1 = (buf.sample.data[pos] as f32 - 128.0) / 128.0;
                // x + 1
                let s2 = (buf.sample.data[clamp(pos + 1, buf.base_pos + buf.len - 1)] as f32 - 128.0) / 128.0;
                // x + 2
                let s3 = (buf.sample.data[clamp(pos + 2, buf.base_pos + buf.len - 1)] as f32 - 128.0) / 128.0;

                // x - 1
                let s4 = (buf.sample.data[pos.saturating_sub(1)] as f32 - 128.0) / 128.0;
                // x - 2
                let s5 = (buf.sample.data[pos.saturating_sub(2)] as f32 - 128.0) / 128.0;

                // x + 3
                let s6 = (buf.sample.data[clamp(pos + 3, buf.base_pos + buf.len - 1)] as f32 - 128.0) / 128.0;
                use std::f32::consts::PI;

                let r1 = buf.position.fract() as f32;
                let r2 = (1.0 - f32::cos(r1 * PI)) / 2.0;

                //let s = s1; // No interp
                //let s = s1 + (s2 - s1) * r1; // Linear interp
                //let s = s1 * (1.0 - r2) + s2 * r2; // Cosine interp
                //let s = cubic_interp(s1, s2, s4, s3, r1); // Cubic interp
                //let s = lanczos_interp(s1, s2, s3, s4, r1);
                let s = lanczos_interp6(s1, s2, s3, s4, s5, s6, r1);
                // Ideally we want sinc/lanczos interpolation, since that's what DirectSound appears to use.

                // -128..128
                let sl = s * pan_l * vol * 32768.0;
                let sr = s * pan_r * vol * 32768.0;

                buf.position += advance;

                if buf.position as usize >= buf.len {
                    if buf.looping && buf.nloops != 1 {
                        buf.position %= buf.len as f64;
                        if buf.nloops != -1 {
                            buf.nloops -= 1;
                        }
                    } else {
                        buf.position = 0.0;
                        buf.playing = false;
                        break;
                    }
                }

                // Signed
                let (mut l, mut r) = ((*frame & 0xFFFF) as i16, (*frame >> 16) as i16);

                // eprintln!("I: {:3} {:08} {:04X} {:04X} {:04X} {:04X} {:08X}", j, i, sl as i16, sr as i16, l, r, *frame);

                l = l.saturating_add(sl as i16);
                r = r.saturating_add(sr as i16);

                *frame = (l as u32 & 0xFFFF) | ((r as u32) << 16);

                // eprintln!("O: {:3} {:08} {:04X} {:04X} {:04X} {:04X} {:08X}", j, i, sl as i16, sr as i16, l, r, *frame);

            }
        }
    }
}

pub fn centibel_to_scale(a: i32) -> f32 {
    f32::powf(10.0, a as f32 / 2000.0)
}

#[derive(Clone)]
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
    // -1 = infinite
    pub nloops: i32,
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
            nloops: -1
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
    pub fn organya_select_octave(&mut self, octave: usize, pipi: bool) {
        const OFFS: &[usize] = &[0x000, 0x100,
                                 0x200, 0x280,
                                 0x300, 0x340,
                                 0x360, 0x370];
        const LENS: &[usize] = &[256_usize,256,128,128,64,32,16,8];
        self.base_pos = OFFS[octave];
        self.len = LENS[octave];
        // What does this do??
        //self.position %= self.len as f64;
        if pipi && !self.playing {
            self.nloops = ((octave+1) * 4) as i32;
        }
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
