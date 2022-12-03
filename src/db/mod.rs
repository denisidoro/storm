pub mod file;
mod filemap;
mod header;
mod tree;

use self::file::Kb;
use self::header::Header;
use self::tree::TreeIndex;
use crate::fs::{self, IPathBuf};
use crate::smalldate::SmallDate;
use anyhow::{Context, Result};
use file::File;
use filemap::FileMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tree::Tree;
use walkdir::{DirEntry, WalkDir};

pub struct Db {
    header: Header,
    tree: Tree,
    filemap: FileMap,
}

impl Display for Db {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let db_sorted = self.sort().map_err(|_| fmt::Error)?;
        let txt = format!(
            "{}=\n{}=\n{}",
            db_sorted.header, db_sorted.tree, db_sorted.filemap
        );
        fmt.write_str(txt.trim_end())?;

        Ok(())
    }
}

impl Db {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            tree: Tree::new(),
            filemap: FileMap::new(),
        }
    }

    pub fn add_tag(&mut self, k: String, v: String) {
        self.header.tags.insert(k, v);
    }

    pub fn add_file(&mut self, filepath: &Path, prefix_to_strip: &Path) -> Result<()> {
        let (kb, date) = fs::metadata(filepath)?;
        let relative_filepath = filepath.strip_prefix(prefix_to_strip)?;
        self.add(relative_filepath, kb, date)
    }

    pub fn add(&mut self, path: &Path, kb: Kb, date: SmallDate) -> Result<()> {
        let parent = path.parent().context("Invalid parent")?;
        let filename = path
            .file_name()
            .context("Invalid filename")?
            .to_string_lossy()
            .to_string();
        let parent_id = self.tree.add_path(parent)?;
        let file = File {
            filename,
            kb: Some(kb),
            date: Some(date),
        };
        self.filemap.insert(&parent_id, file);
        Ok(())
    }

    fn from_path_lines(lines: &mut dyn Iterator<Item = String>) -> Result<Self> {
        let header = Header::new();
        let mut tree = Tree::new();
        let mut filemap = FileMap::new();

        for line in lines {
            let path = PathBuf::from(line.trim());
            let parent = path.parent().context("no parent")?;
            let index = tree.add_path(parent)?;
            let filename = path
                .file_name()
                .context("no filename")?
                .to_string_lossy()
                .to_string();
            let kb = None;
            let date = None;
            let file = File { filename, kb, date };
            filemap.insert(&index, file);
        }
        Ok(Self {
            header,
            tree,
            filemap,
        })
    }

    fn from_lines(lines: &mut dyn Iterator<Item = String>) -> Result<Self> {
        let header = Header::from_lines(lines)?;
        let tree = Tree::from_lines(lines)?;
        let filemap = FileMap::from_lines(lines)?;
        Ok(Self {
            header,
            tree,
            filemap,
        })
    }

    pub fn iter(&self) -> DbIter {
        DbIter {
            key_id: 0,
            vec_id: 0,
            filemap: &self.filemap,
        }
    }

    pub fn entry_pair(&self, entry: &DbEntry) -> (PathBuf, &File) {
        let path = self.tree.path(entry.parent_id).expect("Invalid parent_id");
        let (_, vec) = self.filemap.ordered(entry.key_id).expect("Invalid key_id");
        let file = vec.get(entry.vec_id).expect("Invalid vec_id");

        (path, file)
    }

    fn sort_value(&self, entry: &DbEntry) -> String {
        let (mut path, file) = self.entry_pair(entry);
        path.push(&file.filename);
        path.to_string().to_lowercase()
    }

    fn hashes(&self) -> Vec<u64> {
        self.iter()
            .map(|entry| {
                let (mut path, file) = self.entry_pair(&entry);
                path.push(&file.filename);
                hash(&path.to_string())
            })
            .collect()
    }

    // fn get_file(&self, path: &Path) -> Option<&File> {
    //     let parent = path.parent().expect("no parent");
    //     let filename = path
    //         .file_name()
    //         .expect("invalid filename")
    //         .to_string_lossy()
    //         .to_string();

    //     let index = match self.tree.find_path(parent) {
    //         None => return None,
    //         Some(i) => i,
    //     };

    //     self.filemap.get_file(&index, &filename)
    // }

    fn sort(&self) -> Result<Self> {
        let header = self.header.clone();
        let mut tree = Tree::new();
        let mut filemap = FileMap::new();

        let mut entries: Vec<DbEntry> = self.iter().collect();
        entries.sort_by(|a, b| {
            let a_str = self.sort_value(a);
            let b_str = self.sort_value(b);
            a_str.cmp(&b_str)
        });

        for entry in entries {
            let (parent, file) = self.entry_pair(&entry);
            let new_parent_id = tree.add_path(&parent)?;
            filemap.insert(&new_parent_id, file.clone());
        }

        let db = Db {
            header,
            tree,
            filemap,
        };
        Ok(db)
    }
}

pub struct DbIter<'a> {
    key_id: TreeIndex,
    vec_id: usize,
    filemap: &'a FileMap,
}

pub struct DbEntry {
    parent_id: TreeIndex,
    key_id: TreeIndex,
    vec_id: usize,
}

impl<'a> DbIter<'a> {
    fn next_helper(&mut self, fallback: bool) -> Option<DbEntry> {
        match self.filemap.ordered(self.key_id) {
            None => None,
            Some((parent_id, vec)) => match vec.get(self.vec_id) {
                None => {
                    if fallback {
                        None
                    } else {
                        self.key_id += 1;
                        self.vec_id = 0;
                        self.next_helper(true)
                    }
                }
                Some(_) => {
                    let entry = DbEntry {
                        parent_id,
                        key_id: self.key_id,
                        vec_id: self.vec_id,
                    };
                    self.vec_id += 1;
                    Some(entry)
                }
            },
        }
    }
}

