extern crate tar;
extern crate flate2;

use std::io::{fs, Command, File};
use std::io::fs::PathExtensions;
use std::sync::TaskPool;

fn main() {
    let p = TaskPool::new(8);

    for dir in fs::readdir(&Path::new("dl")).unwrap().into_iter() {
        compile(&dir);
    }
}

fn compile(p: &Path) {
    let files = fs::readdir(p).unwrap();
    for file in files.into_iter().filter(|p| p.is_file()) {
        if file.extension_str() != Some("crate") { continue }
        let dir = file.with_extension("");
        if dir.join(".ok").is_file() { continue }

        if !dir.join(".unpack").is_file() {
            let _ = fs::rmdir_recursive(&dir);
            let f = File::open(&file).unwrap();
            let stream = flate2::reader::GzDecoder::new(f).unwrap();
            let mut archive = tar::Archive::new(stream);
            archive.unpack(p).unwrap();
            File::create(&dir.join(".unpack")).unwrap();
        }

        let out = Command::new("cargo").arg("build").cwd(&dir).output().unwrap();
        if !out.status.success() {
            println!("bad {}", file.display());
            File::create(&dir.join(".out")).write(out.output.as_slice()).unwrap();
            File::create(&dir.join(".err")).write(out.error.as_slice()).unwrap();
        } else {
            File::create(&dir.join(".ok")).unwrap();
            println!("good {}", file.display());
        }
    }
}
