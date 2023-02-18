use std::{fs::{File, self}, io::stdout};

use clap::{Command, arg};
use glob::glob;
use qrca::{reader::QRCAReader, Entry, writer::QRCAWriter};
fn cli() -> Command {
    Command::new("qrca-tools")
        .about("Tools for creating and reading QRCA files")
        .subcommand_required(true)
        .allow_external_subcommands(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("pack")
                .about("Pack a folder into a QRCA file")
                .arg(arg!(folder: <FOLDER>))
                .arg(arg!(out: [OUT]))
        )
        .subcommand(
            Command::new("list")
                .about("List QRCA file")
                .arg(arg!(file: <FILE>))
        )
        .subcommand(
            Command::new("read")
                .about("Read a file in QRCA file")
                .arg(arg!(file: <FILE>))
                .arg(arg!(qrcafile: <QRCAFILE>))
                .arg(arg!(--start -s [START]))
                .arg(arg!(--end -e [START]))
        )
        .subcommand(
            Command::new("info")
                .about("Get info about a file in QRCA file")
                .arg(arg!(file: <FILE>))
                .arg(arg!(qrcafile: <QRCAFILE>))
        )
        .subcommand(
            Command::new("unpack")
                .about("unpack a QRCA file")
                .arg(arg!(file: <FILE>))
        )
}

pub fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("list", sub_matches)) => {
            let file = sub_matches.get_one::<String>("file").expect("Couldn't get file from args").clone();
            let file = File::open(file).expect("Couldn't open file");
            let reader = QRCAReader::new(file).expect("Couldn't open qrca reader");
            serde_json::to_writer(stdout().lock(), &reader.entries()).expect("Couldn't send entries");
        },
        Some(("info", sub_matches)) => {
            let file = sub_matches.get_one::<String>("file").expect("Couldn't get file from args").clone();
            let file = File::open(file).expect("Couldn't open file");
            let qfile = sub_matches.get_one::<String>("qrcafile").expect("Couldn't get file in qrca file from args").clone();
            let mut reader = QRCAReader::new(file).expect("Couldn't open qrca reader");
            let er = reader.entry_by_path(qfile).expect("Couldn't get file in QRCA file");
            serde_json::to_writer(stdout().lock(), &Entry { size: er.size, path: er.path }).expect("Couldn't send entries");
        },
        Some(("read", sub_matches)) => {
            let file = sub_matches.get_one::<String>("file").expect("Couldn't get file from args").clone();
            let file = File::open(file).expect("Couldn't open file");
            let qfile = sub_matches.get_one::<String>("qrcafile").expect("Couldn't get file in qrca file from args").clone();
            let mut reader = QRCAReader::new(file).expect("Couldn't open qrca reader");
            let er = reader.entry_by_path(qfile).expect("Couldn't get file in QRCA file");
            let start: u64 = sub_matches.get_one::<String>("start").unwrap_or(&"0".to_string()).parse::<u64>().expect("Start wrong");
            let end: u64 = sub_matches.get_one::<String>("end").unwrap_or(&er.size.to_string()).parse::<u64>().expect("End wrong");
            let range = if sub_matches.contains_id("start") || sub_matches.contains_id("end") {
                Some(start as u64..end as u64)
            }else {
                None
            };
            er.read_to_write(stdout().lock(), range).unwrap();

        },
        Some(("pack", sub_matches)) => {
            let folder_s = sub_matches.get_one::<String>("folder").expect("Couldn't get folder from args").clone();
            let folder = glob(&format!("{}/*",folder_s)).expect("Couldn't get contents of the folder");
            let mut w = QRCAWriter::new();
            for file in folder.into_iter().filter(|f| f.is_ok()) {
                let file = file.unwrap();
                let f = fs::read(file.clone()).expect("Couldn't read file");
                let filename = file.file_name().unwrap().to_string_lossy();
                let filename = filename.as_ref();
                w.add_entry(format!("/{}",filename), f);
            }
            let folder = sub_matches.get_one::<String>("out").unwrap_or(&format!("{}.qrca", folder_s.trim_end_matches('/'))).clone();            
            let qf = File::create(folder).expect("Couldn't open output file");
            w.write(qf).expect("Couldn't write to file");
        },
        _ => unreachable!(),
    }
}