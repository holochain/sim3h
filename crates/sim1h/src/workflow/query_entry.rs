use crate::trace::tracer;
use lib3h::error::Lib3hResult;
use crate::dht::bbdht::dynamodb::client::Client;
use crate::trace::LogContext;
use crate::dht::bbdht::error::BbDhtError;
use lib3h_protocol::data_types::QueryEntryData;
use crate::dht::bbdht::dynamodb::api::aspect::read::get_entry_aspects;
use holochain_json_api::json::JsonString;
use lib3h_protocol::data_types::EntryAspectData;
use holochain_core_types::network::query::NetworkQuery;
use std::convert::TryFrom;

pub fn query_entry(
    log_context: &LogContext,
    client: &Client,
    query_entry_data: &QueryEntryData,
) -> Lib3hResult<Vec<EntryAspectData>> {
    tracer(&log_context, "publish_entry");

    let table_name = query_entry_data.space_address.to_string();
    let entry_address = query_entry_data.entry_address.clone();

    let query_raw = query_entry_data.query.as_slice();
    let utf8_result = std::str::from_utf8(&query_raw.clone());
    let query_str = match utf8_result {
        Ok(v) => v,
        Err(err) => {
            Err(BbDhtError::CorruptData(err.to_string()))?
        }
    };
    let query_json = JsonString::from_json(&query_str.to_string());
    let query = match NetworkQuery::try_from(query_json.clone()) {
        Ok(v) => v,
        Err(err) => {
            Err(BbDhtError::CorruptData(err.to_string()))?
        }
    };

    let entry_aspects = get_entry_aspects(
        log_context,
        client,
        &table_name,
        &entry_address,
    )?;

    Ok(match query {
        NetworkQuery::GetEntry => {
            let _keep = vec!["content", "header"];
            let v = entry_aspects.into_iter().filter(|_| true).collect::<Vec<_>>();
            v
        }
        NetworkQuery::GetLinks(_link_type, _link_tag, _maybe_crud_status, _get_links_network_query) => {
            let v = entry_aspects.into_iter().filter(|_| true).collect::<Vec<_>>();
            v
        }
    })
}

#[cfg(test)]
pub mod tests {

    use crate::workflow::query_entry::query_entry;
    use crate::trace::tracer;
    use crate::dht::bbdht::dynamodb::client::local::local_client;
    use crate::workflow::fixture::query_entry_data_fresh;
    use crate::workflow::fixture::space_data_fresh;
    use crate::workflow::join_space::join_space;
    use crate::workflow::fixture::entry_address_fresh;
    use crate::workflow::publish_entry::publish_entry;
    use crate::workflow::fixture::provided_entry_data_fresh;
    use crate::test::unordered_vec_compare;

    #[test]
    pub fn query_entry_test() {
        let log_context = "query_entry_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let space_data = space_data_fresh();
        let entry_address = entry_address_fresh();
        let query_entry_data = query_entry_data_fresh(&space_data, &entry_address);
        let provided_entry_data = provided_entry_data_fresh(&space_data, &entry_address);

        // join space
        assert!(join_space(&log_context, &local_client, &space_data).is_ok());

        // publish entry
        assert!(publish_entry(&log_context, &local_client, &provided_entry_data).is_ok());

        match query_entry(&log_context, &local_client, &query_entry_data) {
            Ok(v) => {
                assert!(unordered_vec_compare(v, provided_entry_data.entry.aspect_list))
            },
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }

}
