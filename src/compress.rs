//! Simple compression scheme for the RM2 framebuffer.

use std::convert::TryInto;

const MARKER: u8 = 0xAB;

#[cfg(target_arch = "arm")]
fn compress8_arm(buf: &[u8; 8]) -> u8 {
    let mask: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
    let mut gt_vec2: [u32; 2] = [0, 0];
    let mut lt_vec2: [u32; 2] = [0, 0];

    unsafe {
        asm!(
            "vmov.i8 d0, #0",
            "vmov.i8 d1, #255",
            "vld1.8 d2, [{0}]",
            "vld1.8 d5, [{1}]",
            "vcgt.u8 d3, d2, d0", // gt = vec > 0
            "vclt.u8 d4, d2, d1", // lt = vec < 255
            "vand d3, d5, d3",    // "shift" gt elements
            "vand d4, d5, d4",    // "shift" lt elements
            "vpaddl.u8 d3, d3",   // gt: combine 8 u8 to 4 u16
            "vpaddl.u8 d4, d4",   // lt: combine 8 u8 to 4 u16
            "vpaddl.u16 d3, d3",  // gt: combine 4 u16 to 2 u32
            "vpaddl.u16 d4, d4",  // lt: combine 4 u16 to 2 u32
            "vst1.8 d3, [{2}]",
            "vst1.8 d4, [{3}]",
            in(reg) buf.as_ptr(), in(reg) mask.as_ptr(),
            in(reg) gt_vec2.as_mut_ptr(), in(reg) lt_vec2.as_mut_ptr(),
            options(nostack),
        )
    }

    let gt = gt_vec2[0] | gt_vec2[1];
    let lt = lt_vec2[0] | lt_vec2[1];

    // any element that is not 0 or 255?
    if gt & lt != 0 {
        MARKER
    } else {
        gt as u8
    }
}

fn compress8(buf: &[u8; 8]) -> u8 {
    let mut gt: u8 = 0;
    let mut lt: u8 = 0;
    for (i, v) in buf.iter().enumerate() {
        gt |= ((*v > 0)   as u8) << i;
        lt |= ((*v < 255) as u8) << i;
    }

    // any element that is not 0 or 255?
    if gt & lt != 0 {
        MARKER
    } else {
        gt
    }
}

pub fn compress_buf_slow(buf: &[u8], out: &mut Vec<u8>) {
    for chunk in buf.chunks(8) {
        let v = compress8(chunk.try_into().unwrap());
        out.push(v);
        if v == MARKER {
            for b in chunk {
                out.push(*b);
            }
        }
    }
}

#[cfg(target_arch = "arm")]
pub fn compress_buf_fast(buf: &[u8], out: &mut Vec<u8>) {
    for chunk in buf.chunks(8) {
        let v = compress8_arm(chunk.try_into().unwrap());
        out.push(v);
        if v == MARKER {
            for b in chunk {
                out.push(*b);
            }
        }
    }
}
