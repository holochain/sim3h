use crate::dht::bbdht::dynamodb::api::table::describe::describe_table;
use crate::dht::bbdht::dynamodb::client::Client;
use rusoto_core::RusotoError;
use rusoto_dynamodb::DescribeTableError;
use crate::trace::LogContext;
use crate::trace::tracer;

pub fn table_exists(
    log_context: &LogContext,
    client: &Client,
    table_name: &str,
) -> Result<bool, RusotoError<DescribeTableError>> {
    tracer(&log_context, "table_exist");

    let table_description_result = describe_table(log_context, client, table_name);
    match table_description_result {
        Ok(table_description) => Ok(match table_description.table_status {
            Some(status) => {
                if status == "ACTIVE".to_string() {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }),
        Err(err) => match err {
            RusotoError::Service(DescribeTableError::ResourceNotFound(_)) => Ok(false),
            _ => Err(err),
        },
    }
}

pub fn until_table_exists_or_not(log_context: &LogContext, client: &Client, table_name: &str, exists: bool) {
    loop {
        tracer(&log_context, "until_table_exists_or_not");
        match table_exists(log_context, client, table_name) {
            Ok(does_exist) => {
                if exists == does_exist {
                    break;
                }
            }
            Err(err) => {
                error!("list error while waiting for table to exist: {}", err);
            }
        }
    }
}

pub fn until_table_exists(log_context: &LogContext, client: &Client, table_name: &str) {
    until_table_exists_or_not(log_context, client, table_name, true);
}

pub fn until_table_not_exists(log_context: &LogContext, client: &Client, table_name: &str) {
    until_table_exists_or_not(log_context, client, table_name, false);
}

#[cfg(test)]
pub mod tests {

    use crate::dht::bbdht::dynamodb::api::table::create::create_table;
    use crate::dht::bbdht::dynamodb::api::table::delete::delete_table;
    use crate::dht::bbdht::dynamodb::api::table::exist::table_exists;
    use crate::dht::bbdht::dynamodb::api::table::fixture::table_name_fresh;
    use crate::dht::bbdht::dynamodb::client::local::local_client;
    use crate::dht::bbdht::dynamodb::schema::fixture::attribute_definitions_a;
    use crate::dht::bbdht::dynamodb::schema::fixture::key_schema_a;
    use crate::trace::tracer;

    #[test]
    fn table_exists_test() {
        let log_context = "table_exists_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let key_schema = key_schema_a();
        let attribute_definitions = attribute_definitions_a();

        // not exists
        assert!(!table_exists(&log_context, &local_client, &table_name).expect("could not check if table exists"));

        // create
        assert!(create_table(
            &log_context,
            &local_client,
            &table_name,
            &key_schema,
            &attribute_definitions
        )
        .is_ok());

        // exists
        assert!(table_exists(&log_context, &local_client, &table_name).expect("could not check if table exists"));

        // delete
        assert!(delete_table(&log_context, &local_client, &table_name).is_ok());

        // not exists
        assert!(!table_exists(&log_context, &local_client, &table_name).expect("could not check if table exists"));
    }

}
