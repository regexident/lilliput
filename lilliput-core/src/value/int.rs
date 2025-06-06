use std::{
    hash::{Hash, Hasher},
    num::TryFromIntError,
};

#[cfg(any(test, feature = "testing"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;

mod signed;
mod unsigned;

pub use self::{signed::SignedIntValue, unsigned::UnsignedIntValue};

/// Represents an integer number.
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[derive(Copy, Clone)]
pub enum IntValue {
    /// Signed value.
    Signed(SignedIntValue),
    /// Unsigned value.
    Unsigned(UnsignedIntValue),
}

impl IntValue {
    /// Returns `true`, if `self` is signed, otherwise `false`.
    pub fn is_signed(&self) -> bool {
        match self {
            Self::Signed(_) => true,
            Self::Unsigned(_) => false,
        }
    }
}

impl Default for IntValue {
    fn default() -> Self {
        Self::Unsigned(Default::default())
    }
}

macro_rules! impl_int_value_from {
    ($t:ty => $v:ident) => {
        impl From<$t> for IntValue {
            fn from(value: $t) -> Self {
                Self::$v(value.into())
            }
        }
    };
}

impl_int_value_from!(i8 => Signed);
impl_int_value_from!(i16 => Signed);
impl_int_value_from!(i32 => Signed);
impl_int_value_from!(i64 => Signed);

impl_int_value_from!(u8 => Unsigned);
impl_int_value_from!(u16 => Unsigned);
impl_int_value_from!(u32 => Unsigned);
impl_int_value_from!(u64 => Unsigned);

macro_rules! impl_int_value_from_size {
    ($t:ty) => {
        impl From<$t> for IntValue {
            fn from(value: $t) -> Self {
                match <$t>::BITS {
                    8 => Self::from(value as u8),
                    16 => Self::from(value as u16),
                    32 => Self::from(value as u32),
                    64 => Self::from(value as u64),
                    _ => unimplemented!(),
                }
            }
        }
    };
}

impl_int_value_from_size!(isize);
impl_int_value_from_size!(usize);

impl PartialEq for IntValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Signed(lhs), Self::Signed(rhs)) => lhs == rhs,
            (Self::Signed(lhs), Self::Unsigned(rhs)) => {
                let lhs = lhs.canonicalized();
                let rhs = rhs.canonicalized();

                if lhs.is_negative() {
                    false
                } else {
                    (lhs as u64) == rhs
                }
            }
            (Self::Unsigned(lhs), Self::Signed(rhs)) => {
                let lhs = lhs.canonicalized();
                let rhs = rhs.canonicalized();

                if rhs.is_negative() {
                    false
                } else {
                    lhs == (rhs as u64)
                }
            }
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => lhs == rhs,
        }
    }
}

impl PartialOrd for IntValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for IntValue {}

impl Ord for IntValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => lhs.cmp(rhs),
            (Self::Signed(lhs), Self::Signed(rhs)) => lhs.cmp(rhs),
            (Self::Unsigned(lhs), Self::Signed(rhs)) => {
                let lhs = lhs.canonicalized();
                let rhs = rhs.canonicalized();
                if rhs.is_negative() {
                    std::cmp::Ordering::Greater
                } else {
                    lhs.cmp(&(rhs as u64))
                }
            }
            (Self::Signed(lhs), Self::Unsigned(rhs)) => {
                let lhs = lhs.canonicalized();
                let rhs = rhs.canonicalized();
                if lhs.is_negative() {
                    std::cmp::Ordering::Less
                } else {
                    (lhs as u64).cmp(&rhs)
                }
            }
        }
    }
}

impl Hash for IntValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Self::Unsigned(value) => {
                let value = value.canonicalized();
                value.to_ne_bytes().hash(state)
            }
            Self::Signed(value) => {
                let value = value.canonicalized();
                if value.is_negative() {
                    value.to_ne_bytes().hash(state)
                } else {
                    (value as u64).to_ne_bytes().hash(state)
                }
            }
        }
    }
}

impl std::fmt::Debug for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signed(value) => std::fmt::Debug::fmt(&value, f),
            Self::Unsigned(value) => std::fmt::Debug::fmt(&value, f),
        }
    }
}

