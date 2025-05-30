use crate::{
    error::Result,
    header::BytesHeader,
    io::{Read, Reference},
    marker::Marker,
    value::BytesValue,
};

use super::Decoder;

impl<'de, R> Decoder<R>
where
    R: Read<'de>,
{
    // MARK: - Value

    /// Decodes a byte array value, as a slice reference.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn decode_bytes<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>> {
        let header = self.decode_bytes_header()?;

        self.decode_bytes_of(header, scratch)
    }

    /// Decodes a byte array value, as an owned buffer.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn decode_bytes_buf(&mut self) -> Result<Vec<u8>> {
        let header = self.decode_bytes_header()?;

        self.decode_bytes_buf_of(header)
    }

    /// Decodes a byte array value, as a `BytesValue`.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn decode_bytes_value(&mut self) -> Result<BytesValue> {
        self.decode_bytes_buf().map(From::from)
    }

    // MARK: - Header

    /// Decodes a byte array value's header.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn decode_bytes_header(&mut self) -> Result<BytesHeader> {
        let byte = self.pull_byte_expecting(Marker::Bytes)?;

        let len_width_exponent = byte & BytesHeader::LEN_WIDTH_EXPONENT_BITS;

        let len_width: u8 = 1 << len_width_exponent;
        let len = self.pull_len_bytes(len_width)?;

        #[cfg(feature = "tracing")]
        tracing::debug!(byte = crate::binary::fmt_byte(byte), len = len);

        Ok(BytesHeader::for_len(len))
    }

    // MARK: - Skip

    /// Skips the byte array value for a given `header`.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn skip_bytes_value_of(&mut self, header: BytesHeader) -> Result<()>
    where
        R: Read<'de>,
    {
        self.reader.skip(header.len())
    }

    // MARK: - Body

    /// Decodes byte array value for a given `header`, as a `BytesValue`.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn decode_bytes_value_of(&mut self, header: BytesHeader) -> Result<BytesValue> {
        self.decode_bytes_buf_of(header).map(From::from)
    }

    // MARK: - Private

    /// Decodes byte array value for a given `header`, using a scratch buffer.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn decode_bytes_of<'s>(
        &'s mut self,
        header: BytesHeader,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>> {
        self.pull_bytes(header.len(), scratch)
    }

    /// Decodes byte array value for a given `header`, returning an owned buffer.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn decode_bytes_buf_of(&mut self, header: BytesHeader) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self.decode_bytes_of(header, &mut buf)? {
            Reference::Borrowed(slice) => {
                debug_assert_eq!(buf.len(), 0);
                buf.extend_from_slice(slice);
            }
            Reference::Copied(slice) => {
                debug_assert_eq!(slice.len(), buf.len());
            }
        }

        Ok(buf)
    }
}
