pub const FRQ_TBL: [i16; 12] = [
    261,278,294,311,329,349,371,391,414,440,466,494
];

pub const PAN_TBL: [i16; 13] = [
    0,43,86,129,172,215,256,297,340,383,426,469,512
];

pub const OCT_TBL: [i16; 8] = [
    32,64,64,128,128,128,128,128
];

pub fn org_key_to_freq(key: u8, a: i16) -> i32 {
    let (oct, pitch) = org_key_to_oct_pitch(key);

    let freq = FRQ_TBL[pitch as usize] as f32;
    let _oct  = OCT_TBL[oct as usize];

    // wave_size
    ((freq * 256.0 * f32::powi(2.0, oct as i32 - 3)) as i32) + (1000 - a as i32)
}

pub fn org_key_to_drum_freq(key: u8) -> i32  {
    key as i32 * 800 + 100
}

pub fn org_pan_to_pan(pan: u8) -> i32 {
    (PAN_TBL[pan as usize] as i32 - 256) * 10
}

pub fn org_vol_to_vol(vol: u8) -> i32 {
    (vol as i32 - 255) * 8
}

pub fn org_key_to_oct_pitch(key: u8) -> (u8, u8) {
    (key/12, key%12)
}