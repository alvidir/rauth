pub trait Session {
    fn id(&self) -> &str;
    fn user(&self) -> &str;
    fn deadline(&self) -> u64;
}