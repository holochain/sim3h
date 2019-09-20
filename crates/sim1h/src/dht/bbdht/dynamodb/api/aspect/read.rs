use crate::dht::bbdht::dynamodb::api::item::read::get_item_by_address;
use crate::dht::bbdht::dynamodb::client::Client;
use crate::dht::bbdht::dynamodb::schema::cas::ASPECT_ADDRESS_KEY;
use crate::dht::bbdht::dynamodb::schema::cas::ASPECT_KEY;
use crate::dht::bbdht::dynamodb::schema::cas::ASPECT_LIST_KEY;
use crate::dht::bbdht::dynamodb::schema::cas::ASPECT_PUBLISH_TS_KEY;
use crate::dht::bbdht::dynamodb::schema::cas::ASPECT_TYPE_HINT_KEY;
use crate::dht::bbdht::dynamodb::schema::TableName;
use crate::dht::bbdht::error::BbDhtError;
use crate::dht::bbdht::error::BbDhtResult;
use crate::trace::tracer;
use crate::trace::LogContext;
use holochain_persistence_api::cas::content::Address;
use lib3h_protocol::data_types::EntryAspectData;
use rusoto_dynamodb::AttributeValue;
use std::collections::HashMap;

fn try_aspect_from_item(item: HashMap<String, AttributeValue>) -> BbDhtResult<EntryAspectData> {
    let aspect_address = match item[ASPECT_ADDRESS_KEY].s.clone() {
        Some(address) => Address::from(address),
        None => {
            return Err(BbDhtError::MissingData(format!(
                "Missing aspect_address: {:?}",
                item
            )))
        }
    };

    let aspect = match item[ASPECT_KEY].b.clone() {
        Some(binary_data) => binary_data.to_vec().into(),
        None => {
            return Err(BbDhtError::MissingData(format!(
                "Missing aspect: {:?}",
                item
            )))
        }
    };

    let publish_ts = match item[ASPECT_PUBLISH_TS_KEY].n.clone() {
        Some(publish_ts) => publish_ts.parse()?,
        None => {
            return Err(BbDhtError::MissingData(format!(
                "Missing publish_ts: {:?}",
                item
            )))
        }
    };

    let type_hint = match item[ASPECT_TYPE_HINT_KEY].s.clone() {
        Some(type_hint) => type_hint,
        None => {
            return Err(BbDhtError::MissingData(format!(
                "Missing type_hint: {:?}",
                item
            )))
        }
    };

    Ok(EntryAspectData {
        aspect_address: aspect_address,
        aspect: aspect,
        publish_ts: publish_ts,
        type_hint: type_hint,
    })
}

pub fn try_aspect_list_from_item(
    item: HashMap<String, AttributeValue>,
) -> BbDhtResult<Vec<Address>> {
    let addresses = match item[ASPECT_LIST_KEY].ss.clone() {
        Some(addresses) => addresses.iter().map(|s| Address::from(s.clone())).collect(),
        None => {
            return Err(BbDhtError::MissingData(format!(
                "Missing aspect_list: {:?}",
                item
            )))
        }
    };

    Ok(addresses)
}

pub fn get_aspect(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    aspect_address: &Address,
) -> BbDhtResult<Option<EntryAspectData>> {
    tracer(&log_context, "read_aspect");

    match get_item_by_address(&log_context, &client, &table_name, &aspect_address) {
        Ok(get_output) => match get_output {
            Some(aspect_item) => Ok(Some(try_aspect_from_item(aspect_item)?)),
            None => Ok(None),
        },
        Err(err) => Err(err.into()),
    }
}

