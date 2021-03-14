use wasm_bindgen::{JsCast, JsValue, prelude::*};
use web_sys::{WebSocket as WebSysSocket, MessageEvent};
use std::convert::AsRef;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display, Formatter};

pub struct WebSocket(WebSysSocket);

impl WebSocket {
    pub fn connect<S: AsRef<str> + ?Sized>(url: &S) -> Result<Self, SocketError> {
        Ok(WebSocket(WebSysSocket::new(url.as_ref())?))
    }

    pub fn onopen<F>(&self, callback: F)
    where
        F: FnMut() + 'static
    {
        let closure = Closure::<dyn FnMut()>::new(callback);
        self.0.set_onopen(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    pub fn onmessage<F>(&self, callback: F)
    where
        F: FnMut(MessageEvent) + 'static
    {
        let closure = Closure::<dyn FnMut(MessageEvent)>::new(callback);
        self.0.set_onmessage(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
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