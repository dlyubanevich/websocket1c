use addin1c::{name, AddinResult, CStr1C, MethodInfo, Methods, PropInfo, SimpleAddin, Variant};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{collections::HashMap, error::Error};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_tungstenite::{
    connect_async,
    tungstenite::http::{Request, Uri},
    MaybeTlsStream, WebSocketStream,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct WebSocket1CAddIn {
    runtime: Arc<Runtime>,
    websocket: Option<WebSocketConnection>,
    last_error: Option<Box<dyn Error>>,
}

impl WebSocket1CAddIn {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::default())
    }

    fn connect(
        &mut self,
        address: &mut Variant,
        json_headers: &mut Variant,
        return_value: &mut Variant,
    ) -> AddinResult {
        self.runtime.clone().block_on(async {
            let request_data =
                RequestData::try_new(address.get_string()?, json_headers.get_string()?)?;
            let request = Request::try_from(request_data)?;
            let (stream, _) = connect_async(request)
                .await
                .map_err(|error| format!("{error}"))?;
            let (sender, receiver) = stream.split();
            self.websocket = Some(WebSocketConnection { sender, receiver });
            return_value.set_bool(true);
            Ok(())
        })
    }
    fn send(&mut self, message: &mut Variant, return_value: &mut Variant) -> AddinResult {
        self.runtime.clone().block_on(async {
            let message = message.get_string()?;
            match self.websocket.as_mut() {
                None => Err("Отсутствует установленное соединение!".to_owned().into()),
                Some(websocket) => {
                    websocket
                        .sender
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            message.into(),
                        ))
                        .await?;
                    return_value.set_bool(true);
                    Ok(())
                }
            }
        })
    }

    fn receive(&mut self, timeout: &mut Variant, return_value: &mut Variant) -> AddinResult {
        self.runtime.clone().block_on(async {
            match self.websocket.as_mut() {
                None => Err("Отсутствует установленное соединение!".to_owned().into()),
                Some(websocket) => {
                    let timeout = timeout.get_i32()?;
                    match tokio::time::timeout(
                        Duration::from_millis(timeout as u64),
                        websocket.receiver.next(),
                    )
                    .await
                    {
                        Err(_) | Ok(None) => {
                            return_value.set_str1c("")?;
                            Ok(())
                        }
                        Ok(Some(result)) => {
                            let message = result?.to_text()?.to_owned();
                            return_value.set_str1c(message)?;
                            Ok(())
                        }
                    }
                }
            }
        })
    }
    fn disconnect(&mut self, return_value: &mut Variant) -> AddinResult {
        self.websocket = None;
        return_value.set_bool(true);
        Ok(())
    }
    fn version(&mut self, return_value: &mut Variant) -> AddinResult {
        return_value.set_str1c(VERSION.to_owned())?;
        Ok(())
    }
    fn last_error(&mut self, return_value: &mut Variant) -> AddinResult {
        match self.last_error.as_ref() {
            Some(err) => return_value
                .set_str1c(err.to_string().as_str())
                .map_err(|e| e.into()),
            None => return_value.set_str1c("").map_err(|e| e.into()),
        }
    }
}

impl SimpleAddin for WebSocket1CAddIn {
    fn name() -> &'static CStr1C {
        name!("WebSocket1CAddIn")
    }
    fn save_error(&mut self, err: Option<Box<dyn Error>>) {
        self.last_error = err;
    }
    fn methods() -> &'static [MethodInfo<Self>] {
        &[
            MethodInfo {
                name: name!("Подключиться"),
                method: Methods::Method2(Self::connect),
            },
            MethodInfo {
                name: name!("ОтправитьСообщение"),
                method: Methods::Method1(Self::send),
            },
            MethodInfo {
                name: name!("ПолучитьСообщение"),
                method: Methods::Method1(Self::receive),
            },
            MethodInfo {
                name: name!("Отключиться"),
                method: Methods::Method0(Self::disconnect),
            },
            MethodInfo {
                name: name!("Версия"),
                method: Methods::Method0(Self::version),
            },
        ]
    }

    fn properties() -> &'static [PropInfo<Self>] {
        &[PropInfo {
            name: name!("ОписаниеОшибки"),
            getter: Some(Self::last_error),
            setter: None,
        }]
    }
}

impl Default for WebSocket1CAddIn {
    fn default() -> Self {
        Self {
            last_error: None,
            websocket: None,
            runtime: Arc::new(Runtime::new().unwrap()),
        }
    }
}

struct WebSocketConnection {
    sender: SplitSink<
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::Message,
    >,
    receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

#[derive(Default)]
struct RequestData {
    address: String,
    headers: HashMap<String, String>,
}

impl RequestData {
    fn try_new(address: String, json_headers: String) -> Result<Self, String> {
        let headers = if !json_headers.is_empty() {
            serde_json::from_str::<HashMap<String, serde_json::Value>>(&json_headers)
                .map_err(|error| error.to_string())?
                .into_iter()
                .map(|(key, value)| {
                    let value = match value {
                        serde_json::Value::Null => "".to_owned(),
                        serde_json::Value::Bool(bool) => bool.to_string(),
                        serde_json::Value::Number(number) => number.to_string(),
                        serde_json::Value::String(str) => str,
                        serde_json::Value::Array(_) => "".to_owned(),
                        serde_json::Value::Object(_) => "".to_owned(),
                    };
                    (key, value)
                })
                .collect()
        } else {
            HashMap::default()
        };

        Ok(RequestData { address, headers })
    }
}

impl TryFrom<RequestData> for Request<()> {
    type Error = String;

    fn try_from(data: RequestData) -> Result<Self, Self::Error> {
        let uri = data.address.parse::<Uri>().map_err(|err| err.to_string())?;
        let authority = uri
            .authority()
            .ok_or("No host name in the URL".to_string())?
            .as_str();
        let host = authority
            .find('@')
            .map(|idx| authority.split_at(idx + 1).1)
            .unwrap_or_else(|| authority);

        let websocket_key = tokio_tungstenite::tungstenite::handshake::client::generate_key();

        let mut request_builder = Request::get(data.address.as_str())
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", websocket_key);

        for (key, value) in data.headers.iter() {
            request_builder = request_builder.header(key.as_str(), value.as_str());
        }
        request_builder.body(()).map_err(|error| error.to_string())
    }
}
