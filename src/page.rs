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

    /// write_value_at_offset writes a given value (as BigEndian) at a certain offset
    /// overriding values at that offset.
    pub fn write_value_at_offset(&mut self, offset: usize, value: usize) -> Result<(), Error> {
        if offset > PAGE_SIZE - PTR_SIZE {
            return Err(Error::UnexpectedError);
        }
        let bytes = value.to_be_bytes();
        self.data[offset..offset + PTR_SIZE].clone_from_slice(&bytes);
        Ok(())
    }

    /// get_value_from_offset Fetches a value calculated as BigEndian, sized to usize.
    /// This function may error as the value might not fit into a usize.
    pub fn get_value_from_offset(&self, offset: usize) -> Result<usize, Error> {
        let bytes = &self.data[offset..offset + PTR_SIZE];
        let Value(res) = Value::try_from(bytes)?;
        Ok(res)
    }

    /// insert_bytes_at_offset pushes #size bytes from offset to end_offset
    /// inserts #size bytes from given slice.
    pub fn insert_bytes_at_offset(
        &mut self,
        bytes: &[u8],
        offset: usize,
        end_offset: usize,
        size: usize,
    ) -> Result<(), Error> {
        // This Should not occur - better verify.
        if end_offset + size > self.data.len() {
            return Err(Error::UnexpectedError);
        }
        for idx in (offset..=end_offset).rev() {
            self.data[idx + size] = self.data[idx]
        }
        self.data[offset..offset + size].clone_from_slice(&bytes);
        Ok(())
    }

    /// write_bytes_at_offset write bytes at a certain offset overriding previous values.
    pub fn write_bytes_at_offset(
        &mut self,
        bytes: &[u8],
        offset: usize,
        size: usize,
    ) -> Result<(), Error> {
        self.data[offset..offset + size].clone_from_slice(&bytes);
        Ok(())
    }

    /// get_ptr_from_offset Fetches a slice of bytes from certain offset and of certain size.
    pub fn get_ptr_from_offset(&self, offset: usize, size: usize) -> &[u8] {
        &self.data[offset..offset + size]
    }

    /// get_data returns the underlying array.
    pub fn get_data(&self) -> [u8; PAGE_SIZE] {
        *self.data
    }
}

/// Attempts to convert a slice to an array of a fixed size (PTR_SIZE),
/// and then return the BigEndian value of the byte array.
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
