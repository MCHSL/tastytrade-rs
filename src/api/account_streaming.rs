use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    accounts::{Account, Balance},
    Result, TastyTrade,
};

use super::order::LiveOrderRecord;

static WEBSOCKET_URL: &str = "wss://streamer.cert.tastyworks.com";

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SubRequestAction {
    Heartbeat,
    Connect,
    PublicWatchlistsSubscribe,
    QuoteAlertsSubscribe,
    UserMessageSubscribe,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct SubRequest<T> {
    auth_token: String,
    action: SubRequestAction,
    value: Option<T>,
}

pub struct HandlerAction {
    action: SubRequestAction,
    value: Option<Box<dyn erased_serde::Serialize + Send + Sync>>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum Notification {
    Order(LiveOrderRecord),
    AccountBalance(Balance),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct StatusMessage {
    pub status: String,
    pub action: String,
    pub web_socket_session_id: String,
    pub request_id: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ErrorMessage {
    pub status: String,
    pub action: String,
    pub web_socket_session_id: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SubMessage {
    ErrorMessage(ErrorMessage),
    StatusMessage(StatusMessage),
    Notification(Notification),
}

pub struct AccountWebsocketHandler {
    pub event_receiver: flume::Receiver<SubMessage>,
    pub action_sender: flume::Sender<HandlerAction>,
}

impl AccountWebsocketHandler {
    pub async fn connect<T: AsRef<str>>(token: T) -> Result<Self> {
        let token = token.as_ref().to_owned();
        let (event_sender, event_receiver) = flume::unbounded();
        let (action_sender, action_receiver): (
            flume::Sender<HandlerAction>,
            flume::Receiver<HandlerAction>,
        ) = flume::unbounded();

        let url = url::Url::parse(WEBSOCKET_URL).unwrap();

        let (ws_stream, _response) = connect_async(url).await?;
        println!("WebSocket handshake has been successfully completed");

        let (mut write, mut read) = ws_stream.split();

        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                let data = message.unwrap().into_data();
                println!("{:?}", String::from_utf8_lossy(&data));
                let data: SubMessage = serde_json::from_slice(&data).unwrap();
                event_sender.send_async(data).await.unwrap();
            }
        });

        let token_clone = token.clone();
        tokio::spawn(async move {
            while let Ok(action) = action_receiver.recv_async().await {
                let message = SubRequest {
                    auth_token: token_clone.clone(),
                    action: action.action,
                    value: action.value,
                };
                let message = serde_json::to_string(&message).unwrap();

                println!("{message:?}");

                let message = Message::Text(message);

                write.send(message).await.unwrap();
            }
        });

        let sender_clone = action_sender.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                sender_clone
                    .send_async(HandlerAction {
                        action: SubRequestAction::Heartbeat,
                        value: None,
                    })
                    .await
                    .unwrap();
            }
        });

        Ok(Self {
            event_receiver,
            action_sender,
        })
    }

    pub async fn subscribe_to_account<'a>(&self, account: &'a Account<'a>) {
        self.send(
            SubRequestAction::Connect,
            Some(vec![account.inner.account.account_number.clone()]),
        )
        .await;
    }

    pub async fn send<T: Serialize + Send + Sync + 'static>(
        &self,
        action: SubRequestAction,
        value: Option<T>,
    ) {
        self.action_sender
            .send_async(HandlerAction {
                action,
                value: value
                    .map(|inner| Box::new(inner) as Box<dyn erased_serde::Serialize + Send + Sync>),
            })
            .await
            .unwrap();
    }

    pub async fn handle_events<F>(&mut self, f: F)
    where
        F: Fn(SubMessage),
    {
        while let Ok(ev) = self.event_receiver.recv_async().await {
            f(ev);
        }
    }
}

impl TastyTrade {
    pub async fn create_account_streamer(&self) -> Result<AccountWebsocketHandler> {
        AccountWebsocketHandler::connect(&self.session_token).await
    }
}
