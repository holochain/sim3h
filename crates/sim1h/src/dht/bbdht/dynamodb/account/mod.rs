use crate::dht::bbdht::dynamodb::client::Client;
use crate::trace::tracer;
use crate::trace::LogContext;
use rusoto_core::RusotoError;
use rusoto_dynamodb::DescribeLimitsError;
use rusoto_dynamodb::DescribeLimitsOutput;
use rusoto_dynamodb::DynamoDb;

pub fn describe_limits(
    log_context: &LogContext,
    client: &Client,
) -> Result<DescribeLimitsOutput, RusotoError<DescribeLimitsError>> {
    tracer(&log_context, "begin: describe_limits");
    let result = client.describe_limits().sync();
    tracer(&log_context, "complete: describe_limits");
    result
}

#[cfg(test)]
pub mod tests {

    use crate::dht::bbdht::dynamodb::account::describe_limits;
    use crate::dht::bbdht::dynamodb::client::fixture::bad_client;
    use crate::dht::bbdht::dynamodb::client::local::local_client;
    use crate::trace::tracer;

    #[test]
    fn describe_limits_ok_test() {
        let log_context = "describe_limits_ok_test";

        tracer(&log_context, "fixtures");
        let local_client = local_client();

        // describe limits
        assert!(describe_limits(&log_context, &local_client).is_ok());
    }

    #[test]
    fn describe_limits_bad_test() {
        let log_context = "describe_limits_bad_test";

        tracer(&log_context, "fixtures");
        let bad_client = bad_client();

        // fail to describe limits
        assert!(describe_limits(&log_context, &bad_client).is_err());
    }

}
