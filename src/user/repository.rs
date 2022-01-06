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
use crate::metadata::domain::MetadataRepository;

use super::domain::{User, UserRepository};

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
    pool: &'a Pool<ConnectionManager<PgConnection>>,
    metadata_repo: M,
}

impl<'a, M: MetadataRepository> PostgresUserRepository<'a, M> {
    fn create_on_conn(&self, conn: &PgConnection, user: &mut User) -> Result<(), PgError>  {
         // in order to create a user it must exists the metadata for this user
         // PostgresMetadataRepository::create_on_conn(conn, &mut user.meta)?;

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

    fn delete_on_conn(&self, conn: &PgConnection, user: &User) -> Result<(), PgError>  {
        let _result = diesel::delete(
            users.filter(id.eq(user.id))
        ).execute(conn)?;

        //PostgresMetadataRepository::delete_on_conn(conn, &user.meta)?;

        Ok(())
   }

    fn build_first(&self, results: &[PostgresUser]) -> Result<User, Box<dyn Error>> {
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        let meta = self.metadata_repo.find(results[0].meta_id)?;

        Ok(User{
            id: results[0].id,
            name: results[0].name.clone(),
            email: results[0].email.clone(),
            password: results[0].password.clone(),
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
    
        self.build_first(&results)
    }
    
    fn find_by_email(&self, target: &str) -> Result<User, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = self.pool.get()?;
            users.filter(email.eq(target))
                 .load::<PostgresUser>(&connection)?
        };
    
        self.build_first(&results)
    }

    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()?;
        conn.transaction::<_, PgError, _>(|| self.create_on_conn(&conn, user))?;
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
        let conn = self.pool.get()?;
        conn.transaction::<_, PgError, _>(|| self.delete_on_conn(&conn, user))?;
        Ok(())
    }
}