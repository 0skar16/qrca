use std::io::Cursor;

use rand::Rng;

use crate::{writer::QRCAWriter, reader::QRCAReader};

#[test]
fn write_read_test() {
    let mut aw = QRCAWriter::new();
    let rb1: &[u8] = &rand::thread_rng().gen::<[u8; 32]>();
    aw.add_entry("/test.txt", rb1.to_vec());
    let rb2: &[u8] = &rand::thread_rng().gen::<[u8; 32]>();
    aw.add_entry("enb.xtstd", rb2.to_vec());
    let mut bytes = vec![];
    aw.write(&mut bytes).unwrap();
    let mut ar = QRCAReader::new(Cursor::new(bytes)).unwrap();
    let e = ar.entry_by_path("/test.txt").unwrap();
    assert_eq!(e.read(None).unwrap(), rb1);
    let e = ar.entry_by_path("/enb.xtstd").unwrap();
    assert_eq!(e.read(None).unwrap(), rb2);

}