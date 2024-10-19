use crate::chunk::{ChunkParsingError, ChunkReadingError};
use memmap2::Mmap;
use std::io::Read;
use std::path::PathBuf;
use tracing::error;

pub struct LoadedAnvilFile {
    pub table: [u8; 4096],
    data_map: Mmap,
}

pub fn get_chunk(x: u32, z: u32, file_path: PathBuf) -> Option<Vec<u8>> {
    let loaded_file = load_anvil_file(file_path).ok()?;
    loaded_file.get_chunk(x, z)
}

/// Memory map the file and return a `LoadedAnvilFile` struct
///
/// The `LoadedAnvilFile` struct contains the table and the data map and can be used to get chunk data
///
/// This is pretty fragile when it comes to things like other programs writing to the file while it's open
/// so be careful when using this and make sure to handle errors gracefully
///
/// Arguments:
///
/// * `file_path` - The path to the file
///
/// Returns:
///
/// * `Result<LoadedAnvilFile, ChunkReadingError>` - The loaded anvil file
///
/// # Examples
///
/// ```no_run
/// use std::fs::File;
/// use fastanvil::Region;
/// use std::path::PathBuf;
/// use crate::pumpkin_world::chunk::unsafe_anvil::load_anvil_file;
///
/// let file_path = PathBuf::from("r.0.0.mca");
///
/// let mut fast_file = Region::from_stream(File::open(file_path.clone()).unwrap()).unwrap();
/// let loaded_file = load_anvil_file(file_path).unwrap();
///
/// let chunk = loaded_file.get_chunk(0, 0);
/// let fast_chunk = fast_file.read_chunk(0, 0).unwrap();
///
/// assert_eq!(chunk, fast_chunk);
/// ```
#[allow(unsafe_code)]
pub fn load_anvil_file(file_path: PathBuf) -> Result<LoadedAnvilFile, ChunkReadingError> {
    // Check if the file exists
    if !file_path.exists() {
        return Err(ChunkReadingError::RegionNotFound(file_path));
    }

    match file_path.metadata() {
        Ok(meta) => {
            // We should have at least 8KB of data; 4KB for locations and 4KB for timestamps
            if meta.len() <= (4 * 1024) * 2 {
                return Err(ChunkReadingError::ParsingError(
                    ChunkParsingError::InvalidRegionData(file_path),
                ));
            }
        }
        Err(e) => {
            return Err(ChunkReadingError::IoError(e.kind()));
        }
    }

    let file = std::fs::File::open(&file_path).map_err(|e| ChunkReadingError::IoError(e.kind()))?;

    let res = unsafe { Mmap::map(&file) }.map_err(|e| ChunkReadingError::IoError(e.kind()))?;

    let table = {
        let mut table = [0; 4096];
        table.copy_from_slice(&res[0..4096]);
        table
    };

    Ok(LoadedAnvilFile {
        table,
        data_map: res,
    })
}

impl LoadedAnvilFile {
    /// Get all the locations from the table
    ///
    /// The locations are 32-bit integers, where the first 24 bits are the offset in the file, and
    /// the last 8 bits are the size of the chunk. Generally these aren't useful on their own, but
    /// can be used to get the chunk data with `get_chunk_from_location`. They are probably in order
    /// but not guaranteed to be
    pub fn get_locations(&self) -> Vec<u32> {
        (0..1024)
            .map(|i| {
                u32::from(self.table[i * 4]) << 24
                    | u32::from(self.table[i * 4 + 1]) << 16
                    | u32::from(self.table[i * 4 + 2]) << 8
                    | u32::from(self.table[i * 4 + 3])
            })
            .filter(|&x| x != 0)
            .collect::<Vec<u32>>()
    }

    /// Get the data from the mmaped file, given an offset and size
    #[allow(unsafe_code)]
    fn get_data_from_file(&self, offset: u32, size: u32) -> Vec<u8> {
        unsafe {
            let start = self.data_map.as_ptr().add(offset as usize);
            let slice = std::slice::from_raw_parts(start, size as usize);
            slice.to_vec()
        }
    }

