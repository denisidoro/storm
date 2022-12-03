use super::file::File;
use super::tree::TreeIndex;
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub struct FileMap {
    files: HashMap<TreeIndex, Vec<File>>,
    order: Vec<TreeIndex>,
}

impl FileMap {
    pub(super) fn new() -> Self {
        Self {
            files: HashMap::new(),
            order: vec![],
        }
    }

    pub fn from_lines(lines: &mut dyn Iterator<Item = String>) -> Result<Self> {
        let mut files = FileMap::new();

        let mut id = 0;

        for line in lines {
            if line.starts_with('=') {
                break;
            }

            if line.trim().is_empty() {
                continue;
            }

            if !line.contains(';') {
                id = line.trim().parse::<usize>()?;
                continue;
            }

            let file = File::from_line(&line)?;
            files.insert(&id, file);
        }

        Ok(files)
    }

    pub(super) fn insert(&mut self, k: &TreeIndex, v: File) -> usize {
        match self.files.get_mut(k) {
            Some(vec) => {
                let filename_lowercase = v.filename.to_lowercase();
                let existing_position = vec
                    .iter()
                    .position(|f| f.filename.to_lowercase() == filename_lowercase);
                match existing_position {
                    Some(p) => {
                        vec[p] = v;
                        p
                    }
                    None => {
                        vec.push(v);
                        vec.len() - 1
                    }
                }
            }
            None => {
                self.order.push(*k);
                self.files.insert(*k, vec![v]);
                0
            }
        }
    }

    pub fn get(&self, k: &TreeIndex) -> Option<&Vec<File>> {
        self.files.get(k)
    }

    pub fn ordered(&self, k: usize) -> Option<(usize, &Vec<File>)> {
        self.order
            .get(k)
            .map(|filemap_key| (*filemap_key, self.get(filemap_key).expect("Invalid filemap_key")))
    }

    // pub(super) fn get_file(&self, k: &TreeIndex, filename: &str) -> Option<&File> {
    //     let insensitive_filename = filename.trim_end_matches(".7z");
    //     match self.files.get(k) {
    //         None => None,
    //         Some(v) => v
    //             .iter()
    //             .find(|x| x.filename.trim_end_matches(".7z") == insensitive_filename),
    //     }
    // }
}

impl Display for FileMap {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for i in &self.order {
            let files = &self.files[i];

            fmt.write_str(&format!("{}\n", i))?;
            for file in files {
                fmt.write_str(&format!("{}\n", file))?;
            }
        }
        Ok(())
    }
}
