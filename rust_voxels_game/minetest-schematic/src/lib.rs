use std::{
    io::{self, ErrorKind, Read},
    mem, str,
};
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MTSNode {
    /// `content` in the specification, index into `MTS::node_names`
    pub name_id: u16,
    pub param1: u8,
    pub param2: u8,
}

impl MTSNode {
    pub const fn probability(self) -> u8 {
        self.param1 & 0x7F
    }
    pub const fn force_place(self) -> bool {
        self.param1 & 0x80 != 0
    }
}

#[derive(Debug, Clone)]
pub struct MTS {
    pub size_x: u16,
    pub size_y: u16,
    pub size_z: u16,
    pub node_names: Vec<Box<str>>,
    pub nodes: Vec<MTSNode>,
    pub y_slice_probabilities: Vec<u8>,
}

impl MTS {
    pub const SIGNATURE: [u8; 4] = *b"MTSM";
    pub const CURRENT_VERSION: u16 = 4;
    pub const MAX_NODE_COUNT: usize = isize::MAX as usize / mem::size_of::<MTSNode>();
    pub const fn valid_size(size_x: u16, size_y: u16, size_z: u16, max_node_count: usize) -> bool {
        let Some(total_nodes) = Self::try_node_count(size_x, size_y, size_z) else {
            return false;
        };
        total_nodes <= max_node_count
    }
    pub const fn try_node_count(size_x: u16, size_y: u16, size_z: u16) -> Option<usize> {
        let Some(x_mul_y) = (size_x as usize).checked_mul(size_y as usize) else {
            return None;
        };
        x_mul_y.checked_mul(size_z as usize)
    }
    pub const fn node_count(size_x: u16, size_y: u16, size_z: u16) -> usize {
        if let Some(retval) = Self::try_node_count(size_x, size_y, size_z) {
            retval
        } else {
            panic!("invalid node count");
        }
    }
    pub fn pos_to_node_index(&self, x: u16, y: u16, z: u16) -> usize {
        assert!(
            Self::valid_size(self.size_x, self.size_y, self.size_z, Self::MAX_NODE_COUNT),
            "size too big"
        );
        assert!(
            x < self.size_x && y < self.size_y && z < self.size_z,
            "position out of range"
        );
        let mut retval = 0usize;
        retval += z as usize;
        retval *= self.size_y as usize;
        retval += y as usize;
        retval *= self.size_x as usize;
        retval += x as usize;
        retval
    }
    fn read_bytes<const N: usize>(reader: &mut impl io::BufRead) -> Result<[u8; N], MTSError> {
        let mut buf = [0; N];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }
    fn read_u8(reader: &mut impl io::BufRead) -> Result<u8, MTSError> {
        Ok(Self::read_bytes::<1>(reader)?[0])
    }
    fn read_u16(reader: &mut impl io::BufRead) -> Result<u16, MTSError> {
        Ok(u16::from_be_bytes(Self::read_bytes(reader)?))
    }
    fn read_string(reader: &mut impl io::BufRead) -> Result<String, MTSError> {
        let len = Self::read_u16(reader)? as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf).map_err(|e| e.utf8_error())?)
    }
    pub fn read<R: io::BufRead>(reader: &mut R, max_node_count: usize) -> Result<MTS, MTSError> {
        let max_node_count = max_node_count.min(Self::MAX_NODE_COUNT);
        if Self::read_bytes(reader)? != Self::SIGNATURE {
            return Err(MTSError::InvalidSignature);
        }
        let version = Self::read_u16(reader)?;
        if version != Self::CURRENT_VERSION {
            return Err(MTSError::UnsupportedVersion { version });
        }
        let size_x = Self::read_u16(reader)?;
        let size_y = Self::read_u16(reader)?;
        let size_z = Self::read_u16(reader)?;
        if !Self::valid_size(size_x, size_y, size_z, max_node_count) {
            return Err(MTSError::SizeTooBig {
                size_x,
                size_y,
                size_z,
            });
        }
        let mut y_slice_probabilities = vec![0; size_y as usize];
        reader.read_exact(&mut y_slice_probabilities)?;
        let node_names_len = Self::read_u16(reader)?;
        let mut node_names = Vec::with_capacity(node_names_len.into());
        for _ in 0..node_names_len {
            node_names.push(Self::read_string(reader)?.into_boxed_str());
        }
        let mut reader = flate2::bufread::ZlibDecoder::new(reader);
        let node_count = Self::node_count(size_x, size_y, size_z);
        let mut buf = vec![0; node_count * 2];
        reader.read_exact(&mut buf)?;
        let mut nodes: Vec<MTSNode> = vec![
            MTSNode {
                name_id: 0,
                param1: 0,
                param2: 0
            };
            node_count
        ];
        let mut buf_reader = &*buf;
        for node in &mut nodes {
            node.name_id = Self::read_u16(&mut buf_reader)?;
            if node.name_id >= node_names_len {
                return Err(MTSError::NameIdOutOfRange {
                    name_id: node.name_id,
                    node_names_len,
                });
            }
        }
        buf.truncate(node_count);
        reader.read_exact(&mut buf)?;
        let mut buf_reader = &*buf;
        for node in &mut nodes {
            node.param1 = Self::read_u8(&mut buf_reader)?;
        }
        reader.read_exact(&mut buf)?;
        let mut buf_reader = &*buf;
        for node in &mut nodes {
            node.param2 = Self::read_u8(&mut buf_reader)?;
        }
        match reader.read_exact(&mut [0u8]) {
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {}
            e => {
                e?;
                return Err(MTSError::TooManyBytes);
            }
        }
        Ok(MTS {
            size_x,
            size_y,
            size_z,
            node_names,
            nodes,
            y_slice_probabilities,
        })
    }
}

