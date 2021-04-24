use std::error::Error;
use crate::proto::TicketKind;

use crate::proto::client_proto;
use client_proto::TicketResponse;

pub struct TxTicket<'a> {
    kind: i32,
    ident: &'a str
}

impl<'a> TxTicket<'a> {
    pub fn new(kind: i32, ident: &'a str) -> Self {
        TxTicket{
            kind: kind,
            ident: ident
        }
    }

    pub fn execute(&self) -> Result<TicketResponse, Box<dyn Error>> {
        println!("Got a Ticket request for application {} ", self.ident);

        Ok(TicketResponse{
            id: "".to_string(),
            deadline: 0
        })
    }
}