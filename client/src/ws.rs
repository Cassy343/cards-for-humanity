use common::protocol::{encode, serverbound::ServerBoundPacket};
use serde::Serialize;
use std::{
    convert::AsRef,
    error::Error as StdError,
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};
use uuid::Uuid;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket as WebSysSocket};

macro_rules! set_handler {
    ($sock:expr, $event_type:ty, $handler:ident, $callback:expr) => {{
        let clone = $sock.clone();
        let closure = Closure::<dyn FnMut($event_type)>::new(move |event| $callback(&clone, event));
        $sock.0.$handler(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }};
}

pub struct WebSocket(Arc<WebSysSocket>);

impl WebSocket {
    pub fn connect<S: AsRef<str> + ?Sized>(url: &S) -> Result<Self, SocketError> {
        Ok(WebSocket(Arc::new(WebSysSocket::new(url.as_ref())?)))
    }

    pub fn onopen<F>(&self, mut callback: F)
    where F: FnMut(&Self) + 'static {
        let clone = self.clone();
        let closure = Closure::<dyn FnMut()>::new(move || callback(&clone));
        self.0.set_onopen(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    pub fn onmessage<F>(&self, mut callback: F)
    where F: FnMut(&Self, MessageEvent) + 'static {
        set_handler!(self, MessageEvent, set_onmessage, callback)
    }

    pub fn onclose<F>(&self, mut callback: F)
    where F: FnMut(&Self, CloseEvent) + 'static {
        set_handler!(self, CloseEvent, set_onclose, callback)
    }

    pub fn onerror<F>(&self, mut callback: F)
    where F: FnMut(&Self, ErrorEvent) + 'static {
        set_handler!(self, ErrorEvent, set_onerror, callback)
    }

    pub fn send_packet<P: Serialize>(&self, packet: &P) -> Result<(), SocketError> {
        self.send_packets(&[packet])
    }

    pub fn send_packet_with_id(&self, packet: ServerBoundPacket) -> Result<Uuid, SocketError> {
        let (packet, id) = packet.with_id();
        self.send_packets(&[packet]).map(|_| id)
    }

    pub fn send_packets<'a, T, P>(&self, packets: &'a T) -> Result<(), SocketError>
    where
        &'a T: IntoIterator<Item = &'a P>,
        P: Serialize + 'a,
    {
        let data = encode(&packets.into_iter().collect::<Vec<_>>());
        crate::console_log!("Sending packet data: {}", data);
        self.0.send_with_str(&data).map_err(Into::into)
    }
}

impl Clone for WebSocket {
    fn clone(&self) -> Self {
        WebSocket(Arc::clone(&self.0))
    }
}

pub struct SocketError(JsValue);

impl SocketError {
    fn new(inner: JsValue) -> Self {
        SocketError(inner)
    }
}

impl Debug for SocketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "WebSocket error: {:?}", self.0)
    }
}

impl Display for SocketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl StdError for SocketError {}

impl From<JsValue> for SocketError {
    fn from(inner: JsValue) -> Self {
        Self::new(inner)
    }
}
