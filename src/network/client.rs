use super::*;

pub struct InnerClient<T> {
    sender: Sender<T>,
    t: PhantomData<T>,
}

type ProtectedClient<T> = Arc<Mutex<InnerClient<T>>>;
#[derive(Clone)]
pub struct Client<T>(pub ProtectedClient<T>);

impl<T: KnetTransform + Send + Clone + Sync + Debug + 'static> Client<T> {
    /* ******************************************************* */
    /*           Public                                        */
    /* ******************************************************* */
    ///
    pub async fn run(addr: impl ToSocketAddrs) -> Result<(Self, Receiver<T>)> {
        let stream = TcpStream::connect(addr).await?;
        let stream = Arc::new(stream);
        let (sender_event, receiver_event) = mpsc::unbounded::<T>();
        let (sender, receiver) = mpsc::unbounded::<T>();
        let (sender_shutdown, receiver_shutdown) = mpsc::unbounded::<()>();
        let client = Client(Arc::new(Mutex::new(InnerClient {
            sender,
            t: PhantomData,
        })));
        spawn_and_log_error(Client::loop_read(
            client.0.clone(),
            Arc::clone(&stream),
            sender_event,
            sender_shutdown,
        ));
        spawn_and_log_error(Client::connection_writer_loop(
            receiver,
            stream,
            receiver_shutdown,
        ));
        Ok((client, receiver_event))
    }

    pub async fn write(client: ProtectedClient<T>, data: T) -> Result<()> {
        info!("dota dead");
        client.lock().await.sender.send(data).await?;
        Ok(())
    }

    /* ******************************************************* */
    /*           Private                                       */
    /* ******************************************************* */

    async fn loop_read(
        client: ProtectedClient<T>,
        stream: Arc<TcpStream>,
        mut sender_event: Sender<T>,
        _sender_shutdown: Sender<()>,
    ) -> Result<()> {
        let mut reader = BufReader::new(&*stream);
        let size_payload = T::get_size_of_payload();
        loop {
            client.lock().await;
            let mut vector_payload = vec![0u8; size_payload];
            let buffer_payload = vector_payload[..].as_mut();
            match reader.read_exact(buffer_payload).await {
                Ok(_) => {
                    info!("recv from server {:?}", buffer_payload);
                    let size_data = T::get_size_of_data(&buffer_payload);
                    let mut vector_data = vec![0u8; size_data];
                    let buffer_data = vector_data[..].as_mut();

                    reader.read_exact(buffer_data).await?;
                    vector_payload.extend_from_slice(&buffer_data);
                    let data = T::from_raw(&vector_payload);
                    sender_event.send(data).await?;
                }
                Err(e) => {
                    error!("error in client : {}", e);
                    break Ok(());
                }
            }
        }
    }

    async fn connection_writer_loop(
        mut messages: Receiver<T>,
        stream: Arc<TcpStream>,
        mut shutdown: Receiver<()>,
    ) -> Result<()> {
        let mut stream = &*stream;
        loop {
            select! {
                msg = messages.next().fuse() => match msg {
                    Some(ref data) =>{
                        info!("connection_writer_loop msg {:?}", msg);
                        stream.write_all(&data.serialize()[..]).await?
                    },
                    None => break,
                },
                void = shutdown.next().fuse() => match void {
                    Some(_) => {
                        info!("connection_writer_loop void");
                    },
                    None => {
                        info!("connection_writer_loop void toi");
                        break;
                    },
                }
            }
        }
        Ok(())
    }
}
