use super::binance_ws_settings::{BinanceWsSetting, BinanceWsSettingWrapper};
use super::event_handler::*;
use super::models::*;
use my_web_socket_client::WsCallback;
use my_web_socket_client::WsConnection;
use my_web_socket_client::{StartWsConnectionDataToApply, WebSocketClient};
use rust_extensions::Logger;
use serde_json::Error;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::tungstenite::{Bytes, Message};

pub struct BinanceWsClient {
    event_handler: Arc<dyn EventHandler + Send + Sync + 'static>,
    inner_ws_client: Mutex<Option<WebSocketClient>>,
    settings: Arc<BinanceWsSettingWrapper>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl BinanceWsClient {
    pub fn new(
        event_handler: Arc<dyn EventHandler + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
        settings: Arc<dyn BinanceWsSetting + Send + Sync + 'static>,
    ) -> Self {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");
        let settings = Arc::new(BinanceWsSettingWrapper::new(settings));

        Self {
            event_handler,
            inner_ws_client: Default::default(),
            logger,
            settings,
        }
    }

    pub fn start(ws_client: Arc<BinanceWsClient>) {
        let inner = WebSocketClient::new(
            Arc::new("Binance".into()),
            ws_client.settings.clone(),
            ws_client.logger.clone(),
        );
        inner.start(Some(Message::Ping(Bytes::default())), ws_client.clone());
        ws_client.inner_ws_client.lock().unwrap().replace(inner);
    }

    fn parse_msg(&self, msg: &str) -> Result<WsDataEvent, String> {
        let value: Result<serde_json::Value, Error> = serde_json::from_str(msg);

        if let Ok(value) = value {
            if let Some(data) = value.get("data") {
                return self.parse_msg(&data.to_string());
            }

            if let Ok(event) = serde_json::from_value::<WsDataEvent>(value) {
                return Ok(event);
            }
        }

        Err(format!("Failed to parse message: {}", msg))
    }
}

#[async_trait::async_trait]
impl WsCallback for BinanceWsClient {
    async fn before_start_ws_connect(
        &self,
        _url: String,
    ) -> Result<StartWsConnectionDataToApply, String> {
        Ok(StartWsConnectionDataToApply::default())
    }

    async fn on_connected(&self, _: Arc<WsConnection>) {
        self.logger.write_info(
            "BinanceWsClient".to_string(),
            "Connected to Binance websocket".to_string(),
            None,
        );
        self.event_handler.on_connected().await;
    }

    async fn on_disconnected(&self, _: Arc<WsConnection>) {}

    async fn on_data(&self, connection: Arc<WsConnection>, data: Message) {
        match data {
            Message::Text(msg) => {
                let event = self.parse_msg(&msg);
                match event {
                    Ok(event) => {
                        self.event_handler.on_data(event).await;
                    }
                    Err(err) => {
                        self.logger.write_info(
                            "BinanceWsClient".to_string(),
                            format!("Disconnecting... {} ", err),
                            None,
                        );
                        connection.disconnect().await;
                    }
                }
            }
            Message::Ping(_) => {
                connection
                    .send_message(Message::Ping(Bytes::default()))
                    .await;
            }
            Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => (),
            Message::Close(_) => {
                self.logger.write_info(
                    "BinanceWsClient".to_string(),
                    "Disconnecting... Received close ws message".to_string(),
                    None,
                );
            }
        }
    }
}
