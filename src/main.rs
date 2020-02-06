mod bnk;
mod org;
mod playback;
mod stuff;
mod wav;

use crate::playback::PlaybackEngine;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, Write};

const BANK_DATA: &[u8] = include_bytes!("../assets/samples/Samples.bnk");

fn main() -> io::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    let file  = File::open(&args[0])?;
    let f     = BufReader::new(file);

    let org = org::Song::load_from(f)?;
    let bnk = bnk::SoundBank::load_from(BANK_DATA)?;

    let mut playback = PlaybackEngine::new(org, bnk);

    loop {
        let mut buf = vec![0x8080; 44100];

        playback.render_to(&mut buf);

        for frame in &buf {
            io::stdout().write_all(&frame.to_be_bytes()).unwrap();
        }
    }

    //Ok(())
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
