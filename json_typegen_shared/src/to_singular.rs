pub trait ToSingular {
    fn to_singular(&self) -> String;
}

impl ToSingular for &str {
    // conservative placeholder impl for now to pass the tests
    fn to_singular(&self) -> String {
        if self.ends_with("les") || self.ends_with("pes") {
            return self[0..self.len() - 1].to_string();
        } else if self.ends_with("ss")
            || self.ends_with("es")
            || self.ends_with("is")
            || self.ends_with("as")
            || self.ends_with("us")
            || self.ends_with("os")
            || self.ends_with("news")
        {
            return self.to_string();
        } else if self.ends_with('s') {
            return self[0..self.len() - 1].to_string();
        }
        self.to_string()
    }
}
