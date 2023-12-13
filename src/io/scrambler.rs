use super::FrameType;

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
        match self {
            Self::Type1 => unimplemented!(),
            Self::Type2 => {
                let len = buf.len();

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

                for idx in 0..len.min(Self::XOR_TABLE.len()) {
                    buf[idx] ^= Self::XOR_TABLE[idx];
                }
            }
        }
    }
}
