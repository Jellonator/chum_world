extern crate clap;
extern crate byteorder;
extern crate crc;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate gtk;
extern crate gio;

use std::path::{Path, PathBuf};
use std::error;
use std::fs::{self, File};
use std::io::{Write, Read};
use std::cmp;

pub mod dgc;
pub mod ngc;
pub mod util;
pub mod gui;

/// Complete Chum archive.
/// Contains both a .NGC archive and a .DGC archive.
struct ChumArchive {
    dgc: dgc::DgcArchive,
    ngc: ngc::NgcArchive,
}

/// A Result type that can be any error.
type CResult<T> = Result<T, Box<error::Error>>;

fn load_archive(path: &Path) -> CResult<ChumArchive> {
    let path = PathBuf::from(path);
    let ngc_path = path.with_extension("NGC");
    let dgc_path = path.with_extension("DGC");

    let mut name_file = File::open(ngc_path)?;
    let mut data_file = File::open(dgc_path)?;
    let dgca = dgc::DgcArchive::read_from(&mut data_file)?;
    let ngca = ngc::NgcArchive::read_from(&mut name_file)?;

    Ok(ChumArchive {
        dgc: dgca,
        ngc: ngca
    })
}

/// Info command.
/// Gets information about the given archive.
fn cmd_info(matches: &clap::ArgMatches) -> CResult<()> {
    let archive = load_archive(Path::new(matches.value_of_os("FILE").unwrap()))?;
    
    let chunk_size = archive.dgc.chunk_size;
    let mut max_file_size = 0usize;
    let mut min_file_size = usize::max_value();
    let mut num_files = 0;
    let mut total_size = 0;
    for i in 0..archive.dgc.data.len() {
        let chunk = &archive.dgc.data[i];
        let mut chunk_total_size = 0;
        for f in &chunk.data {
            chunk_total_size += f.data.len();
            total_size += f.data.len();
            num_files += 1;
            max_file_size = cmp::max(max_file_size, f.data.len());
            min_file_size = if min_file_size == 0 {
                f.data.len()
            } else {
                cmp::min(min_file_size, f.data.len())
            }
        }
        let padding_size = chunk_size - chunk_total_size;
        println!("Chunk {:>3}: {:>3} files {:>8}B data {:>8}B padding", i, 
                 chunk.data.len(), chunk_total_size, padding_size);
    }
    println!("Chunk size: {}B ({0:X})", chunk_size);
    let average_size = total_size / num_files;
    println!("Total size: {}B, num files: {}, average file size: {}B", total_size, num_files, average_size);
    println!("Minimum size: {}B, Maximum size: {}B", min_file_size, max_file_size);

    Ok(())
}

/// List command.
/// Lists all of the files in the given archive.
fn cmd_list(matches: &clap::ArgMatches) -> CResult<()> {
    let archive = load_archive(Path::new(matches.value_of_os("FILE").unwrap()))?;
    let id_lookup = &archive.ngc.names;
    for chunk in archive.dgc.data {
        for file in chunk.data {
            // println!("Type: {}", id_lookup[&file.type_id].filename);
            let id: u32 = file.id1 as u32;
            let typestr = if file.id1 == file.id2 {
                format!("{}", id_lookup[&file.type_id])
            } else {
                format!("{1}/{0}", id_lookup[&file.type_id], id_lookup[&file.id2])
            };
            println!("{:8X} {:>35}: {}", id, typestr, &id_lookup[&file.id1]);
        }
    }
    
    /* ID's are all IEEE CRC32 hashes
    use crc::crc32;
    for element in id_lookup {
        let id: u32 = element.0 as u32;
        let filename: &str = &element.1.filename;
        let hieee = crc32::checksum_ieee(&filename.as_bytes());
        let hcast = crc32::checksum_castagnoli(&filename.as_bytes());
        let hkoop = crc32::checksum_koopman(&filename.as_bytes());
        println!("{:8X}, {:8X}({:5}), {:8X}({:5}), {:8X}({:5})", id, hieee, hieee==id, hcast, hcast==id, hkoop, hkoop==id);
    }*/

    Ok(())
}

/// Represents the data stored in the .json file.
/// This is necessary for serializing archive data into a json file, as the
/// information in the DgcArchive and NgcArchive need to be merged, and some
/// data that is stored can be safely removed (e.g. splitting files into 
/// chunks, chunk sizes, the actual file's data, etc.).
#[derive(Serialize, Deserialize)]
struct JsonData {
    header: String,
    folder: String,
    files: Vec<JsonDataFile>,
    names: Vec<JsonDataName>,
}

/// Represents a file element in the .json file.
#[derive(Serialize, Deserialize)]
struct JsonDataFile {
    id: i32,
    type_id: i32,
    parent_id: i32,
    file_name: String,
}

/// Represents a name element in the .json file.
#[derive(Serialize, Deserialize)]
struct JsonDataName {
    id: i32,
    name: String,
}

