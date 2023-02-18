use std::{io::{Read, Seek, self, Write}, ops::Range};

use bitstream_io::{BitReader, LittleEndian, BitRead};

use anyhow::Result;

use crate::Entry;
pub struct QRCAReader<R> {
    entries: Vec<Entry>,
    data_start: u64,
    reader: R,
}
impl<'a, R: Read + Seek> QRCAReader<R> {
    pub fn new(mut reader: R) -> Result<Self> {
        let mut bitr = BitReader::endian(&mut reader, LittleEndian);
        if &bitr.read_to_vec(5)? != b"\0qrca" {
            io::Result::<()>::Err(io::Error::new(io::ErrorKind::Other, "Wrong magic code!".to_string()))?;
        }
        let entries = (0..bitr.read::<u32>(32)?).map(|_| {
            let path = {
                let len = bitr.read::<u32>(16)? as usize;
                let bytes = bitr.read_to_vec(len)?;
                std::str::from_utf8(&bytes)?.to_owned()
            };
            let size = bitr.read::<u64>(64)?;
            Ok(crate::Entry { path, size})
        })
        .filter_map(|e: Result<Entry>| {
            if let Ok(e) = e {
                Some(e)
            }else{
                None
            }
        })
        .collect();
        let start = reader.seek(std::io::SeekFrom::Current(0))?;
        Ok(Self {
            entries,
            reader,
            data_start: start,
        })
    }
    pub fn entry(&'a mut self, entry_id: usize) -> Result<EntryReader<'a, R>> {
        let mut c = self.data_start;
        for ent in &(&self.entries)[0..entry_id] {
            c+=ent.size; 
        }
        let e = self.entries.get(entry_id).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Couldn't get entry!".to_string()))?;
        Ok(EntryReader::new(&mut self.reader, e.size, c, e.path.clone()))
    }
    pub fn entry_by_path(&'a mut self, path: impl ToString) -> Result<EntryReader<'a, R>> {
        let mut id = 0;
        let path = path.to_string();
        for e in self.entries.clone() {
            if e.path == path {
                return self.entry(id);
            }
            id += 1;
        }
        io::Result::<()>::Err(io::Error::new(io::ErrorKind::Other, "Couldn't get entry!".to_string()))?;
        unreachable!()
    }
    pub fn entries(&self) -> Vec<Entry> {
        self.entries.clone()
    }
}

pub struct EntryReader<'a, R> {
    reader: &'a mut R,
    data_pos_start: u64,
    pub size: u64,
    pub path: String,
}
impl<'a, R: Read + Seek> EntryReader<'a, R> {
    const BLKSIZE: u64 = 10^6;
    pub fn new(reader: &'a mut R, size: u64, start: u64, path: String) -> Self {
        Self { reader, data_pos_start: start, size, path }
    }
    pub fn read(self, range: Option<Range<u64>>) -> Result<Vec<u8>> {
        let range = range.clone().unwrap_or(0..self.size);
        let rl = range.end-range.start;
        let mut buf = vec![0u8; rl as usize];
        self.reader.seek(io::SeekFrom::Start(self.data_pos_start + range.start))?;
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }
    pub fn read_to_write<W: Write>(self, mut w: W, range: Option<Range<u64>>) -> Result<()> {
        let range = range.clone().unwrap_or(0..self.size);
        let rl = range.end-range.start;
        self.reader.seek(io::SeekFrom::Start(self.data_pos_start+range.start))?;
        let blks = rl/Self::BLKSIZE;
        let lb = rl%Self::BLKSIZE;
        for _ in 0..blks {
            let mut buf = vec![0u8; Self::BLKSIZE as usize];
            self.reader.read_exact(&mut buf)?;
            w.write_all(&buf)?;
        }
        if lb > 0 {
            let mut buf = vec![0u8; lb as usize];
            self.reader.read_exact(&mut buf)?;
            w.write_all(&buf)?;
        }
        Ok(())
    }
}