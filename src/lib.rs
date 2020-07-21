#![crate_name = "knet"]
//! A Server and Client with parrallel read/write with `async_std` library.
//! 
//! Internal data will be automatically serialized and de-serialized by the trait  [`KnetTransform`].
//! 
//! It is advised to use the internal derive macro [`DeriveKnet`] on `enum`.
//!
//! Here is a basic example:
//!
//! [-> Server/Client](index.html#serverclient)
//! 
//! [-> DeriveKnet](index.html#deriveknet)
//!
//! [`DeriveKnet`]: ./derive.DeriveKnet.html
//! 
//! # Server/Client
//! The [`Server`] and the [`Client`] has the [`run`] that allows you to start asynchronously.
//! It return (`Self`, [`Receiver`])
//! 
//! You can read the [`Event`] with the `receiver` in a loop. 
//! 
//! ## Example 
//! ```
//! loop {
//!     match receiver.try_next() {
//!        Ok(Some(event)) => {
//!             println!("Receive event<T> {:?} ", event);
//!          }
//!         Ok(None) => {
//!             eprintln!("Connection is down");
//!             break;
//!         }
//!         Err(e) => {
//!             error!("Nothing receive from receiver", e);
//!         }
//!     }
//! }
//! ```

//! There is small difference between them : 
//! * [`Server::write_all`] and [`Client::write`] have similar prototype but just difference name.
//! It allow you to send the `T` over the network.
//! * [`Server::write`] is the same as above but have a [`Id`] paramater to precise on which connection you want to send the data.
//! * The [`Receiver`] from the `Server` have a wrapper [`Event`].
//! 
//! 
//! [`Server::write_all`]: ./struct.Server.html#method.write_all
//! [`Server::write`]: ./struct.Server.html#method.write
//! [`Client::write`]: ./struct.Client.html#method.write
//! 
//! [`Receiver`]: ./futures_channel/mpsc/struct.UnboundedReceiver.html
//! [`run`]: ./struct.Server.html#method.run
//! [`Client`]: ./struct.Client.html
//! [`Server`]: ./struct.Server.html
//! 
//! # DeriveKnet
//! 
//! The derive macro allow you to automatically implement the trait [`KnetTransform`].
//! 
//! ## Example
//! 
//! ```
//! #[derive(DeriveKnet, Debug, PartialEq, Clone, Copy)]
//! enum Data {
//!     Byte(u8),
//!     Integer(i32),
//!     Char(char),
//!     Float(f64),
//! }
//! ```
//!
//! [`Event`]: ./enum.Event.html
//! [`KnetTransform`]: ./trait.KnetTransform.html
//! [`Id`]: ./type.Id.html
//! 

#[doc(hidden)]
mod network;
pub use derive_knet::DeriveKnet;
pub use network::Client;
pub use network::KnetTransform;
pub use network::Server;
pub use network::Id;
pub use network::Event;
