use super::*;
pub trait KnetTransform: Copy + Sized {
    ///Serialize the data into a vector of byte
    fn serialize(&self) -> ::std::vec::Vec<u8>;
    ///Deserialize the data into a vector of byte
    fn deserialize(&mut self, _: &[u8]);
    ///Get the size of the payload of the serialize data
    fn get_size_of_payload() -> usize;
    /// Get size of the data followed by the payload
    /// Note the `data` have to be the same size as the `get_size_of_payload` function
    fn get_size_of_data(data: &[u8]) -> usize;
    ///Create `Self` by the vector of byte
    fn from_raw(_: &[u8]) -> Self;
}
