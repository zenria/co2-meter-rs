use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use std::collections::HashMap;
use std::fmt::Debug;

pub trait MqttData {
    /// Get the data to send to MQTT topics.
    ///
    /// Each value in the returned hashmap will be published
    /// on the topic "base_topic" + "/" + "key"
    fn get_data(&self) -> HashMap<String, Vec<u8>>;
}

pub struct MqttConfig {
    port: u16,
    host: String,
    base_topic: String,
}

pub struct MqttSender {
    config: MqttConfig,
    mqtt_client: MqttClient,
}
impl MqttConfig {
    pub fn new(host: String, port: u16, base_topic: String) -> Self {
        let base_topic = MqttConfig::normalize_base_topic(&base_topic);
        MqttConfig {
            host,
            port,
            base_topic,
        }
    }

    fn normalize_base_topic(base_topic: &str) -> String {
        let mut normalized_base_topic = String::with_capacity(base_topic.len() + 1);
        normalized_base_topic.push_str(base_topic);
        if !base_topic.ends_with('/') {
            normalized_base_topic.push('/');
        }
        normalized_base_topic
    }

    fn get_topic(&self, metric_name: &str) -> String {
        let mut ret = String::with_capacity(metric_name.len() + self.base_topic.len());
        ret.push_str(&self.base_topic);
        ret.push_str(metric_name);
        ret
    }
}

impl MqttSender {
    pub fn new(config: MqttConfig) -> Self {
        let reconnection_options = ReconnectOptions::Always(10);
        let mqtt_options = MqttOptions::new("test-pubsub2", config.host.clone(), config.port)
            .set_keep_alive(10)
            .set_reconnect_opts(reconnection_options)
            .set_clean_session(false);

        let (mqtt_client, _) = match MqttClient::start(mqtt_options) {
            Err(e) => {
                error!("Unable to initialize MQTT client, {}", e);
                std::process::exit(1)
            }
            Ok(r) => r,
        };
        MqttSender {
            config,
            mqtt_client,
        }
    }

    pub fn send_data<D: MqttData + Debug>(&mut self, data: D) {
        for (k, v) in data.get_data() {
            if let Err(e) =
                self.mqtt_client
                    .publish(self.config.get_topic(&k), QoS::AtLeastOnce, false, v)
            {
                error!("Unable to send metric {}", e);
            } else {
                debug!("Data sent to MQTT {:?}", data);
            }
        }
    }
}
