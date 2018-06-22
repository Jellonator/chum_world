extern crate clap;
extern crate byteorder;
extern crate crc;

use std::path::PathBuf;
use std::error;

pub mod dgc;
pub mod ngc;

/* Complete Chum archive
 * Contains both a .NGC archive and a .DGC archive
 * /
struct ChumArchive {
    dgca: dgc::DgcArchive,
    ngca: NgcArchive,
}*/

fn main() -> Result<(), Box<error::Error>> {
    let app = clap::App::new("Chum World")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .version("1.0")
        .author("James \"Jellonator\" B. <jellonator00@gmail.com>")
        .about("Edits Revenge of the Flying Dutchman archive files")
        .subcommand(clap::SubCommand::with_name("list")
            .about("Lists the contents of the given archive")
            .arg(clap::Arg::with_name("FILE")
                 .help("The archive file to open")
                 .required(true)
                 .index(1)));
    let matches = app.get_matches();
    if let Some(cmdlist) = matches.subcommand_matches("list") {
        let dgc_path = PathBuf::from(cmdlist.value_of_os("FILE").unwrap());
        let ngc_path = dgc_path.with_extension("NGC");
        println!("{:?}, {:?}", dgc_path, ngc_path);

        let dgca = dgc::load_archive_file(&dgc_path)?;
        let ngca = ngc::load_directory_file(&ngc_path)?;

        let id_lookup = ngca.gen_id_lookup();
        for chunk in dgca.data {
            for file in chunk.data {
                println!("Type: {}", id_lookup[&file.type_id].filename);
                println!("ID1: {:12}: {}", &file.id1, id_lookup[&file.id1].filename);
                println!("ID2: {:12}: {}", &file.id2, id_lookup[&file.id2].filename);
                println!();
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
    }
    Ok(())
}
