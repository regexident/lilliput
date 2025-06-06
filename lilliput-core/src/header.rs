//! Value headers.

mod bool;
mod bytes;
mod float;
mod int;
mod map;
mod null;
mod seq;
mod string;
mod unit;

#[cfg(any(test, feature = "testing"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;

use crate::marker::Marker;

pub use self::{
    bool::BoolHeader,
    bytes::BytesHeader,
    float::FloatHeader,
    int::{CompactIntHeader, ExtendedIntHeader, IntHeader},
    map::{CompactMapHeader, ExtendedMapHeader, MapHeader},
    null::NullHeader,
    seq::{CompactSeqHeader, ExtendedSeqHeader, SeqHeader},
    string::{CompactStringHeader, ExtendedStringHeader, StringHeader},
    unit::UnitHeader,
};

#[cfg(any(test, feature = "testing"))]
pub(crate) fn arbitrary_len() -> impl Strategy<Value = usize> {
    proptest::prop_oneof![
        proptest::num::u8::ANY.prop_map(|len| len as usize),
        proptest::num::u16::ANY.prop_map(|len| len as usize),
        proptest::num::u32::ANY.prop_map(|len| len as usize),
        proptest::num::u64::ANY.prop_map(|len| len as usize),
    ]
}

/// A value's header.
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Header {
    /// Represents a integer number.
    Int(IntHeader),

    /// Represents a string.
    String(StringHeader),

    /// Represents a sequence of values.
    Seq(SeqHeader),

    /// Represents a map of key-value pairs.
    ///
    /// By default the map is backed by a `BTreeMap`. Enable the `preserve_order`
    /// feature of serde_lilliput to use `OrderMap` instead, which preserves
    /// entries in the order they are inserted into the map.
    Map(MapHeader),

    /// Represents a floating-point number.
    Float(FloatHeader),

    /// Represents a byte array.
    Bytes(BytesHeader),

    /// Represents a boolean.
    Bool(BoolHeader),

    /// Represents a unit value.
    Unit(UnitHeader),

    /// Represents a null value.
    Null(NullHeader),
}

impl Default for Header {
    #[inline]
    fn default() -> Self {
        Self::Null(NullHeader)
    }
}

impl From<IntHeader> for Header {
    #[inline]
    fn from(value: IntHeader) -> Self {
        Self::Int(value)
    }
}

impl From<StringHeader> for Header {
    #[inline]
    fn from(value: StringHeader) -> Self {
        Self::String(value)
    }
}

impl From<SeqHeader> for Header {
    #[inline]
    fn from(value: SeqHeader) -> Self {
        Self::Seq(value)
    }
}

impl From<MapHeader> for Header {
    #[inline]
    fn from(value: MapHeader) -> Self {
        Self::Map(value)
    }
}

impl From<FloatHeader> for Header {
    #[inline]
    fn from(value: FloatHeader) -> Self {
        Self::Float(value)
    }
}

impl From<BytesHeader> for Header {
    #[inline]
    fn from(value: BytesHeader) -> Self {
        Self::Bytes(value)
    }
}

impl From<BoolHeader> for Header {
    #[inline]
    fn from(value: BoolHeader) -> Self {
        Self::Bool(value)
    }
}

impl From<UnitHeader> for Header {
    #[inline]
    fn from(value: UnitHeader) -> Self {
        Self::Unit(value)
    }
}

impl From<NullHeader> for Header {
    #[inline]
    fn from(value: NullHeader) -> Self {
        Self::Null(value)
    }
}

impl Header {
    /// Returns the header's type marker.
    pub fn marker(&self) -> Marker {
        match self {
            Header::Int(_) => Marker::Int,
            Header::String(_) => Marker::String,
            Header::Seq(_) => Marker::Seq,
            Header::Map(_) => Marker::Map,
            Header::Float(_) => Marker::Float,
            Header::Bytes(_) => Marker::Bytes,
            Header::Bool(_) => Marker::Bool,
            Header::Unit(_) => Marker::Unit,
            Header::Null(_) => Marker::Null,
        }
    }
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
        fn encode_decode_roundtrip(header in Header::arbitrary(), config in EncoderConfig::arbitrary()) {
            let mut encoded: Vec<u8> = Vec::new();
            let writer = VecWriter::new(&mut encoded);
            let mut encoder = Encoder::new(writer, config);
            encoder.encode_header(&header).unwrap();

            let reader = SliceReader::new(&encoded);
            let mut decoder = Decoder::from_reader(reader);
            let decoded = decoder.decode_header().unwrap();
            prop_assert_eq!(&decoded, &header);
        }
    }
}
