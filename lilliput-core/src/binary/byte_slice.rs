use super::byte::Byte;

pub(crate) struct BytesSliceIter<'a>(&'a [u8]);

impl Iterator for BytesSliceIter<'_> {
    type Item = Byte;

    fn next(&mut self) -> Option<Self::Item> {
        let (item, slice) = self.0.split_first()?;

        self.0 = slice;

        Some(Byte(*item))
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct BytesSlice<'a>(pub &'a [u8]);

impl<'a> BytesSlice<'a> {
    pub fn iter(&self) -> BytesSliceIter<'a> {
        BytesSliceIter(self.0)
    }
}

impl<'a> IntoIterator for BytesSlice<'a> {
    type Item = <BytesSliceIter<'a> as Iterator>::Item;

    type IntoIter = BytesSliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl std::fmt::Display for BytesSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, byte) in self.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{byte:0>2x}")?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl std::fmt::Debug for BytesSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, byte) in self.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            if f.alternate() {
                write!(f, "{byte:#08b}")?;
            } else {
                write!(f, "{byte:08b}")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl std::fmt::LowerHex for BytesSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "0x ")?;
        }
        for (index, byte) in self.iter().enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }
            write!(f, "{byte:0>2x}")?;
        }
        Ok(())
    }
}

impl std::fmt::UpperHex for BytesSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "0x ")?;
        }
        for (index, byte) in self.iter().enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }
            write!(f, "{byte:0>2X}")?;
        }
        Ok(())
    }
}

impl std::fmt::Octal for BytesSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "0o ")?;
        }
        for (index, byte) in self.iter().enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }
            write!(f, "{byte:0>3o}")?;
        }
        Ok(())
    }
}

impl std::fmt::Binary for BytesSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "0b ")?;
        }
        for (index, byte) in self.iter().enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }
            write!(f, "{byte:08b}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    #[test]
    fn debug() {
        let bytes = BytesSlice(&[42, 13, 37]);

        assert_eq!(format!("{bytes:?}"), "[00101010, 00001101, 00100101]");
        assert_eq!(
            format!("{bytes:#?}"),
            "[0b00101010, 0b00001101, 0b00100101]"
        );
    }

    #[test]
    fn display() {
        let bytes = BytesSlice(&[42, 13, 37]);

        assert_eq!(format!("{bytes}"), "[2a, 0d, 25]");
    }

    #[test]
    fn lower_hex() {
        let bytes = BytesSlice(&[42, 13, 37]);

        assert_eq!(format!("{bytes:x}"), "2a 0d 25");
        assert_eq!(format!("{bytes:#x}"), "0x 2a 0d 25");
    }

    #[test]
    fn upper_hex() {
        let bytes = BytesSlice(&[42, 13, 37]);

        assert_eq!(format!("{bytes:X}"), "2A 0D 25");
        assert_eq!(format!("{bytes:#X}"), "0x 2A 0D 25");
    }

    #[test]
    fn octal() {
        let bytes = BytesSlice(&[42, 13, 37]);

        assert_eq!(format!("{bytes:o}"), "052 015 045");
        assert_eq!(format!("{bytes:#o}"), "0o 052 015 045");
    }

    #[test]
    fn binary() {
        let bytes = BytesSlice(&[42, 13, 37]);

        assert_eq!(format!("{bytes:b}"), "00101010 00001101 00100101");
        assert_eq!(format!("{bytes:#b}"), "0b 00101010 00001101 00100101");
    }
}
