use crate::dht::bbdht::dynamodb::api::agent::write::touch_agent;
use crate::dht::bbdht::dynamodb::api::table::create::ensure_cas_table;
use crate::dht::bbdht::dynamodb::client::Client;
use crate::trace::tracer;
use crate::trace::LogContext;
use lib3h_protocol::data_types::SpaceData;
use lib3h_protocol::protocol::ClientToLib3hResponse;
use crate::dht::bbdht::error::BbDhtResult;

/// create space if not exists
/// touch agent
pub fn join_space(
    log_context: &LogContext,
    client: &Client,
    join_space_data: &SpaceData,
) -> BbDhtResult<ClientToLib3hResponse> {
    tracer(&log_context, "join_space");

    let table_name = String::from(join_space_data.space_address.clone());

    ensure_cas_table(&log_context, &client, &table_name)?;
    touch_agent(
        &log_context,
        &client,
        &table_name,
        &join_space_data.agent_id,
    )?;
    Ok(ClientToLib3hResponse::JoinSpaceResult)
}

#[cfg(test)]
pub mod tests {

    use crate::dht::bbdht::dynamodb::client::fixture::bad_client;
    use crate::dht::bbdht::dynamodb::client::local::local_client;
    use crate::space::fixture::space_data_fresh;
    use crate::trace::tracer;
    use crate::workflow::from_client::join_space::join_space;
    use lib3h_protocol::protocol::ClientToLib3hResponse;

    #[test]
    fn join_space_test() {
        let log_context = "join_space_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let space_data = space_data_fresh();

        tracer(&log_context, "check response");

        match join_space(&log_context, &local_client, &space_data) {
            Ok(ClientToLib3hResponse::JoinSpaceResult) => {}
            Ok(result) => {
                panic!("test OK panic: {:?} {:?}", result, &space_data);
            }
            Err(err) => {
                tracer(&log_context, "join_space_test Err panic");
                panic!("{:?} {:?}", err, &space_data);
            }
        }
    }

    #[test]
    fn join_space_bad_client_test() {
        let log_context = "join_space_bad_client_test";

        tracer(&log_context, "fixtures");
        let bad_client = bad_client();
        let space_data = space_data_fresh();

        tracer(&log_context, "check response");
        match join_space(&log_context, &bad_client, &space_data) {
            Err(_) => {
                tracer(&log_context, "👌");
            }
            Ok(v) => {
                panic!("bad Ok {:?}", v);
            }
        }
    }

}
