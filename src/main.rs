#![windows_subsystem = "windows"]
extern crate byteorder;
extern crate clap;
extern crate crc;
extern crate gtk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod dgc;
pub mod extract;
pub mod gui;
pub mod ngc;
pub mod plugin;
pub mod util;

use std::cmp;
use std::error;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use util::{CResult, ChumArchive};

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
    Ok(())
}

/// Extract command.
/// Extracts the data from an archive into a folder and a json file.
fn cmd_extract(matches: &clap::ArgMatches) -> CResult<()> {
    let archive = load_archive(Path::new(matches.value_of_os("INPUT").unwrap()))?;
    let output_path = Path::new(matches.value_of_os("OUTPUT").unwrap());

    fs::create_dir_all(&output_path)?;
    let mut merge = false;
    if output_path.join("meta.json").exists() {
        if matches.is_present("replace") {
            for path in fs::read_dir(&output_path)? {
                let path = path?;
                if path.file_type()?.is_file() {
                    fs::remove_file(&path.path())?;
                }
            }
        }
        else if matches.is_present("merge") {
            merge = true;
        }
        else {
            println!("The given folder already exists. Consider using the following flags:");
            println!("    --merge,-m to merge the contents of the file with the existing folder");
            println!("    --replace,-p to replace the existing folder");
            return Ok(());
        }
    }

    extract::extract_archive(&archive, &output_path, merge)?;

    println!("Extraction successful");

    Ok(())
}

/// Pack command.
/// Pack the extracted .json and data folder back into archive files.
fn cmd_pack(matches: &clap::ArgMatches) -> CResult<()> {
    let input_path = Path::new(matches.value_of_os("INPUT").unwrap());

    let archive = extract::import_archive(&input_path)?;

    let path = Path::new(matches.value_of_os("OUTPUT").unwrap());
    let ngc_path = path.with_extension("NGC");
    let dgc_path = path.with_extension("DGC");

    let mut ngc_file = File::create(ngc_path)?;
    archive.ngc.write_to(&mut ngc_file)?;

    let mut dgc_file = File::create(dgc_path)?;
    archive.dgc.write_to(&mut dgc_file)?;

    println!("Packing successful");

    Ok(())
}

fn main() -> Result<(), Box<error::Error>> {
    // Generate commands
    let app = clap::App::new("Chum World")
        //.setting(clap::AppSettings::ArgRequiredElseHelp)
        .version("0.2.0")
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
            .about("Extract the contents of an archive to a folder")
            .arg(clap::Arg::with_name("INPUT")
                 .help("The archive file to open")
                 .required(true)
                 .index(1))
            .arg(clap::Arg::with_name("OUTPUT")
                 .help("The folder to extract the archive's contents to")
                 .required(true)
                 .index(2))
            .arg(clap::Arg::with_name("merge")
                 .help("Merge with existing")
                 .long("merge")
                 .short("m"))
            .arg(clap::Arg::with_name("replace")
                 .help("Replace existing folder")
                 .long("replace")
                 .short("p")
                 .conflicts_with("merge")))
        .subcommand(clap::SubCommand::with_name("pack")
            .about("Pack the extracted contents of an archive back into an archive")
            .arg(clap::Arg::with_name("INPUT")
                 .help("The folder with extracted file contents")
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
