extern crate curl;
extern crate serialize;

use curl::http;
use std::io::{mod, fs, File};
use std::io::fs::PathExtensions;
use std::str;
use serialize::{json, Decodable};

use std::sync::TaskPool;

#[deriving(Decodable)]
struct Crate {
    id: String,
    max_version: String,
}

#[deriving(Decodable)]
struct Crates { crates: Vec<Crate>, meta: CrateMeta }
#[deriving(Decodable)]
struct CrateMeta { total: uint }

const HOST: &'static str = "https://crates-io.herokuapp.com";

fn get<T: Decodable<json::Decoder, json::DecoderError>>(url: &str) -> T {
    let mut handle = http::Handle::new();

    println!("get -- {}", url);
    let resp = handle.get(url).exec().unwrap();
    let got = str::from_utf8(resp.get_body()).unwrap();
    json::decode(got).unwrap()
}

fn main() {
    let mut amt = 100000;
    let mut page = 1u;
    let base = "/api/v1/crates?per_page=100";
    let mut crates = Vec::new();
    while crates.len() < amt {
        let data = get::<Crates>(format!("{}{}&page={}", HOST, base,
                                         page).as_slice());
        page += 1;
        amt = data.meta.total;
        crates.extend(data.crates.into_iter());
    }

    let p = TaskPool::new(8);
    let (tx, rx) = channel();
    let mut amt = 0;
    for c in crates.iter() {
        let tx = tx.clone();
        let url = format!("{}/api/v1/crates/{}/{}/download", HOST, c.id,
                          c.max_version);
        let dst = Path::new(format!("dl/{}/{}-{}.crate", c.id, c.id,
                                    c.max_version));
        if dst.exists() {
            println!("skipping: {}", dst.display());
            continue
        }
        amt += 1;

        fs::mkdir_recursive(&dst.dir_path(), io::USER_DIR).unwrap();
        p.execute(proc() {
            let mut handle = http::Handle::new();
            println!("downloading: {}", dst.display());
            let resp = handle.get(url.as_slice()).follow_redirects(true)
                             .exec().unwrap();
            File::create(&dst).write(resp.get_body()).unwrap();
            tx.send(());
        });
    }
    rx.iter().take(amt).count();
}