impl<'a> Iterator for DbIter<'a> {
    type Item = DbEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_helper(false)
    }
}

fn hash(txt: &str) -> u64 {
    let mut hasher = DefaultHasher::new();

    txt.trim_start_matches('/')
        .trim_end_matches(".7z")
        .to_lowercase()
        .hash(&mut hasher);

    hasher.finish()
}

pub fn read(path: &Path) -> Result<Db> {
    let mut lines = fs::read_lines(path)?;
    Db::from_lines(&mut lines)
}

pub fn read_list(path: &Path) -> Result<Db> {
    let mut lines = fs::read_lines(path)?;
    Db::from_path_lines(&mut lines)
}

pub fn write(db: Db, path: &Path) -> Result<()> {
    let serialized = db.to_string();
    fs::write(path, &serialized)?;
    Ok(())
}

fn is_db(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".storm.txt"))
        .unwrap_or(false)
}

// pub fn get_file(db_folder: &Path, path: &Path) -> Option<(Header, File)> {
//     let walker = WalkDir::new(db_folder).into_iter();
//     for entry in walker.filter_map(|e| e.ok()) {
//         if !is_db(&entry) {
//             continue;
//         }
//         let mut lines = fs::read_lines(entry.path()).expect("invalid file");
//         let db = Db::from_lines(&mut lines).expect("Invalid db");
//         if let Some(file) = db.get_file(path) {
//             let header = db.header.clone();
//             return Some((header, file.clone()));
//         }
//     }
//
//     None
// }

pub fn all_hashes(db_folder: &Path) -> Vec<u64> {
    let mut hashes = vec![];

    let walker = WalkDir::new(db_folder).into_iter();
    for entry in walker.filter_map(|e| e.ok()) {
        if !is_db(&entry) {
            continue;
        }
        let mut lines = fs::read_lines(entry.path()).expect("invalid file");
        let db = Db::from_lines(&mut lines).expect("Invalid db");
        hashes.append(&mut db.hashes());
    }

    hashes.sort_unstable();
    hashes
}

pub fn has(path: &Path, hashes: &[u64]) -> bool {
    let needle = hash(&path.to_string());
    hashes.binary_search(&needle).is_ok()
}

#[cfg(test)]
mod tests {
    use crate::smalldate::SmallDate;

    use super::*;

    const SERIALIZED_UNSORTED: &str = "foo: lorem
bar: ipsum
=
dcim
  camera
books
  fiction
  horror
=
2
mountain.jpg;456;211230
beach and sand.jpg;123;211229
5
monster.txt;22;930606
4
spaceship.txt;44;201030";

    const SERIALIZED: &str = "bar: ipsum
foo: lorem
=
books
  fiction
  horror
dcim
  camera
=
2
spaceship.txt;44;201030
3
monster.txt;22;930606
5
beach and sand.jpg;123;211229
mountain.jpg;456;211230";

    #[test]
    fn test_tree() -> Result<()> {
        let mut lines = SERIALIZED.lines().map(|x| x.to_owned());
        let db = Db::from_lines(&mut lines)?;

        assert_eq!(
            db.tree.path(5)?.to_string_lossy().to_string(),
            "dcim/camera".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_deser() -> Result<()> {
        let mut lines = SERIALIZED_UNSORTED.lines().map(|x| x.to_owned());

        let db0 = Db::from_lines(&mut lines)?;
        let db0_str = db0.to_string();
        assert_eq!(db0_str.trim(), SERIALIZED.trim());

        let mut db0_lines = db0_str.lines().map(|x| x.to_owned());
        let db = Db::from_lines(&mut db0_lines)?;
        assert_eq!(db.to_string().trim(), SERIALIZED.trim());

        Ok(())
    }

    #[test]
    fn test_idempotency() -> Result<()> {
        let mut lines = SERIALIZED.lines().map(|x| x.to_owned());
        let mut db = Db::from_lines(&mut lines)?;

        let path = PathBuf::from("books/fiction/spaceship.txt");
        let kb = 123;
        let date = SmallDate::from_str("941231")?;
        db.add(&path, kb, date)?;

        let expected = SERIALIZED
            .trim()
            .replace("spaceship.txt;44;201030", "spaceship.txt;123;941231");

        assert_eq!(db.to_string().trim(), expected);

        Ok(())
    }

    // #[test]
    // fn test_getfile() -> Result<()> {
    //     let mut lines = SERIALIZED.lines().map(|x| x.to_owned());
    //     let db = Db::from_lines(&mut lines)?;

    //     let path = PathBuf::from("books/fiction/spaceship.txt");
    //     assert_eq!(db.get_file(&path).unwrap().filename, "spaceship.txt");

    //     let path = PathBuf::from("books/fiction/alien.txt");
    //     assert!(db.get_file(&path).is_none());

    //     Ok(())
    // }

    #[test]
    fn test_getfile() -> Result<()> {
        let mut lines = SERIALIZED.lines().map(|x| x.to_owned());
        let db = Db::from_lines(&mut lines)?;

        let mut hashes = db.hashes();
        hashes.sort_unstable();

        let path = PathBuf::from("books/fiction/alien.txt");
        assert!(!has(&path, &hashes));

        let path = PathBuf::from("books/fiction/spaceship.txt");
        assert!(has(&path, &hashes));

        Ok(())
    }
}
