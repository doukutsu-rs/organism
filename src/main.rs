mod bnk;
mod org;
mod playback;
mod stuff;
mod wav;

use crate::playback::PlaybackEngine;

use std::env;
use std::fs::File;
use std::io::{self, BufReader, Write};

use byteorder::{LE, WriteBytesExt};

const BANK_DATA: &[u8] = include_bytes!("../assets/samples/Samples.bnk");

fn main() -> io::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let output_wav = args.get(1).map_or(false, |x| x == "wav");

    let file  = File::open(&args[0])?;
    let f     = BufReader::new(file);

    let org = org::Song::load_from(f)?;
    let bnk = bnk::SoundBank::load_from(BANK_DATA)?;

    let mut playback = PlaybackEngine::new(org, bnk);

    let mut time = std::time::Duration::new(0, 0);

    if output_wav {
        print_wav_header(playback.get_total_samples())?;
    }

    loop {
        eprint!("\rRendering {:02}:{:02}", time.as_secs() / 60, time.as_secs() % 60);

        let mut buf = vec![0x8080; 44100];

        let frames = playback.render_to(&mut buf);

        for frame in &buf[..frames] {
            io::stdout().write_all(&frame.to_be_bytes()).unwrap();
        }

        time += std::time::Duration::from_secs(1);

        if frames < buf.len() {
            break;
        }
    }

    Ok(())
}

fn print_wav_header(samples: u32) -> io::Result<()> {
    let data_size = 2 * samples;
    let riff_size = 36 + data_size;

    let format = WAVEFORMATEX::new(2, 44100, 8);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    out.write_all(b"RIFF")?;
    out.write_u32::<LE>(riff_size)?;
    out.write_all(b"WAVE")?;
    out.write_all(&mut format.to_bytes())?;
    out.write_all(b"data")?;
    out.write_u32::<LE>(data_size)?;

    Ok(())
}

#[allow(non_snake_case)]
#[repr(C)]
struct WAVEFORMATEX {
    // Must be 1
    wFormatTag: u16,
    // Must be 2
    nChannels: u16,
    // Must be 44100
    nSamplesPerSec: u32,
    // Must be 44100 * nBlockAlign
    nAvgBytesPerSec: u32,
    // Must be nChannels * wBitsPerSample / 8
    nBlockAlign: u16,
    // Must be 8
    wBitsPerSample: u16,
}

#[allow(non_snake_case)]
impl WAVEFORMATEX {
    const fn new(nChannels: u16, nSamplesPerSec: u32, wBitsPerSample: u16) -> Self {
        let nBlockAlign = nChannels * wBitsPerSample / 8;
        let nAvgBytesPerSec = nSamplesPerSec * nBlockAlign as u32;

        WAVEFORMATEX {
            wFormatTag: 1,
            nChannels,
            nSamplesPerSec,
            nAvgBytesPerSec,
            nBlockAlign,
            wBitsPerSample
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(24);
        out.write_all(b"fmt ").unwrap();
        out.write_u32::<LE>(16).unwrap();
        out.write_u16::<LE>(self.wFormatTag).unwrap();
        out.write_u16::<LE>(self.nChannels).unwrap();
        out.write_u32::<LE>(self.nSamplesPerSec).unwrap();
        out.write_u32::<LE>(self.nAvgBytesPerSec).unwrap();
        out.write_u16::<LE>(self.nBlockAlign).unwrap();
        out.write_u16::<LE>(self.wBitsPerSample).unwrap();
        out
    }
}

/*
fn main() {
    let mut all_a = Vec::new();
    let mut all_b = Vec::new();

    for i in 0..100 {
        let wave = &BANK_DATA[i*256..(i+1)*256];
        let mut half = [0; 128];
        cut(&mut half, wave);

        // How OrgMaker outputs it (double resample)
        let a = resample(44100, &half, 56320);
        // How one could do it (single resample)
        let b = resample(44100,  wave,     112640);

        all_a.extend_from_slice(&a);
        all_b.extend_from_slice(&b);

        assert_eq!(a.len(), b.len());

        let mut count = 0;
        let mut delta = 0;

        for (_, (ax, bx)) in a.iter().zip(b.iter()).enumerate() {
            if ax != bx {
                // println!("[{:3}] a = {:3}, b = {:3}", i, ax, bx);

                count += 1;

                let diff = match ax.cmp(bx) {
                    std::cmp::Ordering::Less => bx - ax,
                    std::cmp::Ordering::Greater => ax - bx,
                    _ => 0
                };

                if diff > delta {
                    delta = diff;
                }
            }
        }

        println!("[Wave{:02}] {:3} samples, {:3} differences, largest delta: {:3}", i, a.len(), count, delta);
    }

    std::fs::write("a", all_a).unwrap();
    std::fs::write("b", all_b).unwrap();
}
*/

/*
fn main() {
    println!("{}", std::mem::size_of::<PlaybackEngine>());
}

fn cut(dst: &mut [u8], src: &[u8]) {
    let step = src.len() / dst.len();

    let mut i = 0;

    for x in dst.iter_mut() {
        *x = src[i];
        i += step;
    }
}
*/

/*
// Linearly resample from src_freq to dst_freq
fn resample(dst_freq: u32, src: &[u8], src_freq: u32) -> Vec<u8> {
    let mut v = Vec::new();

    let step = src_freq as f32 / dst_freq as f32;

    let mut iacc = 0_f32;

    while (iacc as usize) < src.len() {
        v.push(src[iacc as usize]);
        iacc += step;
    }

    v
}
*/
