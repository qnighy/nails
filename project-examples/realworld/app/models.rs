use crate::schema::*;

#[derive(Queryable)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub token: &'a str,
    pub username: &'a str,
    pub bio: Option<&'a str>,
    pub image: Option<&'a str>,
}

#[derive(Queryable)]
pub struct Tag {
    pub id: i64,
    pub tag: String,
}
