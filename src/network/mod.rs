mod client;
mod server;
pub mod trait_knet;
use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    sync::{Arc, Mutex},
    task,
};
pub use client::Client;
use futures::{channel::mpsc, select, FutureExt, SinkExt};
pub use log::*;
pub use server::{Event, Server};
use std::fmt::Debug;
use std::marker::PhantomData;
pub use trait_knet::KnetTransform;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
pub type Id = u8;
use std::marker::Sized;
fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + Sync + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            warn!("{}", e)
        }
    })
}
