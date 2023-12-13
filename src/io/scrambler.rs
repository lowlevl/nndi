use super::FrameType;

#[cfg(doc)]
use super::Frame;

/// An implementation of the _scrambling_ & _unscrambling_
/// mechanism present in [`Frame`]s.
///
/// Heavily inspired by the work done by the VLC team
/// on their **libndi**, [see code](https://code.videolan.org/jbk/libndi/-/blob/c14b40caafb26a02249f062e7f907ceaa53c1b74/libndi.c#L48-L175).
pub enum Scrambler {
    Type1,
    Type2,
}

impl Scrambler {
    pub const XOR_TABLE: &'static [u8] = &[
        0x4e, 0x44, 0x49, 0xae, 0x2c, 0x20, 0xa9, 0x32, 0x30, 0x31, 0x37, 0x20, 0x4e, 0x65, 0x77,
        0x54, 0x65, 0x6b, 0x2c, 0x20, 0x50, 0x72, 0x6f, 0x70, 0x72, 0x69, 0x65, 0x74, 0x79, 0x20,
        0x61, 0x6e, 0x64, 0x20, 0x43, 0x6f, 0x6e, 0x66, 0x69, 0x64, 0x65, 0x6e, 0x74, 0x69, 0x61,
        0x6c, 0x2e, 0x20, 0x59, 0x6f, 0x75, 0x20, 0x61, 0x72, 0x65, 0x20, 0x69, 0x6e, 0x20, 0x76,
        0x69, 0x6f, 0x6c, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65,
        0x20, 0x4e, 0x44, 0x49, 0xae, 0x20, 0x53, 0x44, 0x4b, 0x20, 0x6c, 0x69, 0x63, 0x65, 0x6e,
        0x73, 0x65, 0x20, 0x61, 0x74, 0x20, 0x68, 0x74, 0x74, 0x70, 0x3a, 0x2f, 0x2f, 0x6e, 0x65,
        0x77, 0x2e, 0x74, 0x6b, 0x2f, 0x6e, 0x64, 0x69, 0x73, 0x64, 0x6b, 0x5f, 0x6c, 0x69, 0x63,
        0x65, 0x6e, 0x73, 0x65, 0x2f, 0x00, 0x00, 0x00,
    ];

    pub fn identify(version: u16, message_type: &FrameType) -> Self {
        match message_type {
            FrameType::Video if version > 3 => Self::Type2,
            FrameType::Audio | FrameType::Text if version > 2 => Self::Type2,
            _ => Self::Type1,
        }
    }

    pub fn unscramble(&self, buf: &mut [u8], mut seed: u32) {
        let len = buf.len();

        match self {
            Self::Type1 => {
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
                        blob ^= temp.wrapping_add(seed1);
                        seed2 = temp ^ blob;

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
                    blob ^= seed1 + seed2;

                    remainder.copy_from_slice(&blob.to_ne_bytes()[..remainder.len()]);
                }
            }
            Self::Type2 => {
                if len >= 8 {
                    let mut temp;

                    for chunk in buf.chunks_exact_mut(8) {
                        let mut blob = u64::from_ne_bytes(
                            chunk
                                .try_into()
                                .expect("Vec::chunks_exact broke invariants"),
                        );

                        temp = seed as i64;
                        seed = (blob & 0xffffffff) as u32;

                        blob = ((((temp
                            .wrapping_mul(len as i64)
                            .wrapping_mul(-0x61c8864680b583eb)
                            as u64)
                            .wrapping_add(0xc42bd7dee6270f1b)
                            ^ blob) as i64)
                            .wrapping_mul(-0xe217c1e66c88cc3)
                            as u64)
                            .wrapping_add(0x2daa8c593b1b4591);

                        chunk.copy_from_slice(&blob.to_ne_bytes());
                    }
                }

                for (idx, byte) in buf
                    .iter_mut()
                    .enumerate()
                    .take(len.min(Self::XOR_TABLE.len()))
                {
                    *byte ^= Self::XOR_TABLE[idx];
                }
            }
        }
    }
}
