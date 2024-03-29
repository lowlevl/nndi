use super::FrameKind;

/// An implementation of the _scrambling_ & _unscrambling_
/// mechanism present in the protocol.
///
/// Heavily inspired by the work done by the VLC team
/// on their **libndi**, [see code](https://code.videolan.org/jbk/libndi/-/blob/c14b40caafb26a02249f062e7f907ceaa53c1b74/libndi.c#L48-L175).
pub enum Scrambler {
    Type1,
    Type2,
}

impl Scrambler {
    /// Detect the scrambler algorithm from the version and the [`FrameType`].
    pub fn new(kind: &FrameKind, version: u16) -> Self {
        match &kind {
            FrameKind::Video if version > 3 => Self::Type2,
            FrameKind::Audio | FrameKind::Text if version > 2 => Self::Type2,
            _ => Self::Type1,
        }
    }

    fn type1(buf: &mut [u8], seed: u32, scramble: bool) {
        let len = buf.len();

        let seed64 = ((seed as u64) << 32) | seed as u64;
        let mut seed1 = seed64 ^ 0xb711674bd24f4b24;
        let mut seed2 = seed64 ^ 0xb080d84f1fe3bf44;

        if len >= 8 {
            let mut temp;

            for chunk in buf.chunks_exact_mut(8) {
                let mut blob = u64::from_ne_bytes(
                    chunk
                        .try_into()
                        .expect("Vec::chunks_exact broke invariants"),
                );

                temp = seed1;
                seed1 = seed2;

                temp ^= temp << 23;
                temp = ((seed1 >> 9 ^ temp) >> 17) ^ temp ^ seed1;

                if scramble {
                    seed2 = temp ^ blob;
                    blob ^= temp.wrapping_add(seed1);
                } else {
                    blob ^= temp.wrapping_add(seed1);
                    seed2 = temp ^ blob;
                }

                chunk.copy_from_slice(&blob.to_ne_bytes());
            }
        }

        let remainder = &mut buf[len - len % 8..];
        if !remainder.is_empty() {
            let mut blob = remainder.to_vec();
            blob.resize(8, 0);
            let mut blob =
                u64::from_ne_bytes(blob.try_into().expect("Vec::resize broke invariants"));

            seed1 ^= seed1 << 23;
            seed1 = ((seed2 >> 9 ^ seed1) >> 17) ^ seed1 ^ seed2;
            blob ^= seed1.wrapping_add(seed2);

            remainder.copy_from_slice(&blob.to_ne_bytes()[..remainder.len()]);
        }
    }

    fn type2(buf: &mut [u8], mut seed: u32, scramble: bool) {
        const XOR_TABLE: &[u8] = &[
            0x4e, 0x44, 0x49, 0xae, 0x2c, 0x20, 0xa9, 0x32, 0x30, 0x31, 0x37, 0x20, 0x4e, 0x65,
            0x77, 0x54, 0x65, 0x6b, 0x2c, 0x20, 0x50, 0x72, 0x6f, 0x70, 0x72, 0x69, 0x65, 0x74,
            0x79, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x43, 0x6f, 0x6e, 0x66, 0x69, 0x64, 0x65, 0x6e,
            0x74, 0x69, 0x61, 0x6c, 0x2e, 0x20, 0x59, 0x6f, 0x75, 0x20, 0x61, 0x72, 0x65, 0x20,
            0x69, 0x6e, 0x20, 0x76, 0x69, 0x6f, 0x6c, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x6f,
            0x66, 0x20, 0x74, 0x68, 0x65, 0x20, 0x4e, 0x44, 0x49, 0xae, 0x20, 0x53, 0x44, 0x4b,
            0x20, 0x6c, 0x69, 0x63, 0x65, 0x6e, 0x73, 0x65, 0x20, 0x61, 0x74, 0x20, 0x68, 0x74,
            0x74, 0x70, 0x3a, 0x2f, 0x2f, 0x6e, 0x65, 0x77, 0x2e, 0x74, 0x6b, 0x2f, 0x6e, 0x64,
            0x69, 0x73, 0x64, 0x6b, 0x5f, 0x6c, 0x69, 0x63, 0x65, 0x6e, 0x73, 0x65, 0x2f, 0x00,
            0x00, 0x00,
        ];

        let len = buf.len();

        if scramble {
            for (idx, byte) in buf.iter_mut().enumerate().take(len.min(XOR_TABLE.len())) {
                *byte ^= XOR_TABLE[idx];
            }
        }

        if len >= 8 {
            let mut temp;

            for chunk in buf.chunks_exact_mut(8) {
                let mut blob = u64::from_le_bytes(
                    chunk
                        .try_into()
                        .expect("Vec::chunks_exact broke invariants"),
                );

                temp = seed as i64;
                let key = (temp
                    .wrapping_mul(len as i64)
                    .wrapping_mul(-0x61c8864680b583eb) as u64)
                    .wrapping_add(0xc42bd7dee6270f1b);

                if scramble {
                    blob = blob.wrapping_sub(0x2daa8c593b1b4591);
                    blob = (blob as i64).wrapping_mul(-0x61c8864680b583eb) as u64;
                    blob ^= key;

                    seed = (blob & 0xffffffff) as u32;
                } else {
                    seed = (blob & 0xffffffff) as u32;

                    blob ^= key;
                    blob = (blob as i64).wrapping_mul(-0xe217c1e66c88cc3) as u64;
                    blob = blob.wrapping_add(0x2daa8c593b1b4591);
                }

                chunk.copy_from_slice(&blob.to_le_bytes());
            }
        }

        if !scramble {
            for (idx, byte) in buf.iter_mut().enumerate().take(len.min(XOR_TABLE.len())) {
                *byte ^= XOR_TABLE[idx];
            }
        }
    }