/// Extract command.
/// Extracts the data from an archive into a folder and a json file.
fn cmd_extract(matches: &clap::ArgMatches) -> CResult<()> {
    let archive = load_archive(Path::new(matches.value_of_os("INPUT").unwrap()))?;
    let output_path = Path::new(matches.value_of_os("OUTPUT").unwrap());
    let output_folder = output_path.with_extension("d");
    let id_lookup = &archive.ngc.names;
    
    fs::create_dir_all(&output_folder)?;
    let mut json_file = File::create(&output_path)?;
    let mut json_data = JsonData {
        folder: output_folder.file_name().unwrap().to_str().unwrap().to_owned(),
        header: String::from_utf8_lossy(&archive.dgc.header.legal_notice).to_string(),
        files: vec![],
        names: vec![],
    };

    for element in &archive.ngc.names {
        json_data.names.push(JsonDataName {
            id: *element.0,
            name: element.1.clone(),
        });
    }

    for chunk in archive.dgc.data {
        for file in chunk.data {
            let fname = util::get_file_string(&id_lookup[&file.id1], file.id1 as u32);
            let fpath = output_folder.join(fname);
            let mut fh = File::create(&fpath)?;
            fh.write_all(&file.data)?;
            json_data.files.push(JsonDataFile {
                id: file.id1,
                type_id: file.type_id,
                parent_id: file.id2,
                file_name: fpath.file_name().unwrap().to_str().unwrap().to_owned(),
            });
        }
    }
    
    serde_json::to_writer_pretty(&mut json_file, &json_data)?;

    Ok(())
}

/// Pack command.
/// Pack the extracted .json and data folder back into archive files.
fn cmd_pack(matches: &clap::ArgMatches) -> CResult<()> {
    // let archive = load_archive(Path::new(matches.value_of_os("OUTPUT").unwrap()))?;
    let json_path = Path::new(matches.value_of_os("INPUT").unwrap());
    // let file_folder = json_path.with_extension("d");

    let json_file = File::open(&json_path)?;
    let json_data: JsonData = serde_json::from_reader(json_file)?;
    let file_folder = json_path.parent().unwrap().join(json_data.folder);
    
    let mut files = Vec::new(); 
    for f in &json_data.files {
        let mut fh = File::open(&file_folder.join(&f.file_name))?;
        let mut data = Vec::new();
        fh.read_to_end(&mut data)?;
        files.push(dgc::DgcFile {
            data: data,
            id1: f.id,
            id2: f.parent_id,
            type_id: f.type_id,
        });
    }

    let max_file_size = files.iter().fold(0,
        |acc, f| cmp::max(acc, f.data.len()));

    let mut dgc = dgc::DgcArchive::new(&json_data.header, max_file_size);
    for f in files {
        dgc.add_file(f);
    }

    let mut ngc = ngc::NgcArchive::new();
    for n in &json_data.names {
        ngc.names.insert(n.id, n.name.clone());
    }

    let path = Path::new(matches.value_of_os("OUTPUT").unwrap());
    let ngc_path = path.with_extension("NGC");
    let dgc_path = path.with_extension("DGC");

    let mut ngc_file = File::create(ngc_path)?;
    ngc.write_to(&mut ngc_file)?;
    let mut dgc_file = File::create(dgc_path)?;
    dgc.write_to(&mut dgc_file)?;

    Ok(())
}

fn main() -> Result<(), Box<error::Error>> {
    // Generate commands
    let app = clap::App::new("Chum World")
        //.setting(clap::AppSettings::ArgRequiredElseHelp)
        .version("1.0")
        .author("James \"Jellonator\" B. <jellonator00@gmail.com>")
        .about("Edits Revenge of the Flying Dutchman archive files")
        .subcommand(clap::SubCommand::with_name("info")
            .about("Get information about the given archive")
            .arg(clap::Arg::with_name("FILE")
                 .help("The archive file to open")
                 .required(true)
                 .index(1)))
        .subcommand(clap::SubCommand::with_name("list")
            .about("Lists the contents of the given archive")
            .arg(clap::Arg::with_name("FILE")
                 .help("The archive file to open")
                 .required(true)
                 .index(1)))
        .subcommand(clap::SubCommand::with_name("extract")
            .about("Extract the contents of an archive to a json file as well as a folder")
            .arg(clap::Arg::with_name("INPUT")
                 .help("The archive file to open")
                 .required(true)
                 .index(1))
            .arg(clap::Arg::with_name("OUTPUT")
                 .help("The json file to output the archive's contents to")
                 .required(true)
                 .index(2)))
        .subcommand(clap::SubCommand::with_name("pack")
            .about("Pack the extracted contents of an archive back into an archive")
            .arg(clap::Arg::with_name("INPUT")
                 .help("The extracted json file")
                 .required(true)
                 .index(1))
            .arg(clap::Arg::with_name("OUTPUT")
                 .help("The output archive file")
                 .required(true)
                 .index(2)));
    // Run given command
    let matches = app.get_matches();
    if let Some(cmdlist) = matches.subcommand_matches("list") {
        cmd_list(cmdlist)?;
    } 
    else if let Some(cmdlist) = matches.subcommand_matches("info") {
        cmd_info(cmdlist)?;
    }
    else if let Some(cmdlist) = matches.subcommand_matches("extract") {
        cmd_extract(cmdlist)?;
    }
    else if let Some(cmdlist) = matches.subcommand_matches("pack") {
        cmd_pack(cmdlist)?;
    } 
    else {
        gui::begin()?;
    }
    Ok(())
}
