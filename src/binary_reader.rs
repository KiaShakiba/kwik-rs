/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{
	path::Path,
	fs::File,
	io::{BufReader, Read, Error, ErrorKind},
};

pub use crate::file_reader::FileReader;

/// Reads a binary file in chunks
pub struct BinaryReader<T>
where
	T: Chunk,
	[u8; T::SIZE]: Sized,
{
	file: BufReader<File>,
	buf: [u8; T::SIZE],
	count: u64,
}

/// Implementing this trait specifies the number of bytes each
/// chunk occupies in the binary file. The file will be read in chunks
/// of that size.
///
/// # Examples
/// ```
/// use kwik::binary_reader::SizedChunk;
///
/// struct MyStruct {
///     // data fields
/// }
///
/// impl SizedChunk for MyStruct {
///     const SIZE: usize = 10;
/// }
/// ```
pub trait SizedChunk {
	const SIZE: usize;
}

/// Implementing this trait allows the binary reader to parse chunks
/// of the binary file into the specified type.
pub trait Chunk: SizedChunk {
	/// Returns an instance of the implemented struct, given a chunk
	/// of the binary file. If the chunk could not be parsed, an
	/// error result is returned.
	///
	/// # Examples
	/// ```
	/// use std::io::Error;
	/// use kwik::binary_reader::{Chunk, SizedChunk};
	///
	/// struct MyStruct {
	///     // data fields
	/// }
	///
	/// impl Chunk for MyStruct {
	///     fn new(chunk: &[u8; Self::SIZE]) -> Result<Self, Error>
	///     where
	///         Self: Sized,
	///     {
	///         // parse the chunk and return an instance of `Self` on success
	///         Ok(MyStruct {})
	///     }
	/// }
	///
	/// impl SizedChunk for MyStruct {
	///     const SIZE: usize = 0;
	/// }
	/// ```
	fn new(buf: &[u8; Self::SIZE]) -> Result<Self, Error>
	where
		Self: Sized,
	;
}

impl<T> FileReader for BinaryReader<T>
where
	T: Chunk,
	[u8; T::SIZE]: Sized,
{
	/// Opens the file at the supplied path. If the file could not be
	/// opened, returns an error result.
	fn new<P>(path: P) -> Result<Self, Error>
	where
		Self: Sized,
		P: AsRef<Path>,
	{
		let opened_file = File::open(path)?;

		let reader = BinaryReader {
			file: BufReader::new(opened_file),
			buf: [0; T::SIZE],
			count: 0,
		};

		Ok(reader)
	}

	/// Returns the number of bytes in the opened file.
	fn size(&self) -> u64 {
		let metadata = self.file
			.get_ref()
			.metadata()
			.expect("Could not get binary file's size.");

		metadata.len()
	}
}

impl<T> BinaryReader<T>
where
	T: Chunk,
	[u8; T::SIZE]: Sized,
{
	/// Reads one chunk of the binary file, as specified by the chunk size,
	/// and returns an option containing the parsed chunk. If the end of the
	/// file is reached, `None` is returned.
	///
	/// # Examples
	/// ```
	/// use std::io::Error;
	/// use std::env;
	/// use kwik::binary_reader::{FileReader, BinaryReader, Chunk, SizedChunk};
	///
	/// let mut path = env::var("CARGO_MANIFEST_DIR").unwrap();
	/// path.push_str("/target/file.bin");
	///
	/// let mut reader = BinaryReader::<MyStruct>::new(&path).unwrap();
	///
	/// while let Some(object) = reader.read_chunk() {
	///     // do something with the object
	/// }
	///
	/// struct MyStruct {
	///     // data fields
	///     data: u32,
	/// }
	///
	/// impl Chunk for MyStruct {
	///     fn new(chunk: &[u8; Self::SIZE]) -> Result<Self, Error>
	///     where
	///         Self: Sized,
	///     {
	///         // parse the chunk and return an instance of `Self` on success
	///         Ok(MyStruct { data: 0 })
	///     }
	/// }
	///
	/// impl SizedChunk for MyStruct {
	///     const SIZE: usize = 4;
	/// }
	/// ```
	pub fn read_chunk(&mut self) -> Option<T> {
		match self.file.read_exact(&mut self.buf) {
			Ok(_) => {
				self.count += 1;

				let object = match T::new(&self.buf) {
					Ok(object) => object,
					Err(err) => panic!("Parse error in chunk {}: {:?}", self.count, err),
				};

				Some(object)
			},

			Err(ref err) if err.kind() ==  ErrorKind::UnexpectedEof => None,
			Err(_) => panic!("An error occurred when reading binary file."),
		}
	}
}
