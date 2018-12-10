extern crate chrono;
extern crate clap;
extern crate num_rational;
extern crate rexiv2;

use std::path::Path;
use std::fs::{self, ReadDir};
use std::cmp::Ordering;

use chrono::{Utc, TimeZone};
use clap::{Arg, App};
use num_rational::Ratio;
use rexiv2::Metadata;

#[derive(Eq)]
struct Timestamp {
    millis: i64
}

impl Timestamp {
    fn from_string(s: &str) -> Timestamp {
        let maybe_dt = Utc.datetime_from_str(s, "%Y:%m:%d %H:%M:%S");
        let millis = maybe_dt
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(0);
        Timestamp { millis: millis, }
    }
}

impl PartialEq for Timestamp {
    fn eq(&self, other: &Timestamp) -> bool {
        self.millis == other.millis
    }
}

struct ImgRef {
    path_string: String,
    created: Timestamp,
    width: i32,
    height: i32,
    exposure_time: Ratio<i32>,
    fnumber: f64,
}

impl ImgRef {

    fn from_path(path: &Path) -> Result<ImgRef, std::io::Error> {
        let s = path.to_str().unwrap();
        let meta = Metadata::new_from_path(path).unwrap();
        let created = match meta.get_tag_string("Exif.Image.DateTime") {
            Ok(s) => s,
            Err(_) => "1970:01:01 00:00:00".to_owned(),
        };
        Ok(
            ImgRef {
                path_string: s.to_owned(),
                created: Timestamp::from_string(&created),
                width: meta.get_pixel_width(),
                height: meta.get_pixel_height(),
                exposure_time: meta.get_exposure_time().unwrap_or(Ratio::new(0, 1)),
                fnumber: meta.get_fnumber().unwrap_or(0.0),
            }
        )
    }

    fn debug_string(&self) -> String {
        format!("{} ({} x {}) <created: {}>",
                self.path_string,
                self.width,
                self.height,
                self.created.millis)
    }
}

impl PartialEq for ImgRef {
    fn eq(&self, other: &ImgRef) -> bool {
        self.created == other.created &&
        self.width == other.width &&
        self.height == other.height &&
        self.fnumber == other.fnumber &&
        self.exposure_time == other.exposure_time
    }
}

impl Eq for ImgRef {}

impl PartialOrd for ImgRef {
    fn partial_cmp(&self, other: &ImgRef) -> Option<Ordering> {
        self.created.millis.partial_cmp(&other.created.millis)
    }
}

impl Ord for ImgRef {
    fn cmp(&self, other: &ImgRef) -> Ordering {
        self.created.millis.cmp(&other.created.millis)
    }
}

struct ImgRefIter {
    read_dir: ReadDir,
    waiting: Vec<ReadDir>,
}

impl ImgRefIter {
    fn from_path<'a>(path: &'a Path) -> Result<ImgRefIter, std::io::Error> {
        let rd = fs::read_dir(path).unwrap();

        Ok(
            ImgRefIter { read_dir: rd, waiting: Vec::new() }
        )
    }
}

impl Iterator for ImgRefIter {
    type Item = ImgRef;

    fn next(&mut self) -> Option<Self::Item> {
        let maybe_dir_entry = self.read_dir.next();

        match maybe_dir_entry {

            None => if self.waiting.is_empty() {
                None
            } else {
                self.read_dir = self.waiting.pop().unwrap();
                self.next()
            },
            Some(Err(err)) => {
                println!("{}", err);
                self.next()
            },
            Some(Ok(dir_entry)) => {
                let path_buf = dir_entry.path();
                let path = path_buf.as_path();

                let is_jpg = path
                    .to_str().unwrap()
                    .to_ascii_lowercase()
                    .ends_with(".jpg");

                if path.is_dir() {
                    let p = fs::read_dir(path);
                    match p {
                        Ok(subtree) => self.waiting.push(subtree),
                        Err(e) => println!("{}", e),
                    }
                    self.next()
                } else if is_jpg {
                    Some(ImgRef::from_path(&path).unwrap())
                } else {
                    self.next()
                }
            },
        }
    }
}

fn main() {

    let matches = App::new("imgdups")
                    .author("Nat Wilson")
                    .about("Finds likely JPEG duplicates")
                    .arg(Arg::with_name("ROOT")
                         .required(true)
                         .index(1))
                    .get_matches();

    let root = matches.value_of("ROOT").unwrap();

    let mut images: Vec<ImgRef> = Vec::new();
    let img_ref_iter = ImgRefIter::from_path(Path::new(&root));
    match img_ref_iter {
        Ok(iter) => for img in iter {
            images.push(img);
        },
        Err(e) => println!("{}", e),
    }

    images.sort();
    println!("found {} images", images.len());

    let mut i = 0;
    while i != images.len() - 1 {
        if images[i] == images[i+1] {
            println!("possible duplicates:\n  {}\n  {}",
                     images[i].debug_string(),
                     images[i+1].debug_string())
        }
        i = i + 1;
    }
}
