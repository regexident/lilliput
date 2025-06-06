#[cfg(any(test, feature = "testing"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;

/// Header representing a boolean.
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct BoolHeader {
    value: bool,
}

impl BoolHeader {
    /// Creates a header from its `value`.
    #[inline]
    pub fn new(value: bool) -> Self {
        Self { value }
    }

    /// Returns the associated value.
    #[inline]
    pub fn value(&self) -> bool {
        self.value
    }
}

impl BoolHeader {
    pub(crate) const MASK: u8 = 0b0000011;
    pub(crate) const TYPE_BITS: u8 = 0b0000010;

    pub(crate) const VALUE_BIT: u8 = 0b0000001;
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_log::test;

    use crate::{
        config::EncoderConfig,
        decoder::Decoder,
        encoder::Encoder,
        io::{SliceReader, VecWriter},
    };

    use super::*;

    proptest! {
        #[test]
        fn encode_decode_roundtrip(header in BoolHeader::arbitrary(), config in EncoderConfig::arbitrary()) {
            let mut encoded: Vec<u8> = Vec::new();
            let writer = VecWriter::new(&mut encoded);
            let mut encoder = Encoder::new(writer, config);
            encoder.encode_bool_header(&header).unwrap();

            prop_assert!(encoded.len() == 1);

            let reader = SliceReader::new(&encoded);
            let mut decoder = Decoder::from_reader(reader);
            let decoded = decoder.decode_bool_header().unwrap();
            prop_assert_eq!(&decoded, &header);
        }
    }
}