    /// Get the chunk data from a location
    ///
    /// Given a location (usually gotten from `get_locations`), this function will return the
    /// decompressed chunk data associated with that location.
    pub fn get_chunk_from_location(&self, location: u32) -> Option<Vec<u8>> {
        let offset = ((location >> 8) & 0xFFFFFF) * 4096;
        let size = (location & 0xFF) * 4096;
        let chunk_data = self.get_data_from_file(offset, size);
        let chunk_header = chunk_data[0..4].to_vec();
        let chunk_compressed_data = chunk_data[5..].to_vec();
        let uncompressed_size = u32::from(chunk_header[0]) << 24
            | u32::from(chunk_header[1]) << 16
            | u32::from(chunk_header[2]) << 8
            | u32::from(chunk_header[3]);
        let compression_type = chunk_data[4];
        match compression_type {
            1 => {
                let mut decompressed_data = Vec::new();
                let mut decoder = flate2::read::GzDecoder::new(&chunk_compressed_data[..]);
                decoder.read_to_end(&mut decompressed_data).unwrap();
                Some(decompressed_data)
            }
            2 => {
                let out = yazi::decompress(&chunk_compressed_data[..], yazi::Format::Zlib).ok();
                match out {
                    Some(data) => Some(data.0),
                    None => {
                        error!("Failed to decompress Zlib data");
                        None
                    }
                }
            }
            3 => Some(chunk_compressed_data),
            4 => {
                let mut decompressed_data = vec![0; uncompressed_size as usize];
                lz4::Decoder::new(&chunk_compressed_data[..])
                    .and_then(|mut decoder| decoder.read_exact(&mut decompressed_data))
                    .ok()?;
                Some(decompressed_data)
            }
            _ => {
                error!("Unknown compression type: {}", compression_type);
                None
            }
        }
    }

    /// Get the chunk data from the table
    ///
    /// The x and z coordinates are the chunk coordinates
    ///
    /// This function will return the decompressed chunk data
    pub fn get_chunk(&self, x: u32, z: u32) -> Option<Vec<u8>> {
        let index = u64::from(4 * ((x % 32) + (z % 32) * 32));
        let location = u32::from(self.table[index as usize * 4]) << 24
            | u32::from(self.table[index as usize * 4 + 1]) << 16
            | u32::from(self.table[index as usize * 4 + 2]) << 8
            | u32::from(self.table[index as usize * 4 + 3]);
        self.get_chunk_from_location(location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastanvil::Region;
    use rayon::prelude::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_load_anvil_file() {
        let file_path = PathBuf::from("../.etc/regions/r.0.0.mca");
        let result = load_anvil_file(file_path.clone());
        assert!(result.is_ok());
        let loaded_file = result.unwrap();
        let mut file = File::open(file_path).unwrap();
        let mut buf: [u8; 4096] = [0; 4096];
        file.read_exact(&mut buf).unwrap();
        assert_eq!(loaded_file.table, buf);
    }

    #[test]
    fn test_bad_load_fails() {
        let file_path = PathBuf::from("../.etc/regions/shouldnotexist.mca");
        let result = load_anvil_file(file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_chunk() {
        let file_path = PathBuf::from("../.etc/regions/r.0.0.mca");
        let loaded_file = load_anvil_file(file_path.clone()).unwrap();
        let chunk = loaded_file.get_chunk(0, 0);
        let fast_chunk = Region::from_stream(File::open(file_path).unwrap())
            .unwrap()
            .read_chunk(0, 0)
            .unwrap();
        assert!(chunk.is_some());
        assert!(fast_chunk.is_some());
        assert_eq!(chunk.clone().unwrap(), fast_chunk.unwrap());
    }

    #[test]
    fn test_get_chunk_from_location() {
        let file_path = PathBuf::from("../.etc/regions/r.0.0.mca");
        let loaded_file = load_anvil_file(file_path).unwrap();
        let locations = loaded_file.get_locations();
        locations.chunks(96).par_bridge().for_each(|chunk| {
            chunk.iter().for_each(|location| {
                let _ = loaded_file.get_chunk_from_location(*location);
            });
        });
    }
}