pub fn get_entry_aspects(
    log_context: &LogContext,
    client: &Client,
    table_name: &TableName,
    entry_address: &Address,
) -> BbDhtResult<Vec<EntryAspectData>> {
    match get_item_by_address(log_context, client, table_name, entry_address) {
        Ok(get_item_output) => match get_item_output {
            Some(item) => {
                let aspect_list = try_aspect_list_from_item(item)?;
                let mut aspects = Vec::new();
                for aspect_address in aspect_list {
                    aspects.push(
                        match get_aspect(log_context, client, table_name, &aspect_address) {
                            Ok(Some(aspect)) => aspect,
                            Ok(None) => {
                                return Err(BbDhtError::MissingData(format!(
                                    "Missing entry aspect data: {:?}",
                                    &aspect_address
                                )))
                            }
                            Err(err) => return Err(err),
                        },
                    )
                }
                Ok(aspects)
            }
            None => Ok(Vec::new()),
        },
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
pub mod tests {

    use crate::dht::bbdht::dynamodb::api::aspect::read::get_aspect;
    use crate::dht::bbdht::dynamodb::api::aspect::read::get_entry_aspects;
    use crate::dht::bbdht::dynamodb::api::aspect::write::append_aspect_list_to_entry;
    use crate::dht::bbdht::dynamodb::api::aspect::write::put_aspect;
    use crate::dht::bbdht::dynamodb::api::table::create::ensure_cas_table;
    use crate::dht::bbdht::dynamodb::api::table::exist::table_exists;
    use crate::dht::bbdht::dynamodb::api::table::fixture::table_name_fresh;
    use crate::dht::bbdht::dynamodb::client::local::local_client;
    use crate::test::unordered_vec_compare;
    use crate::trace::tracer;
    use crate::workflow::fixture::aspect_list_fresh;
    use crate::workflow::fixture::entry_address_fresh;
    use crate::workflow::fixture::entry_aspect_data_fresh;
    use lib3h_protocol::data_types::EntryAspectData;

    #[test]
    fn get_entry_aspects_test() {
        let log_context = "get_entry_aspects_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let entry_address = entry_address_fresh();
        let aspect_list = aspect_list_fresh();

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // cas exists
        assert!(table_exists(&log_context, &local_client, &table_name).is_ok());

        // empty aspect list
        match get_entry_aspects(&log_context, &local_client, &table_name, &entry_address) {
            Ok(aspects) => {
                let expected: Vec<EntryAspectData> = Vec::new();
                assert_eq!(expected, aspects);
            }
            Err(err) => {
                panic!("found entry aspects before adding list {:?}", err);
            }
        }

        // put aspect list
        assert!(append_aspect_list_to_entry(
            &log_context,
            &local_client,
            &table_name,
            &entry_address,
            &aspect_list
        )
        .is_ok());

        // get aspect list
        match get_entry_aspects(&log_context, &local_client, &table_name, &entry_address) {
            Ok(aspects) => {
                assert!(unordered_vec_compare(aspect_list, aspects));
            }
            Err(err) => {
                panic!("no aspects found {:?}", err);
            }
        }
    }

    #[test]
    fn read_aspect_test() {
        let log_context = "read_aspect_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();
        let table_name = table_name_fresh();
        let entry_aspect_data = entry_aspect_data_fresh();

        // ensure cas
        assert!(ensure_cas_table(&log_context, &local_client, &table_name).is_ok());

        // cas exists
        assert!(table_exists(&log_context, &local_client, &table_name).is_ok());

        // put aspect
        assert!(put_aspect(&log_context, &local_client, &table_name, &entry_aspect_data).is_ok());

        // get aspect
        match get_aspect(
            &log_context,
            &local_client,
            &table_name,
            &entry_aspect_data.aspect_address,
        ) {
            Ok(Some(v)) => {
                println!("{:#?}", v);
                assert_eq!(v.aspect_address, entry_aspect_data.aspect_address,);
                assert_eq!(v.aspect_address, entry_aspect_data.aspect_address,);
                assert_eq!(v.type_hint, entry_aspect_data.type_hint,);
                assert_eq!(v.aspect, entry_aspect_data.aspect,);
                assert_eq!(v.publish_ts, entry_aspect_data.publish_ts,);
            }
            Ok(None) => {
                panic!("get_aspect None");
            }
            Err(err) => {
                tracer(&log_context, "get_aspect Err");
                panic!("{:#?}", err);
            }
        }
    }

}
