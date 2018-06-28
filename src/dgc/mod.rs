use std::io::{self, Write, Read};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::cmp;
use std::mem;

/// .DGC header information
/// Format:
/// legal notice [u8; 0x100]
/// chunk size   u32         (implied)
/// junk padding [u8; 0x6FC] (ignored)
/// data         [u8; chunk size * N] (N is any whole number)
pub struct DgcHeader {
    pub legal_notice: [u8; 0x100],
}

/// .DGC file element
/// Format:
/// chunk size u32 (implied)
/// type id    i32
/// id1        i32
/// id2        i32
/// data       [u8] (size is chunk size - 16)
pub struct DgcFile {
    pub data: Vec<u8>,
    pub type_id: i32,
    pub id1: i32,
    pub id2: i32,
}

impl DgcFile {
    /// Get the total size of this file, including its header information.
    pub fn get_size(&self) -> usize {
        self.data.len() + 16
    }

    /// Write this file to the given writer.
    /// Returns the number of bytes that were written.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_u32::<BigEndian>(self.get_size() as u32)?;
        writer.write_i32::<BigEndian>(self.type_id)?;
        writer.write_i32::<BigEndian>(self.id1)?;
        writer.write_i32::<BigEndian>(self.id2)?;
        writer.write_all(&self.data)?;
        Ok(self.get_size())
    }
}

/// .DGC chunk
/// Format:
/// num files u32 (implied)
/// data      [u8; chunk size] (inherited from header)
pub struct DgcChunk {
    pub data: Vec<DgcFile>,
}

impl DgcChunk {
    /// Create a new DgcChunk
    pub fn new() -> DgcChunk {
        DgcChunk {
            data: Vec::new()
        }
    }
    
    /// Add a file to this chunk
    pub fn add_file(&mut self, file: DgcFile) {
        self.data.push(file);
    }

    /// Get the total size of this chunk, including the contents of each file
    /// stored in this chunk, the header data of each file stored in this
    /// chunk, and the header of the chunk itself.
    pub fn get_size(&self) -> usize {
        // Each chunk has a 4 byte header, and each file has a 16 byte header
        self.data.iter().fold(4, |acc, f| acc + f.get_size())
    }

    /// Get the number of files stored within this chunk.
    pub fn get_num_files(&self) -> usize {
        self.data.len()
    }

    /// Write this chunk to the given writer. Also expects a chunk size argument, that describes
    /// exactly how many bytes this chunk should write. If the chunk is too small to fill this
    /// size, then the chunk will zero-pad the rest.
    /// Returns the number of bytes that were written in total to the writer.
    pub fn write_to<W: Write>(&self, writer: &mut W, chunk_size: usize) -> io::Result<usize> {
        let num_files = self.get_num_files() as u32;
        writer.write_u32::<BigEndian>(num_files)?;
        for file in &self.data {
            file.write_to(writer)?;
        }
        let required_padding = chunk_size - self.get_size();
        io::copy(&mut io::repeat(0u8).take(required_padding as u64), writer)?;
        Ok(self.get_size() + required_padding)
    }
}

/// .DGC archive
/// Contains the header information about the archive, as well as all of the files stored in the
/// archive sorted into individual chunks. This structure can also serve as an abstraction layer
/// that can automatically divide up files into chunks without having to to worry about the 
/// details.
pub struct DgcArchive {
    pub header: DgcHeader,
    pub data: Vec<DgcChunk>,
    pub chunk_size: usize,
}

impl DgcArchive {
    /// Create a new DgcArchive. Expects a header and a base chunk size as arguments. The header
    /// must be smaller than 256 bytes. The base chunk size will be automatically rounded up by
    /// 0x800 bytes, and is may change if files are added to this archive.
    pub fn new(header: &str, chunk_size: usize) -> DgcArchive {
        let mut headerdata = [0; 0x100];
        headerdata.copy_from_slice(&header.as_bytes());
        DgcArchive {
            header: DgcHeader {
                legal_notice: headerdata,
            },
            data: vec![],
            chunk_size: calculate_chunk_size(chunk_size),
        }
    }

    /// Iterate over all files in this archive.
    pub fn iter_files(&self) -> impl Iterator<Item=&DgcFile> {
        self.data.iter().flat_map(|chunk| chunk.data.iter())
    }
    
    /// Set a new chunk size. Should be called if a new file is added that is larger than the chunk
    /// size.
    fn reevaluate_files(&mut self, new_size: usize) {
        let mut old_chunks = Vec::new();
        mem::swap(&mut self.data, &mut old_chunks);
        let mut files: Vec<DgcFile> = old_chunks.into_iter().flat_map(|chunk| chunk.data.into_iter()).collect();
        files.sort_unstable_by(|a, b| b.data.len().cmp(&a.data.len()));
        self.data.push(DgcChunk::new());
        self.chunk_size = calculate_chunk_size(new_size);
        while files.len() > 0 {
            let mut chunk = DgcChunk::new();
            let mut i = 0;
            while i < files.len() {
                if files[i].get_size() + chunk.get_size() <= self.chunk_size {
                    chunk.add_file(files.remove(i));
                } else {
                    i += 1;
                }
            }
            self.data.push(chunk);
        }
    }

    /// Add a file to this archive. Will be automatically put into a chunk. This function may
    /// re-distribute files to chunks if the given file is too big to fit into any chunk.
    pub fn add_file(&mut self, file: DgcFile) {
        if file.data.len() > self.chunk_size {
            self.reevaluate_files(file.data.len());
        }
        for chunk in &mut self.data {
            if chunk.get_size() + file.get_size() <= self.chunk_size {
                chunk.add_file(file);
                return;
            }
        }
        let mut new_chunk = DgcChunk::new();
        new_chunk.add_file(file);
        self.data.push(new_chunk);
    }

    /// Write this archive to a writer.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.header.legal_notice)?;
        writer.write_u32::<BigEndian>(self.chunk_size as u32)?;
        io::copy(&mut io::repeat(0u8).take(0x6FC), writer)?;
        for chunk in &self.data {
            chunk.write_to(writer, self.chunk_size)?;
        }
        Ok(())
    }

    /// Create an archive from a reader.
    pub fn read_from<R: Read>(file: &mut R) -> io::Result<DgcArchive> {
        let mut legal_notice: [u8; 0x100] = [0; 0x100];
        file.read_exact(&mut legal_notice)?;
        let size = file.read_u32::<BigEndian>()?;
        io::copy(&mut file.take(0x6FC), &mut io::sink())?;
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
            chunk_size: size as usize,
        })
    }
}

/// Calculate the size that a chunk would have to be in order to store a file of the given size.
fn calculate_chunk_size(max_size: usize) -> usize {
    // Each chunk's size is a multiple of 0x800 bytes
    const CHUNK_MULT: usize = 0x800;
    if max_size == 0 {
        // avoid subtract with overflow error
        return CHUNK_MULT;
    }
    cmp::max(1, 1 + ((max_size - 1) / CHUNK_MULT)) * CHUNK_MULT
}

/// Load a chunk from the given chunk data.
fn load_chunk(mut data: &[u8]) -> io::Result<DgcChunk> {
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

