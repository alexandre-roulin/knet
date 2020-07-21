# Knet

A server and a client that can write and read data asynchronously to and from a T-type data derived by the knet-derive crate.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/knet.svg
[crates-url]: https://crates.io/crates/knet
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE

[API Docs](https://docs.rs/knet/0.3.1/knet/)

# Example 

```
#[derive(DeriveKnet, Debug, Clone, Copy)]
enum Data {
  Integer(i32),
  Char(char)
}

Server::write_all(server.0.clone(), Data::Integer(32)).await?;
Server::write(server.0.clone(), Data::Integer(32), 0).await?;
Client::write_all(server.0.clone(), Data::Integer(32)).await?;

loop {
    match receiver.try_next() {
       Ok(Some(event)) => {
            println!("Receive event<T> {:?} ", event);
         }
        Ok(None) => {
            eprintln!("Connection is down");
            break;
        }
        Err(e) => {
            error!("Nothing receive from receiver", e);
        }
    }
}
```
