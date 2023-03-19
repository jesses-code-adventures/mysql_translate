use dotenv::dotenv;

pub fn set_vars() {
    dotenv().ok();
}
