//! Decoders for decoding lilliput-encoded values.

use crate::{
    error::{Error, Result},
    header::Header,
    io::{Read, Reference},
    marker::Marker,
    value::Value,
};

mod bool;
mod bytes;
mod float;
mod int;
mod map;
mod null;
mod seq;
mod string;
mod unit;

/// A decoder for decoding lilliput-encoded values.
#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    pos: usize,
}

impl<R> Decoder<R> {
    /// Creates a decoder from a `reader`.
    pub fn from_reader(reader: R) -> Self {
        Decoder { reader, pos: 0 }
    }

    /// Returns the decoder's internal `reader`, consuming `self`.
    pub fn into_reader(self) -> R {
        self.reader
    }

    /// Returns the decoder's current read position.
    pub fn pos(&self) -> usize {
        self.pos
    }
}

impl<'de, R> Decoder<R>
where
    R: Read<'de>,
{
    // MARK: - Value

    /// Decodes a `Value`.
    pub fn decode_value(&mut self) -> Result<Value> {
        let header = self.decode_header()?;
        self.decode_value_of(header)
    }

    // MARK: - Marker

    /// Decodes a value's type `Marker`.
    pub fn peek_marker(&mut self) -> Result<Marker> {
        self.peek_byte().map(Marker::detect)
    }

    // MARK: - Header

    /// Decodes a value's `Header`.
    pub fn decode_header(&mut self) -> Result<Header> {
        match self.peek_marker()? {
            Marker::Int => self.decode_int_header().map(From::from),
            Marker::String => self.decode_string_header().map(From::from),
            Marker::Seq => self.decode_seq_header().map(From::from),
            Marker::Map => self.decode_map_header().map(From::from),
            Marker::Float => self.decode_float_header().map(From::from),
            Marker::Bytes => self.decode_bytes_header().map(From::from),
            Marker::Bool => self.decode_bool_header().map(From::from),
            Marker::Unit => self.decode_unit_header().map(From::from),
            Marker::Null => self.decode_null_header().map(From::from),
        }
    }

    // MARK: - Skip

    /// Skips the next to-be-decoded value.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn skip_value(&mut self) -> Result<()> {
        let header = self.decode_header()?;
        self.skip_value_of(header)
    }

    /// Skips the value for a given `header`.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn skip_value_of(&mut self, header: Header) -> Result<()> {
        match header {
            Header::Int(header) => self.skip_int_value_of(header),
            Header::String(header) => self.skip_string_value_of(header),
            Header::Seq(header) => self.skip_seq_value_of(header),
            Header::Map(header) => self.skip_map_value_of(header),
            Header::Float(header) => self.skip_float_value_of(header),
            Header::Bytes(header) => self.skip_bytes_value_of(header),
            Header::Bool(header) => self.skip_bool_value_of(header),
            Header::Unit(header) => self.skip_unit_value_of(header),
            Header::Null(header) => self.skip_null_value_of(header),
        }
    }

    // MARK: - Body

    /// Decodes value for a given `header`.
    pub fn decode_value_of(&mut self, header: Header) -> Result<Value> {
        match header {
            Header::Int(header) => self.decode_int_value_of(header).map(From::from),
            Header::String(header) => self.decode_string_value_of(header).map(From::from),
            Header::Seq(header) => self.decode_seq_value_of(header).map(From::from),
            Header::Map(header) => self.decode_map_value_of(header).map(From::from),
            Header::Float(header) => self.decode_float_value_of(header).map(From::from),
            Header::Bytes(header) => self.decode_bytes_value_of(header).map(From::from),
            Header::Bool(header) => self.decode_bool_value_of(header).map(From::from),
            Header::Unit(header) => self.decode_unit_value_of(header).map(From::from),
            Header::Null(header) => self.decode_null_value_of(header).map(From::from),
        }
    }
}

// MARK: - Auxiliary Methods

