use std::path::Path;
use std::fs::File;
use std::io;
use std::io::Read;
use byteorder::{BigEndian, ReadBytesExt};

/* .DGC header information
 * Format:
 * legal notice [u8; 0x100]
 * stream size  u32         (implied)
 * junk padding [u8; 0x6FC] (ignored)
 * data         [u8; stream size]
 */
pub struct DgcHeader {
    pub legal_notice: [u8; 256],
}

/* .DGC file element
 * Format:
 * chunk size u32 (implied)
 * type id    i32
 * id1        i32
 * id2        i32
 * data       [u8] (size is multiple of chunk size)
 */
pub struct DgcFile {
    pub data: Vec<u8>,
    pub type_id: i32,
    pub id1: i32,
    pub id2: i32,
}

/* .DGC chunk
 * Format:
 * num files u32 (implied)
 * data      [u8; steam size] (inherited from header)
 */
pub struct DgcChunk {
    pub data: Vec<DgcFile>,
}

/* .DGC archive
 * Contains header and documents
 */
pub struct DgcArchive {
    pub header: DgcHeader,
    pub data: Vec<DgcChunk>,
}

pub fn load_chunk(mut data: &[u8]) -> io::Result<DgcChunk> {
    let num_files = data.read_u32::<BigEndian>()?;
    let mut files = Vec::new();
    for _ in 0..num_files {
        let file_size = data.read_u32::<BigEndian>()?;
        let id_type = data.read_i32::<BigEndian>()?;
        let id1 = data.read_i32::<BigEndian>()?;
        let id2 = data.read_i32::<BigEndian>()?;
        let mut contents: Vec<u8> = vec![0; file_size as usize - 16];
        data.read_exact(&mut contents)?;
        files.push(DgcFile {
            data: contents,
            type_id: id_type,
            id1: id1,
            id2: id2,
        });
    }
    Ok(DgcChunk {
        data: files,
    })
}

pub fn load_archive_file(path: &Path) -> io::Result<DgcArchive> {
    let mut file = File::open(path)?;
    let mut legal_notice: [u8; 0x100] = [0; 0x100];
    file.read_exact(&mut legal_notice)?;
    let size = file.read_u32::<BigEndian>()?;
    io::copy(&mut file.by_ref().take(0x6FC), &mut io::sink())?;
    let mut fdata = Vec::new();
    let mut chunks = Vec::new();
    file.read_to_end(&mut fdata)?;
    if fdata.len() % (size as usize) > 0 {
        println!("Warning: stream size {} is not divisible by chunk size {}!", fdata.len(), size);
    }
    for chunk in fdata.chunks(size as usize) {
        chunks.push(load_chunk(chunk)?);
    }
    Ok(DgcArchive {
        header: DgcHeader {
            legal_notice: legal_notice,
        },
        data: chunks,
    })
}
