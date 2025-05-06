pub trait MapErrToString<T> {
    fn to_estring(self) -> Result<T, String>;
}

impl<T, E: std::string::ToString> MapErrToString<T> for Result<T, E> {
    fn to_estring(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}
