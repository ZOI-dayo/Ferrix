pub trait Decoder<T> {
    fn decode(&self, buf: &[u8]) -> Result<T, Box<dyn std::error::Error>>;
}