    /// Unscramble the `buf` in-place from the provided `seed`.
    pub fn unscramble(&self, buf: &mut [u8], seed: u32) {
        match self {
            Self::Type1 => Self::type1(buf, seed, false),
            Self::Type2 => Self::type2(buf, seed, false),
        }
    }

    /// Scramble the `buf` in-place with the provided `seed`.
    pub fn scramble(&self, buf: &mut [u8], seed: u32) {
        match self {
            Self::Type1 => Self::type1(buf, seed, true),
            Self::Type2 => Self::type2(buf, seed, true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_unscrambles_type1() {
        let mut scrambled = [
            149, 6, 41, 151, 207, 93, 121, 59, 197, 203, 72, 41, 24, 15, 21, 178, 103, 230, 30,
            141, 146, 101, 1, 36, 91, 209, 55, 192, 74, 202, 221, 85, 170, 10, 12, 50, 206, 78, 6,
            245, 251, 208, 94, 149, 18, 215, 88, 195, 44, 238, 37, 124, 109, 186, 4, 230, 113, 172,
            99, 222, 42, 52, 36, 25, 171, 99, 145, 32, 129, 178, 177, 126, 132, 155, 7, 107, 25,
            174, 39, 175, 188,
        ];

        let expected = [
            0, 0, 0, 0, 0, 0, 0, 0, 60, 110, 100, 105, 95, 118, 101, 114, 115, 105, 111, 110, 32,
            116, 101, 120, 116, 61, 34, 51, 34, 32, 118, 105, 100, 101, 111, 61, 34, 53, 34, 32,
            97, 117, 100, 105, 111, 61, 34, 52, 34, 32, 115, 100, 107, 61, 34, 53, 46, 53, 46, 51,
            34, 32, 112, 108, 97, 116, 102, 111, 114, 109, 61, 34, 76, 73, 78, 85, 88, 34, 47, 62,
            0,
        ];

        let seed = scrambled.len() as u32;
        Scrambler::Type1.unscramble(&mut scrambled, seed);

        assert_eq!(scrambled, expected)
    }

    #[test]
    fn it_scrambles_type1() {
        let mut unscrambled = [
            0, 0, 0, 0, 0, 0, 0, 0, 60, 110, 100, 105, 95, 118, 101, 114, 115, 105, 111, 110, 32,
            116, 101, 120, 116, 61, 34, 51, 34, 32, 118, 105, 100, 101, 111, 61, 34, 53, 34, 32,
            97, 117, 100, 105, 111, 61, 34, 52, 34, 32, 115, 100, 107, 61, 34, 53, 46, 53, 46, 51,
            34, 32, 112, 108, 97, 116, 102, 111, 114, 109, 61, 34, 76, 73, 78, 85, 88, 34, 47, 62,
            0,
        ];
        let expected = [
            149, 6, 41, 151, 207, 93, 121, 59, 197, 203, 72, 41, 24, 15, 21, 178, 103, 230, 30,
            141, 146, 101, 1, 36, 91, 209, 55, 192, 74, 202, 221, 85, 170, 10, 12, 50, 206, 78, 6,
            245, 251, 208, 94, 149, 18, 215, 88, 195, 44, 238, 37, 124, 109, 186, 4, 230, 113, 172,
            99, 222, 42, 52, 36, 25, 171, 99, 145, 32, 129, 178, 177, 126, 132, 155, 7, 107, 25,
            174, 39, 175, 188,
        ];

        let seed = unscrambled.len() as u32;
        Scrambler::Type1.scramble(&mut unscrambled, seed);

        assert_eq!(unscrambled, expected)
    }

    #[test]
    fn it_unscrambles_type2() {
        let mut scrambled = [
            233, 100, 241, 22, 71, 237, 67, 214, 159, 81, 45, 219, 196, 196, 115, 101, 193, 217,
            237, 85, 56, 236, 206, 163, 253, 59, 47, 110, 105, 27, 24, 56, 91, 158, 144, 69, 143,
            214, 199, 1, 60, 218, 125, 131, 231, 185, 68, 99, 41, 157, 66, 169, 210, 17, 87, 62,
            105,
        ];
        let expected = [
            83, 72, 81, 50, 32, 2, 0, 0, 20, 2, 0, 0, 24, 0, 0, 0, 1, 0, 0, 0, 67, 164, 130, 63, 0,
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 122, 233, 121, 101, 0, 0, 0, 0, 62, 26,
            146, 204, 10, 124, 60, 0, 0,
        ];

        let seed = 58786;
        Scrambler::Type2.unscramble(&mut scrambled, seed);

        assert_eq!(scrambled, expected)
    }

    #[test]
    fn it_scrambles_type2() {
        let mut unscrambled = [
            83, 72, 81, 50, 32, 2, 0, 0, 20, 2, 0, 0, 24, 0, 0, 0, 1, 0, 0, 0, 67, 164, 130, 63, 0,
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 122, 233, 121, 101, 0, 0, 0, 0, 62, 26,
            146, 204, 10, 124, 60, 0, 0,
        ];
        let expected = [
            233, 100, 241, 22, 71, 237, 67, 214, 159, 81, 45, 219, 196, 196, 115, 101, 193, 217,
            237, 85, 56, 236, 206, 163, 253, 59, 47, 110, 105, 27, 24, 56, 91, 158, 144, 69, 143,
            214, 199, 1, 60, 218, 125, 131, 231, 185, 68, 99, 41, 157, 66, 169, 210, 17, 87, 62,
            105,
        ];

        let seed = 58786;
        Scrambler::Type2.scramble(&mut unscrambled, seed);

        assert_eq!(unscrambled, expected)
    }
}
