pub trait LogBuffer: 'static + Send + Sync {
    //async fn push_line<S: Sized + AsRef<str> + ToString + Send>(&self, line: S);
    fn push_line(&self, line: String);
}

impl LogBuffer for Option<()> {
    fn push_line(&self, _line: String) {}
}
