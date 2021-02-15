#![allow(unused)]

use std::fmt;
use std::error::Error;
use diesel::NotFound;
use crate::schema::{kinds, statuses};
use crate::diesel::prelude::*;
use crate::postgres::*;

custom_derive! {
    #[derive(EnumFromStr)]
    #[derive(Eq, PartialEq, Copy, Clone)]
    #[derive(Debug)]
    pub enum Kind {
        USER,
        APP
    }
}

impl Kind {
    pub fn derive(name: &str) -> Result<Kind, Box<dyn Error>> {
        let upper = name.to_uppercase();
        let kind: Kind = upper.parse()?;
        Ok(kind)
    }

    pub fn to_int32(&self) -> i32 {
        *self as i32 + 1
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="kinds"]
struct DBKind {
    pub id: i32,
    pub name: String,
}

pub fn find_kind_by_id(target: i32) -> Result<Kind, Box<dyn Error>>  {
    use crate::schema::kinds::dsl::*;

    let results = { // block is required because of connection release
        let connection = open_stream().get()?;
        kinds.filter(id.eq(target))
            .load::<DBKind>(&connection)?
    };

    if results.len() > 0 {
        let kind = Kind::derive(&results[0].name)?;
        Ok(kind)
    } else {
        Err(Box::new(NotFound))
    }
}

custom_derive! {
    #[derive(EnumFromStr)]
    #[derive(Eq, PartialEq, Copy, Clone)]
    #[derive(Debug)]
    pub enum Status {
        PENDING,
        ACTIVATED,
        HIDDEN,
    }
}

impl Status {
    pub fn derive(name: &str) -> Result<Status, Box<dyn Error>> {
        let upper = name.to_uppercase();
        let status: Status = upper.parse()?;
        Ok(status)
    }

    pub fn to_int32(&self) -> i32 {
        *self as i32 + 1
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="statuses"]
struct DBStatus {
    pub id: i32,
    pub name: String,
}

pub fn find_status_by_id(target: i32) -> Result<Status, Box<dyn Error>>  {
    use crate::schema::kinds::dsl::*;

    let results = { // block is required because of connection release
        let connection = open_stream().get()?;
        kinds.filter(id.eq(target))
            .load::<DBStatus>(&connection)?
    };

    if results.len() > 0 {
        let status = Status::derive(&results[0].name)?;
        Ok(status)
    } else {
        Err(Box::new(NotFound))
    }
}

custom_derive! {
    #[derive(EnumFromStr)]
    #[derive(Eq, PartialEq, Copy, Clone)]
    #[derive(Debug)]
    pub enum Role {
        OWNER,
        GRANTED,
        READER,
    }
}

impl Role {
    pub fn derive(name: &str) -> Result<Role, Box<dyn Error>> {
        let upper = name.to_uppercase();
        let role: Role = upper.parse()?;
        Ok(role)
    }

    pub fn to_int32(&self) -> i32 {
        *self as i32 + 1
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="statuses"]
struct DBRole {
    pub id: i32,
    pub name: String,
}

pub fn find_role_by_id(target: i32) -> Result<Role, Box<dyn Error>>  {
    use crate::schema::kinds::dsl::*;

    let results = { // block is required because of connection release
        let connection = open_stream().get()?;
        kinds.filter(id.eq(target))
        .load::<DBRole>(&connection)?
    };

    if results.len() > 0 {
        let role = Role::derive(&results[0].name)?;
        Ok(role)
    } else {
        Err(Box::new(NotFound))
    }
}