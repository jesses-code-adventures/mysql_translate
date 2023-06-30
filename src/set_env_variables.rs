use dotenvy::dotenv;

pub fn set_vars() {
    dotenv().expect(".env not found");
}
