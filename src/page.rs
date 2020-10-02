use crate::error::Error;
use std::convert::TryFrom;
use std::mem::size_of;

/// A single page size.
/// Each page represents a node in the BTree.
pub const PAGE_SIZE: usize = 4096;

pub const PTR_SIZE: usize = size_of::<usize>();

/// Value is a wrapper for a value in the page.
pub struct Value(usize);

/// Page is a wrapper for a single page of memory
/// providing some helpful helpers for quick access.
pub struct Page {
    data: Box<[u8; PAGE_SIZE]>,
}

impl Page {
    pub fn new(data: [u8; PAGE_SIZE]) -> Page {
        Page {
            data: Box::new(data),
        }
    }

    /// Fetches a value calculated as BigEndian, sized to usize.
    /// This function may error as the value might not fit into a usize.
    pub fn get_value_from_offset(&self, offset: usize) -> Result<usize, Error> {
        let bytes = &self.data[offset..offset + PTR_SIZE];
        let Value(res) = Value::try_from(bytes)?;
        Ok(res)
    }

    pub fn get_ptr_from_offset(&self, offset: usize, size: usize) -> &[u8] {
        &self.data[offset..offset + size]
    }
}

// Attempts to convert a slice to an array of a fixed size (PTR_SIZE),
// and then return the BigEndian value of the byte array.
impl TryFrom<&[u8]> for Value {
    type Error = Error;

    fn try_from(arr: &[u8]) -> Result<Self, Self::Error> {
        if arr.len() > PTR_SIZE {
            return Err(Error::TryFromSliceError("Unexpected Error: Array recieved is larger than the maximum allowed size of: 4096B."));
        }

        let mut truncated_arr = [0u8; PTR_SIZE];
        for (i, item) in arr.iter().enumerate() {
            truncated_arr[i] = *item;
        }

        Ok(Value(usize::from_be_bytes(truncated_arr)))
    }
}
