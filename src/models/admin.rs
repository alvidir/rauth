use std::error::Error;
use crate::diesel::prelude::*;
use crate::schema::admins;
use crate::models::enums;
use crate::postgres::*;

pub trait Ctrl {
    fn get_id(&self) -> i32;
    fn get_user_id(&self) -> i32;
    fn get_app_id(&self) -> i32;
    fn get_role(&self) -> enums::Role;
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="admins"]
pub struct Admin {
    pub id: i32,
    pub usr_id: i32,
    pub app_id: i32,
    pub role_id: i32,
}

#[derive(Insertable)]
#[table_name="admins"]
struct NewAdmin {
    pub usr_id: i32,
    pub app_id: i32,
    pub role_id: i32,
}

impl Admin {
    pub fn new(usr_id: i32, app_id: i32, role_id: i32) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>> {
        let new_admin = NewAdmin {
            usr_id: usr_id,
            app_id: app_id,
            role_id: role_id,
        };

        let result = { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::insert_into(admins::table)
            .values(&new_admin)
            .get_result::<Admin>(&connection)?
        };

        let wrapper = result.build()?;
        Ok(Box::new(wrapper))
    }

    fn build(&self) -> Result<Wrapper, Box<dyn Error>> {
        let role = enums::find_role_by_id(self.role_id)?;
        Ok(Wrapper{
            admin: self.clone(),
            role: role,
        })
    }
}

struct Wrapper {
    admin: Admin,
    role: enums::Role,
}

impl Ctrl for Wrapper {
    fn get_id(&self) -> i32 {
        self.admin.id
    }

    fn get_user_id(&self) -> i32 {
        self.admin.usr_id
    }

    fn get_app_id(&self) -> i32 {
        self.admin.app_id
    }

    fn get_role(&self) -> enums::Role {
        self.role
    }

}

impl super::Gateway for Wrapper {
    fn delete(&self) -> Result<(), Box<dyn Error>> {
        use crate::schema::admins::dsl::*;

        let connection = open_stream().get()?;
        diesel::delete(
            admins.filter(
                id.eq(self.get_id())
            )
        ).execute(&connection)?;

        Ok(())
    }
}