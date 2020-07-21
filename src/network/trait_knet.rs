use super::*;
pub trait KnetTransform: Copy + Sized {
    fn serialize(&self) -> ::std::vec::Vec<u8>;
    fn deserialize(&mut self, _: &[u8]);
    fn from_raw(_: &[u8]) -> Self;
    fn get_size_of_data(_: &[u8]) -> usize;
    fn get_size_of_payload() -> usize;
}
