pub struct ContentEncoding;

impl ContentEncoding {
    pub const DEFLATE: &str = "deflate";
    pub const ZLIB: &str = "zlib";
}

pub struct ContentType;

impl ContentType {
    pub const JSON: &str = "application/json";
}
