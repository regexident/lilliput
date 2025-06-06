use crate::{
    error::Result,
    header::SeqHeader,
    io::Read,
    marker::Marker,
    value::{Seq, SeqValue},
};

use super::Decoder;

impl<'de, R> Decoder<R>
where
    R: Read<'de>,
{
    // MARK: - Value

    /// Decodes a sequence value.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn decode_seq(&mut self) -> Result<Seq> {
        let header = self.decode_seq_header()?;

        self.decode_seq_of(header)
    }

    /// Decodes a sequence value, as a `SeqValue`.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn decode_seq_value(&mut self) -> Result<SeqValue> {
        let header = self.decode_seq_header()?;

        self.decode_seq_value_of(header)
    }

    // MARK: - Header

    /// Decodes a sequence value's header.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn decode_seq_header(&mut self) -> Result<SeqHeader> {
        let byte = self.pull_byte_expecting(Marker::Seq)?;

        let is_compact = (byte & SeqHeader::COMPACT_VARIANT_BIT) != 0b0;

        if is_compact {
            let len = byte & SeqHeader::COMPACT_LEN_BITS;

            #[cfg(feature = "tracing")]
            tracing::debug!(
                byte = crate::binary::fmt_byte(byte),
                is_compact = true,
                len = len
            );

            Ok(SeqHeader::compact(len))
        } else {
            let len_width = 1 + (byte & SeqHeader::EXTENDED_LEN_WIDTH_BITS);
            let len = self.pull_len_bytes(len_width)?;

            #[cfg(feature = "tracing")]
            tracing::debug!(
                byte = crate::binary::fmt_byte(byte),
                is_compact = false,
                len = len
            );

            Ok(SeqHeader::extended(len))
        }
    }

    // MARK: - Skip

    /// Skips the sequence value for a given `header`.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn skip_seq_value_of(&mut self, header: SeqHeader) -> Result<()> {
        let len: usize = match header {
            SeqHeader::Compact(header) => header.len().into(),
            SeqHeader::Extended(header) => header.len(),
        };

        for _ in 0..len {
            self.skip_value()?; // item
        }

        Ok(())
    }

    // MARK: - Body

    /// Decodes sequence value for a given `header`, as a `SeqValue`.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    pub fn decode_seq_value_of(&mut self, header: SeqHeader) -> Result<SeqValue> {
        self.decode_seq_of(header).map(From::from)
    }

    // MARK: - Private

    /// Decodes sequence value for a given `header`.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
    fn decode_seq_of(&mut self, header: SeqHeader) -> Result<Seq> {
        let mut seq = Seq::default();

        for _ in 0..header.len() {
            let value = self.decode_value()?;
            seq.push(value);
        }

        Ok(seq)
    }
}
