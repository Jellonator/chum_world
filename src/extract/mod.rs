use dgc;
use ngc;
use plugin;
use serde_json;
use std::cmp;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use util::{self, ChumArchive, CResult};

/// Represents the data stored in the .json file.
/// This is necessary for serializing archive data into a json file, as the
/// information in the DgcArchive and NgcArchive need to be merged, and some
/// data that is stored can be safely removed (e.g. splitting files into
/// chunks, chunk sizes, the actual file's data, etc.).
#[derive(Serialize, Deserialize)]
struct JsonData {
    header: String,
    files: Vec<JsonDataFile>,
}

impl JsonData {
    pub fn exists(&self, name: &str) -> bool {
        for f in &self.files {
            if f.id == name {
                return true;
            }
        }
        false
    }
}

/// Represents a file element in the .json file.
#[derive(Serialize, Deserialize)]
struct JsonDataFile {
    id: String,
    type_id: String,
    subtype_id: String,
    file_name: String,
}

/// Extract the given archive to the given folder
/// If merge is true, then this function will look for an existing meta.json file in the given
/// directory to merge with.
pub fn extract_archive(archive: &ChumArchive, output_folder: &Path, merge: bool) -> CResult<()> {
    let id_lookup = &archive.ngc.names;
    let json_path = output_folder.join("meta.json");
    fs::create_dir_all(&output_folder)?;
    let plugin_manager = plugin::PluginManager::new();

    let mut json_data = JsonData {
        header: String::from_utf8_lossy(&archive.dgc.header.legal_notice).to_string(),
        files: vec![],
    };

    for chunk in &archive.dgc.data {
        for file in &chunk.data {
            let ftype = &id_lookup[&file.type_id];
            let fname = util::get_file_string(&id_lookup[&file.id1], file.id1 as u32);
            let fpath = output_folder.join(fname);
            let mut fh = File::create(&fpath)?;
            let mut data = Vec::new();
            plugin_manager.export(ftype, &mut &file.data[..], &mut data)?;
            fh.write_all(&mut &data[..])?;
            json_data.files.push(JsonDataFile {
                id: id_lookup[&file.id1].to_owned(),
                type_id: id_lookup[&file.type_id].to_owned(),
                subtype_id: id_lookup[&file.id2].to_owned(),
                file_name: fpath.file_name().unwrap().to_str().unwrap().to_owned(),
            });
        }
    }

    if merge {
        let json_file = File::open(&json_path)?;
        let temp_json_data: JsonData = serde_json::from_reader(json_file)?;
        for file in temp_json_data.files {
            if !json_data.exists(&file.id) {
                json_data.files.push(file);
            }
        }
    }

    let mut json_file = File::create(&json_path)?;
    serde_json::to_writer_pretty(&mut json_file, &json_data)?;

    Ok(())
}

/// Import an archive from the given path
pub fn import_archive(input_folder: &Path) -> CResult<ChumArchive> {
    let json_path = input_folder.join("meta.json");
    let json_file = File::open(&json_path)?;

    let json_data: JsonData = serde_json::from_reader(json_file)?;
    let plugin_manager = plugin::PluginManager::new();

    let mut files = Vec::new();
    let mut ngc = ngc::NgcArchive::new();
    for f in &json_data.files {
        let mut fh = File::open(&input_folder.join(&f.file_name))?;
        let mut data = Vec::new();
        plugin_manager.import(&f.type_id, &mut fh, &mut data)?;
        let id_hash        = util::hash_name(&f.id);
        let subtypeid_hash = util::hash_name(&f.subtype_id);
        let typeid_hash    = util::hash_name(&f.type_id);
        ngc.names.insert(id_hash,        f.id.to_owned());
        ngc.names.insert(subtypeid_hash, f.subtype_id.to_owned());
        ngc.names.insert(typeid_hash,    f.type_id.to_owned());
        files.push(dgc::DgcFile {
            data: data,
            id1: id_hash,
            id2: subtypeid_hash,
            type_id: typeid_hash,
        });
    }

    let max_file_size = files.iter().fold(0,
        |acc, f| cmp::max(acc, f.data.len()));

    let mut dgc = dgc::DgcArchive::new(&json_data.header, max_file_size);

    for f in files {
        dgc.add_file(f);
    }

    Ok(ChumArchive {
        dgc: dgc,
        ngc: ngc,
    })

    // let path = Path::new(matches.value_of_os("OUTPUT").unwrap());
    // let ngc_path = path.with_extension("NGC");
    // let dgc_path = path.with_extension("DGC");

    // let mut ngc_file = File::create(ngc_path)?;
    // ngc.write_to(&mut ngc_file)?;
    //
    // let mut dgc_file = File::create(dgc_path)?;
    // dgc.write_to(&mut dgc_file)?;
}