#[derive(Debug, Error)]
pub enum MTSError {
    #[error("invalid MTS signature")]
    InvalidSignature,
    #[error("unsupported MTS version {version}")]
    UnsupportedVersion { version: u16 },
    #[error("MTS too big: ({size_x}, {size_y}, {size_z})")]
    SizeTooBig {
        size_x: u16,
        size_y: u16,
        size_z: u16,
    },
    #[error("Name Id (`content` field) is out of range: {name_id} not in 0..{node_names_len}")]
    NameIdOutOfRange { name_id: u16, node_names_len: u16 },
    #[error("too many bytes in decompressed schematic")]
    TooManyBytes,
    #[error(transparent)]
    Utf8Error(#[from] str::Utf8Error),
    #[error(transparent)]
    IoError(#[from] io::Error),
}

impl From<MTSError> for io::Error {
    fn from(value: MTSError) -> Self {
        match value {
            MTSError::Utf8Error(e) => io::Error::new(ErrorKind::InvalidData, e),
            MTSError::IoError(e) => e,
            MTSError::InvalidSignature
            | MTSError::UnsupportedVersion { .. }
            | MTSError::SizeTooBig { .. }
            | MTSError::NameIdOutOfRange { .. }
            | MTSError::TooManyBytes => io::Error::new(ErrorKind::InvalidData, value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        let bytes: &[u8] = &[
            0x4d, 0x54, 0x53, 0x4d, 0x00, 0x04, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x7f, 0x7f,
            0x00, 0x04, 0x00, 0x0d, 0x64, 0x65, 0x66, 0x61, 0x75, 0x6c, 0x74, 0x3a, 0x73, 0x74,
            0x6f, 0x6e, 0x65, 0x00, 0x0c, 0x64, 0x65, 0x66, 0x61, 0x75, 0x6c, 0x74, 0x3a, 0x64,
            0x69, 0x72, 0x74, 0x00, 0x03, 0x61, 0x69, 0x72, 0x00, 0x17, 0x64, 0x65, 0x66, 0x61,
            0x75, 0x6c, 0x74, 0x3a, 0x64, 0x69, 0x72, 0x74, 0x5f, 0x77, 0x69, 0x74, 0x68, 0x5f,
            0x67, 0x72, 0x61, 0x73, 0x73, 0x78, 0x9c, 0x63, 0x60, 0x00, 0x03, 0x46, 0x20, 0x64,
            0x62, 0x60, 0x66, 0x60, 0xaa, 0x87, 0x02, 0x06, 0x28, 0x00, 0x00, 0x32, 0x71, 0x04,
            0x02,
        ];
        let mts = MTS::read(&mut { bytes }, MTS::MAX_NODE_COUNT).unwrap();
        println!("{mts:#?}");
        assert_eq!(
            format!("\n{mts:#?}\n"),
            r#"
MTS {
    size_x: 2,
    size_y: 2,
    size_z: 2,
    node_names: [
        "default:stone",
        "default:dirt",
        "air",
        "default:dirt_with_grass",
    ],
    nodes: [
        MTSNode {
            name_id: 0,
            param1: 127,
            param2: 0,
        },
        MTSNode {
            name_id: 0,
            param1: 127,
            param2: 0,
        },
        MTSNode {
            name_id: 0,
            param1: 127,
            param2: 0,
        },
        MTSNode {
            name_id: 1,
            param1: 127,
            param2: 0,
        },
        MTSNode {
            name_id: 1,
            param1: 127,
            param2: 0,
        },
        MTSNode {
            name_id: 2,
            param1: 127,
            param2: 0,
        },
        MTSNode {
            name_id: 3,
            param1: 127,
            param2: 0,
        },
        MTSNode {
            name_id: 2,
            param1: 127,
            param2: 0,
        },
    ],
    y_slice_probabilities: [
        127,
        127,
    ],
}
"#
        );
    }
}
