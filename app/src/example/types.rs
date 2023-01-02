use pgx::*;
use serde::{Deserialize, Serialize};

///example structs used in postgres database
#[derive(Serialize, Deserialize, PostgresType)]
pub struct ExampleType {
    id: u32,
    name: String,
}

#[pg_extern]
fn get_example_type_name(input: ExampleType) -> String {
    input.name
}
