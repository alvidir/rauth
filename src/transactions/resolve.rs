use std::error::Error;

pub struct TxResolve<'a> {
    id: &'a str,
    data: &'a str
}

impl<'a> TxResolve<'a> {
    pub fn new(id: &'a str, data: &'a str) -> Self {
        TxResolve{
            id: id,
            data: data
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got a Resolve request for ticket {} ", self.id);

        Ok(())
    }
}