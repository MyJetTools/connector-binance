pub use crate::common::{Coin, Id, MarketType, OrderInfo, Side, Symbol, TradeInfo};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WsChannel {
    DepthOrderbook(String),
    BookTicker(String),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum WsDataEvent {
    DepthOrderbook(DepthOrderbookData),
    BookTicker(BookTickerData),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookTickerData {
    #[serde(rename = "u")]
    pub update_id: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "b")]
    pub best_bid: String,

    #[serde(rename = "B")]
    pub best_bid_qty: String,

    #[serde(rename = "a")]
    pub best_ask: String,

    #[serde(rename = "A")]
    pub best_ask_qty: String,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderbookEventType {
    DepthUpdate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepthOrderbookData {
    #[serde(rename = "e")]
    pub event_type: OrderbookEventType,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "U")]
    pub first_update_id: u64,

    #[serde(rename = "u")]
    pub final_update_id: u64,

    #[serde(rename = "pu")]
    #[serde(default)]
    pub previous_final_update_id: Option<u64>,

    #[serde(rename = "b")]
    pub bids: Vec<(Decimal, Decimal)>,

    #[serde(rename = "a")]
    pub asks: Vec<(Decimal, Decimal)>,
}