impl std::fmt::Display for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signed(value) => std::fmt::Display::fmt(value, f),
            Self::Unsigned(value) => std::fmt::Display::fmt(value, f),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for IntValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Signed(value) => value.serialize(serializer),
            Self::Unsigned(value) => value.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for IntValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl serde::de::Visitor<'_> for ValueVisitor {
            type Value = IntValue;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("integer value")
            }

            #[inline]
            fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E> {
                Ok(value.into())
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(value.into())
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl IntValue {
    /// Attempts to convert the value into a signed value.
    pub fn to_signed(self) -> Result<SignedIntValue, TryFromIntError> {
        match self {
            IntValue::Signed(signed) => Ok(signed),
            IntValue::Unsigned(unsigned) => unsigned.to_signed(),
        }
    }

    /// Attempts to convert the value into an unsigned value.
    pub fn to_unsigned(self) -> Result<UnsignedIntValue, TryFromIntError> {
        match self {
            IntValue::Signed(signed) => signed.to_unsigned(),
            IntValue::Unsigned(unsigned) => Ok(unsigned),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, hash::RandomState};

    use proptest::prelude::*;
    use test_log::test;

    use crate::{
        config::EncoderConfig,
        decoder::Decoder,
        encoder::Encoder,
        io::{SliceReader, VecWriter},
        value::Value,
    };

    use super::*;

    proptest! {
        #[test]
        fn eq(signed in i8::MIN..=i8::MAX, unsigned in u8::MIN..=u8::MAX) {
            let signed_values = [
                IntValue::from(signed),
                IntValue::from(signed as i16),
                IntValue::from(signed as i32),
                IntValue::from(signed as i64),
            ];

            let unsigned_values = [
                IntValue::from(unsigned),
                IntValue::from(unsigned as u16),
                IntValue::from(unsigned as u32),
                IntValue::from(unsigned as u64),
            ];

            // signed vs signed
            for lhs_value in &signed_values {
                for rhs_value in &signed_values {
                    prop_assert_eq!(lhs_value, rhs_value);
                    prop_assert_eq!(rhs_value, lhs_value);
                }
            }

            // unsigned vs unsigned
            for lhs_value in &unsigned_values {
                for rhs_value in &unsigned_values {
                    prop_assert_eq!(lhs_value, rhs_value);
                    prop_assert_eq!(rhs_value, lhs_value);
                }
            }

            // signed vs unsigned
            for lhs_value in &signed_values {
                for rhs_value in &unsigned_values {
                    if u8::try_from(signed) == Ok(unsigned) {
                        prop_assert_eq!(lhs_value, rhs_value);
                        prop_assert_eq!(rhs_value, lhs_value);
                    } else {
                        prop_assert_ne!(lhs_value, rhs_value);
                        prop_assert_ne!(rhs_value, lhs_value);
                    }
                }
            }
        }

        #[test]
        fn ord(signed in i8::MIN..=i8::MAX, unsigned in u8::MIN..=u8::MAX) {
            let signed_values = [
                IntValue::from(signed),
                IntValue::from(signed as i16),
                IntValue::from(signed as i32),
                IntValue::from(signed as i64),
            ];

            let unsigned_values = [
                IntValue::from(unsigned),
                IntValue::from(unsigned as u16),
                IntValue::from(unsigned as u32),
                IntValue::from(unsigned as u64),
            ];

            // signed vs signed
            for lhs_value in &signed_values {
                for rhs_value in &signed_values {
                    let value_ordering = lhs_value.cmp(rhs_value);
                    prop_assert_eq!(value_ordering, Ordering::Equal);
                }
            }

            // unsigned vs unsigned
            for lhs_value in &unsigned_values {
                for rhs_value in &unsigned_values {
                    let value_ordering = lhs_value.cmp(rhs_value);
                    prop_assert_eq!(value_ordering, Ordering::Equal);
                }
            }

            // signed vs unsigned
            for lhs_value in &signed_values {
                for rhs_value in &unsigned_values {
                    let value_ordering = lhs_value.cmp(rhs_value);

                    if let Ok(positive) = u8::try_from(signed) {
                        let int_ordering = positive.cmp(&unsigned);
                        prop_assert_eq!(value_ordering, int_ordering);
                    } else {
                        prop_assert_eq!(value_ordering, Ordering::Less);
                    }
                }
            }
        }

        #[test]
        fn hash(signed in i8::MIN..=i8::MAX, unsigned in u8::MIN..=u8::MAX) {
            use std::hash::BuildHasher as _;

            let signed_values = [
                IntValue::from(signed),
                IntValue::from(signed as i16),
                IntValue::from(signed as i32),
                IntValue::from(signed as i64),
            ];

            let unsigned_values = [
                IntValue::from(unsigned),
                IntValue::from(unsigned as u16),
                IntValue::from(unsigned as u32),
                IntValue::from(unsigned as u64),
            ];

            // signed vs signed
            for lhs_value in &signed_values {
                for rhs_value in &signed_values {
                    let build_hasher = RandomState::new();
                    let lhs_hash = build_hasher.hash_one(lhs_value);
                    let rhs_hash = build_hasher.hash_one(rhs_value);
                    prop_assert_eq!(lhs_hash, rhs_hash);
                }
            }

            // unsigned vs unsigned
            for lhs_value in &unsigned_values {
                for rhs_value in &unsigned_values {
                    let build_hasher = RandomState::new();
                    let lhs_hash = build_hasher.hash_one(lhs_value);
                    let rhs_hash = build_hasher.hash_one(rhs_value);
                    prop_assert_eq!(lhs_hash, rhs_hash);
                }
            }
        }
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", IntValue::from(42_u8)), "42");
        assert_eq!(format!("{}", IntValue::from(42_u16)), "42");
        assert_eq!(format!("{}", IntValue::from(42_u32)), "42");
        assert_eq!(format!("{}", IntValue::from(42_u64)), "42");

        assert_eq!(format!("{}", IntValue::from(42_i8)), "42");
        assert_eq!(format!("{}", IntValue::from(42_i16)), "42");
        assert_eq!(format!("{}", IntValue::from(42_i32)), "42");
        assert_eq!(format!("{}", IntValue::from(42_i64)), "42");
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", IntValue::from(42_u8)), "42");
        assert_eq!(format!("{:?}", IntValue::from(42_u16)), "42");
        assert_eq!(format!("{:?}", IntValue::from(42_u32)), "42");
        assert_eq!(format!("{:?}", IntValue::from(42_u64)), "42");

        assert_eq!(format!("{:?}", IntValue::from(42_i8)), "42");
        assert_eq!(format!("{:?}", IntValue::from(42_i16)), "42");
        assert_eq!(format!("{:?}", IntValue::from(42_i32)), "42");
        assert_eq!(format!("{:?}", IntValue::from(42_i64)), "42");

        assert_eq!(format!("{:#?}", IntValue::from(42_u8)), "42_u8");
        assert_eq!(format!("{:#?}", IntValue::from(42_u16)), "42_u16");
        assert_eq!(format!("{:#?}", IntValue::from(42_u32)), "42_u32");
        assert_eq!(format!("{:#?}", IntValue::from(42_u64)), "42_u64");

        assert_eq!(format!("{:#?}", IntValue::from(42_i8)), "42_i8");
        assert_eq!(format!("{:#?}", IntValue::from(42_i16)), "42_i16");
        assert_eq!(format!("{:#?}", IntValue::from(42_i32)), "42_i32");
        assert_eq!(format!("{:#?}", IntValue::from(42_i64)), "42_i64");
    }

    proptest! {
        #[test]
        fn encode_decode_roundtrip(value in IntValue::arbitrary(), config in EncoderConfig::arbitrary()) {
            let mut encoded: Vec<u8> = Vec::new();
            let writer = VecWriter::new(&mut encoded);
            let mut encoder = Encoder::new(writer, config);
            encoder.encode_int_value(&value).unwrap();

            prop_assert!(encoded.len() <= 1 + 8);

            let reader = SliceReader::new(&encoded);
            let mut decoder = Decoder::from_reader(reader);
            let decoded = decoder.decode_int_value().unwrap();
            prop_assert_eq!(&decoded, &value);

            let reader = SliceReader::new(&encoded);
            let mut decoder = Decoder::from_reader(reader);
            let decoded = decoder.decode_value().unwrap();
            let Value::Int(decoded) = decoded else {
                panic!("expected int value");
            };
            prop_assert_eq!(&decoded, &value);
        }
    }
}
