use std::env;

use contextful::Context;
use derivative::Derivative;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct AppCtx {
    // TODO: async
    #[derivative(Debug = "ignore")]
    pub db: Pool<ConnectionManager<PgConnection>>,
    #[derivative(Debug = "ignore")]
    pub secret_key: String,
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

        let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY must be set");

        Self { db, secret_key }
    }
}
