use std::{collections::BTreeMap, io::Write};

use bitstream_io::{BitWriter, LittleEndian, BitWrite};
use std::io::Result;
pub struct QRCAWriter {
    entries: BTreeMap<String, Vec<u8>>,
}
impl QRCAWriter {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
    pub fn add_entry(&mut self, path: impl ToString, data: Vec<u8>) {
        let mut path = path.to_string();
        if !path.starts_with("/") {
            path = format!("/{path}");
        }
        self.entries.insert(path, data);
    }
    pub fn write<W: Write>(self, w: W) -> Result<()> {
        let mut bitw = BitWriter::endian(w, LittleEndian);
        bitw.write_bytes(b"\0qrca")?;
        bitw.write(32, self.entries.len() as u32)?;
        let (es, data): (Vec<(String, usize)>, Vec<Vec<u8>>) = self.entries.clone().into_iter()
            .map(|(f, d)| {
                ((f, d.len()), d)
            })
            .unzip();
        for (filename, dlen) in es {
            bitw.write(16, filename.len() as u16)?;
            bitw.write_bytes(filename.as_bytes())?;
            bitw.write(64, dlen as u64)?;
        }
        for data in data {
            bitw.write_bytes(&data)?;
        }
        Ok(())
    }
}