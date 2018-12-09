extern crate rexiv2;

use std::path::Path;
use std::fs::{self, ReadDir};

struct ImgRef {
    path_string: String,
}

impl ImgRef {

    fn from_path(path: &Path) -> Result<ImgRef, std::io::Error> {
        let s = path.to_str().unwrap();
        Ok(
            ImgRef {
                path_string: s.to_owned(),
            }
        )
    }

    fn metadata(&self) -> Result<rexiv2::Metadata, rexiv2::Rexiv2Error> {
        let path = Path::new(&self.path_string);
        rexiv2::Metadata::new_from_path(path)
    }

    fn create_timestamp(&self) -> i64 {
        // not implemented
        0
    }

    fn debug_string(&self) -> Result<String, rexiv2::Rexiv2Error> {
        let meta = self.metadata().unwrap();
        let media_type = meta.get_media_type().unwrap();
        Ok(
            format!("{} ({} x {}) {:?}",
                    self.path_string,
                    meta.get_pixel_width(),
                    meta.get_pixel_height(),
                    media_type)
        )
    }
}

fn are_identical(left: &ImgRef, right: &ImgRef) -> Result<bool, rexiv2::Rexiv2Error> {
    let left_meta = left.metadata().unwrap();
    let right_meta = right.metadata().unwrap();

    Ok(
        left.create_timestamp() == right.create_timestamp() &&
        left_meta.get_pixel_height() == right_meta.get_pixel_height() &&
        left_meta.get_pixel_width() == right_meta.get_pixel_width()
    )
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
    let root = Path::new("/home/nat/Pictures");
    let img_ref_iter = ImgRefIter::from_path(root);
    match img_ref_iter {
        Ok(iter) => for img in iter {
            println!("{:?}", img.debug_string())
        },
        Err(e) => println!("{}", e),
    }
}
