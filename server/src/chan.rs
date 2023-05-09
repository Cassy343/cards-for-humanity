use either::Either;
use tokio::sync::{mpsc, oneshot};

const INTERNAL_COMM_ERROR: &str = "Internal channel disconnected";

pub struct Tx<T> {
    inner: mpsc::UnboundedSender<T>,
}

impl<T> Tx<T> {
    pub async fn try_send<R>(&self, request: R) -> Option<R::Response>
    where R: Request<T> {
        let (wrapped, response) = request.wrap();
        self.inner.send(wrapped).ok()?;
        Some(match response {
            Either::Left(response) => response,
            Either::Right(response_rx) => response_rx.recv().await,
        })
    }

    pub async fn send<R>(&self, request: R) -> R::Response
    where R: Request<T> {
        self.try_send(request).await.expect(INTERNAL_COMM_ERROR)
    }
}

pub struct Rx<T> {
    inner: mpsc::UnboundedReceiver<T>,
}

impl<T> Rx<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.inner.recv().await
    }
}

pub trait Request<T>: Sized {
    type Response;

    fn wrap(self) -> (T, Either<Self::Response, OneshotRx<Self::Response>>) {
        let (tx, rx) = oneshot();
        (self.wrap_with(tx), Either::Right(rx))
    }

    fn wrap_with(self, channel: OneshotTx<Self::Response>) -> T;
}

pub trait RequestWithoutResponse<T>: Into<T> {}

impl<T, U> Request<T> for U
where U: RequestWithoutResponse<T>
{
    type Response = ();

    fn wrap(self) -> (T, Either<Self::Response, OneshotRx<Self::Response>>) {
        (self.into(), Either::Left(()))
    }

    fn wrap_with(self, _channel: OneshotTx<Self::Response>) -> T {
        self.into()
    }
}

pub fn oneshot<T>() -> (OneshotTx<T>, OneshotRx<T>) {
    let (inner_tx, inner_rx) = oneshot::channel();
    (OneshotTx { inner: inner_tx }, OneshotRx { inner: inner_rx })
}

pub struct OneshotTx<T> {
    inner: oneshot::Sender<T>,
}

impl<T> OneshotTx<T> {
    pub fn send(self, value: T) {
        self.inner
            .send(value)
            .map_err(drop)
            .expect(INTERNAL_COMM_ERROR)
    }
}

pub struct OneshotRx<T> {
    inner: oneshot::Receiver<T>,
}

impl<T> OneshotRx<T> {
    pub async fn recv(self) -> T {
        self.inner.await.expect(INTERNAL_COMM_ERROR)
    }
}
