use native_api_1c::{native_api_1c_core::ffi::connection::Connection, native_api_1c_macro::AddIn};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(AddIn)]
pub struct WebSocket1CAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    runtime: Arc<Runtime>,
    websocket: Option<WebSocketConnection>,

    #[add_in_prop(ty = Str, name = "LastError", name_ru = "ОписаниеОшибки", readable)]
    pub last_error: String,

    #[add_in_func(name = "Connect", name_ru = "Подключиться")]
    #[arg(Str)]
    #[returns(None, result)]
    pub connect: fn(&mut Self, String) -> Result<bool, ()>,

    #[add_in_func(name = "SendMessage", name_ru = "ОтправитьСообщение")]
    #[arg(Str)]
    #[returns(Bool, result)]
    pub send: fn(&mut Self, String) -> Result<bool, ()>,

    #[add_in_func(name = "ReceiveMessage", name_ru = "ПолучитьСообщение")]
    #[returns(Str, result)]
    pub receive: fn(&mut Self) -> Result<String, ()>,

    #[add_in_func(name = "Disconnect", name_ru = "Отключиться")]
    #[returns(Bool, result)]
    pub disconnect: fn(&mut Self) -> Result<bool, ()>,
}

impl WebSocket1CAddIn {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            last_error: "".to_owned(),
            websocket: None,
            runtime: Arc::new(Runtime::new().unwrap()),
            connect: Self::connect,
            send: Self::send,
            receive: Self::receive,
            disconnect: Self::disconnect,
        }
    }
    fn connect(&mut self, address: String) -> Result<bool, ()> {
        self.runtime.clone().block_on(async {
            let stream = match connect_async(&address).await {
                Ok((stream,_)) => stream,
                Err(err) => {
                    return Err(self.handle_error(err.to_string()));
                },
            };
            let (sender, receiver) = stream.split();
            self.websocket = Some(WebSocketConnection { sender, receiver });
            Ok(true)
        })
    }

    fn send(&mut self, message: String) -> Result<bool, ()> {
        self.runtime.clone().block_on(async {
            match self.websocket.as_mut() {
                Some(websocket) => {
                    match websocket
                        .sender
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            message.into(),
                        ))
                        .await {
                            Ok(_) => Ok(true),
                            Err(err) => Err(self.handle_error(err.to_string())),
                        }
                }
                None => Err(self.handle_error("Отсутствует установленное соединение!".to_owned())),
            }
        })
    }
    fn receive(&mut self) -> Result<String, ()> {
        self.runtime.clone().block_on(async {
            match self.websocket.as_mut() {
                Some(websocket) => match websocket.receiver.next().await {
                    Some(result) => {
                        match result {
                            Ok(message) => {
                                match message.to_text() {
                                    Ok(message_text) => Ok(message_text.to_owned()),
                                    Err(err) => Err(self.handle_error(err.to_string())),
                                }   
                            },
                            Err(err) => Err(self.handle_error(err.to_string())),
                        }
                    },
                    None => Err(self.handle_error("Нет сообщений!".to_owned())),
                },
                None => Err(self.handle_error("Отсутствует установленное соединение!".to_owned())),
            }
        })
    }
    fn disconnect(&mut self) -> Result<bool, ()> {
        self.websocket = None;
        Ok(true)
    }
    fn handle_error(&mut self, err: String) -> () {
        self.last_error = err;
        ()
    }
}

struct WebSocketConnection {
    sender: SplitSink<
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::Message,
    >,
    receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}
