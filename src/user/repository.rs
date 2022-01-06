use std::error::Error;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    result::Error as PgError,
    pg::PgConnection,
    NotFound
};

use crate::diesel::prelude::*;
use crate::schema::users;
use crate::schema::users::dsl::*;

use crate::metadata::{
    application::MetadataRepository
};

use super::{
    application::UserRepository,
    domain::User,
};

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(AsChangeset)]
#[derive(Clone)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "users"]
struct PostgresUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub meta_id: i32,
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "users"]
struct NewPostgresUser<'a> {
    pub name: &'a str, 
    pub email: &'a str,
    pub password: &'a str,
    pub meta_id: i32,
}

pub struct PostgresUserRepository<'a, M: MetadataRepository> {
    pub pool: &'a Pool<ConnectionManager<PgConnection>>,
    pub metadata_repo: M,
}

impl<'a, M: MetadataRepository> PostgresUserRepository<'a, M> {
    fn tx_create(&self, conn: &PgConnection, user: &mut User) -> Result<(), PgError>  {
        let new_user = NewPostgresUser {
            email: &user.email,
            name: &user.name,
            password: &user.password,
            meta_id: user.meta.get_id(),
        };

        let result = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<PostgresUser>(conn)?;

        user.id = result.id;
        Ok(())
    }

    fn tx_delete(&self, conn: &PgConnection, user: &User) -> Result<(), PgError>  {
        let _result = diesel::delete(
            users.filter(id.eq(user.id))
        ).execute(conn)?;

        Ok(())
   }

    fn build(&self, pg_user: &PostgresUser) -> Result<User, Box<dyn Error>> {
        let meta = self.metadata_repo.find(pg_user.meta_id)?;

        Ok(User{
            id: pg_user.id,
            name: pg_user.name.clone(),
            email: pg_user.email.clone(),
            password: pg_user.password.clone(),
            meta: meta,
        })
    }
}

impl<'a, M: MetadataRepository> UserRepository for PostgresUserRepository<'a, M> {
    fn find(&self, target: i32) -> Result<User, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = self.pool.get()?;
            users.filter(id.eq(target))
                 .load::<PostgresUser>(&connection)?
        };

        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }
    
        self.build(&results[0]) // another connection consumed here
    }
    
    fn find_by_email(&self, target: &str) -> Result<User, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = self.pool.get()?;
            users.filter(email.eq(target))
                 .load::<PostgresUser>(&connection)?
        };

        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }
    
        self.build(&results[0]) // another connection consumed here
    }

    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
        self.metadata_repo.create(&mut user.meta)?;

        let conn = self.pool.get()?;
        conn.transaction::<_, PgError, _>(|| self.tx_create(&conn, user))?;
        Ok(())
    }

    fn save(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let pg_user = PostgresUser {
            id: user.id,
            name: user.name.to_string(),
            email: user.email.to_string(),
            password: user.password.clone(),
            meta_id: user.meta.get_id(),
        };
        
        let connection = self.pool.get()?;
        diesel::update(users)
            .filter(id.eq(user.id))
            .set(&pg_user)
            .execute(&connection)?;

        Ok(())
    }

    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>> {
        { // block is required because of connection release
            let conn = self.pool.get()?;
            conn.transaction::<_, PgError, _>(|| self.tx_delete(&conn, user))?;
        }

        self.metadata_repo.delete(&user.meta)?; // another connection consumed here
        Ok(())
    }
}