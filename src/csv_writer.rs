/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fs::File;
use std::io::{Error, ErrorKind};
use std::marker::PhantomData;
use csv::{Writer, StringRecord};
pub use crate::file_writer::FileWriter;

pub struct CsvWriter<T: Row> {
	file: Writer<File>,
	buf: CsvRow,
	count: u64,

	_marker: PhantomData<T>,
}

pub struct CsvRow {
	data: StringRecord,
}

pub trait Row {
	fn as_row(&self, _: &mut CsvRow) -> Result<(), Error>;
}

impl<T: Row> FileWriter for CsvWriter<T> {
	fn new(path: &str) -> Result<Self, Error> where Self: Sized {
		let Ok(file) = Writer::from_path(path) else {
			return Err(Error::new(
				ErrorKind::NotFound,
				"Could not create CSV file."
			));
		};

		let writer = CsvWriter {
			file,
			buf: CsvRow::new(),
			count: 0,

			_marker: PhantomData,
		};

		Ok(writer)
	}
}

impl<T: Row> CsvWriter<T> {
	pub fn write_row(&mut self, object: &T) {
		self.buf.data.clear();
		self.count += 1;

		if object.as_row(&mut self.buf).is_err() {
			panic!("Error converting object {} to row", self.count);
		}

		if self.file.write_record(&self.buf.data).is_err() {
			panic!("Could not write to CSV file at row {}.", self.count);
		}
	}
}

impl CsvRow {
	fn new() -> Self {
		CsvRow {
			data: StringRecord::new(),
		}
	}

	pub fn push(&mut self, value: &str) {
		self.data.push_field(value);
	}

	pub fn size(&self) -> usize {
		let items_size = self.data
			.iter()
			.map(|item| item.as_bytes().len())
			.sum::<usize>();

		items_size + self.data.len()
	}
}
