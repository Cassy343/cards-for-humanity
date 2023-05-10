use either::Either;
use tokio::sync::{mpsc, oneshot};

const INTERNAL_COMM_ERROR: &str = "Internal channel disconnected";

pub fn channel<T>() -> (Tx<T>, Rx<T>) {
    let (inner_tx, inner_rx) = mpsc::unbounded_channel();
    (Tx { inner: inner_tx }, Rx { inner: inner_rx })
}

pub struct Tx<T> {
    inner: mpsc::UnboundedSender<T>,
}

impl<T> Clone for Tx<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Tx<T> {
    pub async fn try_send<R>(&self, message: R) -> Option<R::Response>
    where R: Message<T> {
        let (wrapped, response) = message.wrap();
        self.inner.send(wrapped).ok()?;
        Some(match response {
            Either::Left(response) => response,
            Either::Right(response_rx) => response_rx.recv().await,
        })
    }

    pub async fn send<R>(&self, message: R) -> R::Response
    where R: Message<T> {
        self.try_send(message).await.expect(INTERNAL_COMM_ERROR)
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

pub trait Message<T>: Sized {
    type Response;

    fn wrap(self) -> (T, Either<Self::Response, OneshotRx<Self::Response>>);
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

#[macro_export]
macro_rules! proto {
    (
        $name:ident,
        with_response: { $( $wr:ident: $wr_resp_ty:ty ),* },
        without_response: [$( $wor:ident ),*]
    ) => {
        pub enum $name {
            $(
                $wr(
                    $wr,
                    crate::chan::OneshotTx<$wr_resp_ty>
                ),
            )*
            $(
                $wor($wor),
            )*
        }

        $(
            impl crate::chan::Message<$name> for $wr {
                type Response = $wr_resp_ty;

                fn wrap(self) -> ($name, either::Either<Self::Response, crate::chan::OneshotRx<Self::Response>>) {
                    let (tx, rx) = crate::chan::oneshot();
                    ($name::$wr(self, tx), either::Either::Right(rx))
                }
            }
        )*

        $(
            impl crate::chan::Message<$name> for $wor {
                type Response = ();

                fn wrap(self) -> ($name, either::Either<(), crate::chan::OneshotRx<()>>) {
                    ($name::$wor(self), either::Either::Left(()))
                }
            }
        )*
    };
}
