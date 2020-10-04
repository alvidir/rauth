use crate::model::session::traits::*;

pub struct SessionImplementation<'a> {
    pub id: &'a str, // id takes the lifetime of SessionImplemntation
    pub deadline: u64,
}

impl<'a> Session for SessionImplementation<'a> {
    fn id(&self) -> &str {
        self.id
    }

    fn user(&self) -> &str {
        self.id
    }

    fn deadline(&self) -> u64 {
        self.deadline
    }

}