pub use super::*;
#[derive(Debug)]
enum InnerEvent<T> {
    NewPeer {
        id: Id,
        stream: Arc<TcpStream>,
        shutdown: Receiver<()>,
    },
    Data((Id, T)),
}

#[derive(Debug)]
pub enum Event<T> {
    NewConnection(Id),
    Data((Id, T)),
    ConnectionDrop(Id),
}

pub struct Connection<T> {
    id: Id,
    sender: Sender<InnerEvent<T>>,
}

impl<T> Connection<T> {
    fn new(id: Id, sender: Sender<InnerEvent<T>>) -> Self {
        Connection { id, sender }
    }
}
pub struct InnerServer<T> {
    p: PhantomData<T>,
    connections: Vec<Option<Connection<T>>>,
    broker: Option<task::JoinHandle<Result<()>>>,
}
type ProtectedServer<T> = Arc<Mutex<InnerServer<T>>>;
pub struct Server<T>(pub ProtectedServer<T>);

impl<T: KnetTransform + Send + Debug + Sync + Clone + 'static> Server<T> {
    ///Run the server on the `addr` adress and return and the server and Receiver 
    pub async fn run(addr: impl ToSocketAddrs + Sync + Send) -> Result<(Self, Receiver<Event<T>>)> {
        let listener = TcpListener::bind(addr).await?;
        let (sender_event, receiver_event) = mpsc::unbounded::<Event<T>>();
        let (sender_event_connection, receiver_event_connection) =
            mpsc::unbounded::<InnerEvent<T>>();
        let server = Server::<T>(Arc::new(Mutex::new(InnerServer {
            p: PhantomData,
            connections: Vec::new(),
            broker: None,
        })));
        server.0.lock().await.broker = Some(task::spawn(Server::broker_loop(
            server.0.clone(),
            sender_event,
            receiver_event_connection,
        )));

        spawn_and_log_error(Server::accept_loop(
            server.0.clone(),
            listener,
            sender_event_connection,
        ));
        Ok((server, receiver_event))
    }

    ///Write the data to all connection accept by the server
    pub async fn write_all(server: ProtectedServer<T>, data: T) -> Result<()> {
        for option in server.lock().await.connections.iter_mut() {
            if let Some(connection) = option {
                let id = connection.id;
                connection
                    .sender
                    .send(InnerEvent::Data((id, data)))
                    .await?;
            }
        }
        Ok(())
    }

    ///Write the data to a specify connection
    pub async fn write(server: ProtectedServer<T>, data: T, id: Id) -> Result<()> {
        server
            .lock()
            .await
            .connections
            .get_mut::<usize>(id.into())
            .unwrap()
            .as_mut()
            .unwrap()
            .sender
            .send(InnerEvent::Data((id, data)))
            .await?;

        Ok(())
    }
    async fn accept_loop(
        server: ProtectedServer<T>,
        listener: TcpListener,
        sender_event_connection: Sender<InnerEvent<T>>,
    ) -> Result<()> {
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            info!("Accepting connection from : {:?}", stream.peer_addr());
            spawn_and_log_error(Server::connection_loop(
                server.clone(),
                sender_event_connection.clone(),
                stream,
            ));
        }
        drop(sender_event_connection);
        server.lock().await.broker.take().unwrap().await
    }

    async fn connection_loop(
        server: ProtectedServer<T>,
        mut sender_event_connection: Sender<InnerEvent<T>>,
        stream: TcpStream,
    ) -> Result<()> {
        let (_shutdown_sender, shutdown) = mpsc::unbounded();

        let stream = Arc::new(stream);
        let mut reader = BufReader::new(&*stream);
        let id = {
            let connection = &server.lock().await.connections;
            connection
                .iter()
                .position(|opt| opt.is_none())
                .unwrap_or(connection.len()) as Id
        };

        sender_event_connection
            .send(InnerEvent::NewPeer {
                id,
                stream: Arc::clone(&stream),
                shutdown,
            })
            .await?;

        let size_payload = T::get_size_of_payload();
        loop {
            let mut vector_payload = vec![0u8; size_payload];
            let mut buffer_payload = vector_payload[..].as_mut();

            match reader.read_exact(&mut buffer_payload).await {
                Ok(_) => {
                    let size_data = T::get_size_of_data(&buffer_payload);
                    let mut vector_data = vec![0u8; size_data];
                    let buffer_data = vector_data[..].as_mut();

                    if let Err(e) = reader.read_exact(buffer_data).await {
                        error!("error read data {}", e);
                    }
                    vector_payload.extend_from_slice(&buffer_data);
                    let data = T::from_raw(&vector_payload);
                    sender_event_connection
                        .send(InnerEvent::Data((id, data)))
                        .await?;
                }
                Err(e) => {
                    warn!("Error in connection_loop : {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn broker_loop(
        server: ProtectedServer<T>,
        mut sender_event: Sender<Event<T>>,
        mut receiver_event_connection: Receiver<InnerEvent<T>>,
    ) -> Result<()> {
        info!("Starting broker loop");
        let (disconnect_sender, mut disconnect_receiver) =
            mpsc::unbounded::<(Id, Receiver<InnerEvent<T>>)>();
        loop {
            let event: InnerEvent<T> = select! {
                event = receiver_event_connection.next().fuse() => match event {
                    Some(event) => {
                        info!("broker_loop event");
                        event},
                    None => {
                        info!("broker_loop None");
                        break
                    }
                },
                disconnect = disconnect_receiver.next().fuse() => {
                    let (id, _) = disconnect.unwrap();
                    //Todo erase the connection if deconnection
                    info!("broker_loop disconnect");
                    let _ = server.lock().await.connections.get_mut::<usize>(id.into()).unwrap().take();
                    continue;
                }
            };

            match event {
                InnerEvent::NewPeer {
                    id,
                    stream,
                    shutdown,
                } => {
                    info!("New Peer {:?}", id);

                    let (client_sender, mut client_receiver) = mpsc::unbounded();
                    server
                        .lock()
                        .await
                        .connections
                        .insert(id.into(), Some(Connection::new(id, client_sender)));
                    let mut disconnect_sender = disconnect_sender.clone();
                    spawn_and_log_error(async move {
                        let res =
                            Server::connection_writer_loop(&mut client_receiver, stream, shutdown)
                                .await;
                        disconnect_sender.send((id, client_receiver)).await.unwrap();
                        res
                    });
                }
                InnerEvent::Data((id, data)) => {
                    info!("New event {:?}", data);
                    sender_event.send(Event::Data((id, data))).await?;
                }
            }
        }
        server.lock().await.connections.clear();
        drop(disconnect_sender);
        while disconnect_receiver.next().await.is_some() {}
        Ok(())
    }

    async fn connection_writer_loop(
        messages: &mut Receiver<InnerEvent<T>>,
        stream: Arc<TcpStream>,
        mut shutdown: Receiver<()>,
    ) -> Result<()> {
        let mut stream = &*stream;
        loop {
            select! {
                msg = messages.next().fuse() => match msg {
                    Some(msg) =>{
                        info!("connection_writer_loop msg {:?}", msg);
                        if let InnerEvent::Data((id, data)) = msg {
                            stream.write_all(&data.serialize()[..]).await?
                        }
                    },
                    None => break,
                },
                void = shutdown.next().fuse() => match void {
                    Some(_) => {
                        info!("connection_writer_loop void");

                    },
                    None => break,
                }
            }
        }
        Ok(())
    }
}
