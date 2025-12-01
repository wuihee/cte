use crate::database::init;

mod database;

fn main() {
    let _connection = init();
}
