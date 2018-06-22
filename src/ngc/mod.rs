use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::error;
use std::collections::HashMap;

/* .NGC element
 * Format (one on each line of the file):
 * <ID> "FILENAME"
 */
pub struct NgcElement {
    pub filename: String,
    pub id: i32,
}

/* .NGC archive
 * Contains multiple NGC elements
 */
pub struct NgcArchive {
    pub data: Vec<NgcElement>,
}

impl NgcArchive {
    pub fn gen_id_lookup<'a>(&'a self) -> HashMap<i32, &'a NgcElement> {
        let mut map = HashMap::new();
        for element in &self.data {
            map.insert(element.id, element);
        }
        map
    }
}

pub fn load_directory_file(path: &Path) -> Result<NgcArchive, Box<error::Error>> {
    let file = File::open(path)?;
    let file = BufReader::new(file);
    
    let mut elements = Vec::new();

    for line in file.lines() {
        let line = line?;
        if line.len() == 0 || line.starts_with('\0') {
            break;
        }
        // println!("{}", line);
        let pos = line.find(char::is_whitespace).unwrap();
        let id_str = &line[0..pos];
        let file_str = &line[pos+1..];
        let filelen = file_str.len();
        elements.push(NgcElement {
            filename: file_str[1..filelen-1].to_string(),
            id: id_str.parse()?,
        });
        // println!("    -> {}, {}", elements.last().unwrap().filename, elements.last().unwrap().id);
    }

    Ok(NgcArchive{
        data: elements,
    })
}