impl<'de, R> Decoder<R>
where
    R: Read<'de>,
{
    #[inline]
    fn peek_byte(&mut self) -> Result<u8> {
        self.reader.peek_one()
    }

    #[inline]
    fn pull_byte_expecting(&mut self, marker: Marker) -> Result<u8> {
        let pos = self.pos;

        let byte = self.pull_byte()?;

        marker.validate(byte).map_err(|exp| {
            Error::invalid_type(
                exp.unexpected.to_string(),
                exp.expected.to_string(),
                Some(pos),
            )
        })?;

        Ok(byte)
    }

    #[inline]
    fn pull_byte(&mut self) -> Result<u8> {
        let byte = self.reader.read_one()?;

        self.pos += 1;

        Ok(byte)
    }

    #[inline]
    fn pull_bytes_into<'s>(&'s mut self, buf: &'s mut [u8]) -> Result<()> {
        let len = buf.len();

        if len == 0 {
            return Ok(());
        }

        self.reader.read_into(buf)?;

        self.pos += len;

        Ok(())
    }

    #[inline]
    fn pull_bytes<'s>(
        &'s mut self,
        len: usize,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>> {
        let bytes = self.reader.read(len, scratch)?;

        debug_assert_eq!(bytes.len(), len);

        self.pos += len;

        Ok(bytes)
    }

    #[inline]
    fn pull_len_bytes(&mut self, width: u8) -> Result<usize> {
        let pos = self.pos;

        const MAX_WIDTH: usize = 8;
        let mut padded_be_bytes: [u8; MAX_WIDTH] = [0b0; MAX_WIDTH];
        self.pull_bytes_into(&mut padded_be_bytes[(MAX_WIDTH - (width as usize))..])?;

        u64::from_be_bytes(padded_be_bytes)
            .try_into()
            .map_err(|_| Error::number_out_of_range(Some(pos)))
    }
}

// MARK: - Tests

#[cfg(test)]
mod test {
    use crate::{error::ErrorCode, io::SliceReader};

    use super::*;

    #[test]
    fn new() {
        let bytes = SliceReader::new(&[1, 2, 3]);
        let decoder = Decoder::from_reader(&bytes);
        assert_eq!(decoder.pos, 0);
    }

    #[test]
    fn pull_byte() {
        let bytes = SliceReader::new(&[1, 2, 3]);
        let mut decoder = Decoder::from_reader(bytes);
        assert_eq!(decoder.pos, 0);

        let byte = decoder.pull_byte().unwrap();
        assert_eq!(byte, 1);
        assert_eq!(decoder.pos, 1);

        let byte = decoder.pull_byte().unwrap();
        assert_eq!(byte, 2);
        assert_eq!(decoder.pos, 2);

        let byte = decoder.pull_byte().unwrap();
        assert_eq!(byte, 3);
        assert_eq!(decoder.pos, 3);

        let error_code = decoder.pull_byte().unwrap_err().code();
        assert_eq!(error_code, ErrorCode::UnexpectedEndOfFile);
    }

    #[test]
    fn pull_bytes_into() {
        let bytes = SliceReader::new(&[1, 2, 3]);
        let mut decoder = Decoder::from_reader(bytes);
        assert_eq!(decoder.pos, 0);

        let mut buf = vec![];
        decoder.pull_bytes_into(&mut buf).unwrap();
        assert_eq!(buf, &[]);
        assert_eq!(decoder.pos, 0);

        let mut buf = vec![0];
        decoder.pull_bytes_into(&mut buf).unwrap();
        assert_eq!(buf, &[1]);
        assert_eq!(decoder.pos, 1);

        let mut buf = vec![0, 0];
        decoder.pull_bytes_into(&mut buf).unwrap();
        assert_eq!(buf, &[2, 3]);
        assert_eq!(decoder.pos, 3);

        let mut buf = vec![0, 0, 0];
        let error_code = decoder.pull_bytes_into(&mut buf).unwrap_err().code();
        assert_eq!(error_code, ErrorCode::UnexpectedEndOfFile);
        assert_eq!(decoder.pos, 3);
    }

    #[test]
    fn pull_bytes() {
        let bytes = SliceReader::new(&[1, 2, 3]);
        let mut decoder = Decoder::from_reader(bytes);
        let mut scratch = vec![];
        assert_eq!(decoder.pos, 0);

        let reference = decoder.pull_bytes(0, &mut scratch).unwrap();
        assert_eq!(reference.as_ref(), &[]);
        assert_eq!(decoder.pos, 0);

        scratch.clear();

        let reference = decoder.pull_bytes(1, &mut scratch).unwrap();
        assert_eq!(reference.as_ref(), &[1]);
        assert_eq!(decoder.pos, 1);

        scratch.clear();

        let reference = decoder.pull_bytes(2, &mut scratch).unwrap();
        assert_eq!(reference.as_ref(), &[2, 3]);
        assert_eq!(decoder.pos, 3);

        scratch.clear();

        let error_code = decoder.pull_bytes(1, &mut scratch).unwrap_err().code();
        assert_eq!(error_code, ErrorCode::UnexpectedEndOfFile);
        assert_eq!(decoder.pos, 3);
    }
}
