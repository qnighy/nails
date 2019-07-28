use std::env;
use std::fmt;

use contextful::Context;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

#[derive(Clone)]
pub struct AppCtx {
    // TODO: async
    db: Pool<ConnectionManager<PgConnection>>,
}

impl fmt::Debug for AppCtx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AppCtx").field("db", &()).finish()
    }
}

impl Context for AppCtx {}

impl AppCtx {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        // Sanity check
        PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));
        let db = ConnectionManager::new(database_url);
        let db = Pool::builder().build(db).unwrap();
        Self { db }
    }
}
