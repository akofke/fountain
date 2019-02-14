

pub fn morton3(x: f32, y: f32, z: f32) -> u32 {
    let x = to_fixed_point(x);
    let y = to_fixed_point(y);
    let z = to_fixed_point(z);

    let xx = expand_bits(x);
    let yy = expand_bits(y);
    let zz = expand_bits(z);

    (xx << 2) | (yy << 1) | zz

}

fn to_fixed_point(val: f32) -> u32 {
    debug_assert!(0.0 <= val && val < 1.0);

    // multiply by 2**n for n bits after the fixed point.
    // 10 bits in this case (for 3 to fit in a u32)
    (val * 1024.0).trunc() as u32
}

/// Expands a 10-bit u32 into 30 bits by inserting two 0s after each bit
fn expand_bits(val: u32) -> u32 {
    debug_assert!(val & !0x3FF == 0); // only 10 lowest bits of val are set
    let mut val = val;
    // Don't fully understand how the multiplication bit manip works, Maybe use the bit-shift version
    // instead so it's less magical
    val = (val.wrapping_mul(0x00010001)) & 0xFF0000FF;
    val = (val.wrapping_mul(0x00000101)) & 0x0F00F00F;
    val = (val.wrapping_mul(0x00000011)) & 0xC30C30C3;
    val = (val.wrapping_mul(0x00000005)) & 0x49249249;

    val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morton() {
        let x = 0.9999;
        let y = 0.0;
        let z = 0.9999;
        assert_eq!(morton3(x, y, z), 0b00_101101101101101101101101101101);
    }

    #[test]
    fn test_expand_bits() {
        let v = 0x3FF;
        assert_eq!(expand_bits(v), 0b00_001001001001001001001001001001u32);
    }

    #[test]
    fn test_fixed_point() {
        assert_eq!(to_fixed_point(0.99999), 0x3FF);
        assert_eq!(to_fixed_point(0.0), 0);
    }

}