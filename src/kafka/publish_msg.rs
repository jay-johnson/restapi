//! Publish messages to kafka with a fire-and-forget approach
//!
use std::collections::HashMap;

use kafka_threadpool::kafka_publisher::KafkaPublisher;

/// publish_msg
///
/// Wrapper for
/// [`kafka_threadpool::kafka_publisher::KafkaPublisher::add_data_msg()`](kafka_threadpool::kafka_publisher::KafkaPublisher)
/// that will only publish to kafka if the environment variable ``KAFKA_ENABLED`` is ``true`` or ``1``
///
/// # Arguments
///
/// * `kafka_pool` - initialized [`KafkaPublisher`](kafka_threadpool::kafka_publisher::KafkaPublisher)
/// that can publish messages to the configured kafka cluster
/// * `topic` - kafka topic to publish the message into
/// * `key` - kafka partition key
/// * `headers` - optional - headers for the kafka message
/// * `payload` - data within the kafka message
///
pub async fn publish_msg(
    kafka_pool: &KafkaPublisher,
    topic: &str,
    key: &str,
    headers: Option<HashMap<String, String>>,
    payload: &str,
) {
    // if enabled, publish the event to kafka
    if kafka_pool.is_enabled() {
        match kafka_pool.add_data_msg(topic, key, headers, payload).await {
            Ok(res_str) => {
                trace!(
                    "kafka publisher: res={res_str} \
                    topic={topic} key={key}"
                )
            }
            Err(err_str) => {
                error!(
                    "failed to publish login to \
                    kafka with err={err_str}"
                )
            }
        }
    }
}
