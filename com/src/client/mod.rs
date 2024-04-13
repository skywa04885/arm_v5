use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    join,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream, ToSocketAddrs,
    },
    select,
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

use crate::{
    error::Error,
    proto::{Command, Event, Packet, Tag},
};

use self::receiver::SubscriberId;

pub mod receiver;
pub mod transmitter;

/// This struct represents the tag generator.
pub(self) struct TagGenerator {
    counter: Arc<AtomicU64>,
}

impl TagGenerator {
    /// Create a new tag generator.
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Generate a new tag.
    pub fn generate(&self) -> Tag {
        Tag::new(self.counter.fetch_add(1_u64, Ordering::Relaxed))
    }
}

/// This struct represents the client.
pub struct Client;

impl Client {
    /// Connect to the given address.
    pub async fn connect<A>(
        addr: A,
    ) -> Result<(Handle, Worker<OwnedReadHalf, OwnedWriteHalf>), Error>
    where
        A: ToSocketAddrs,
    {
        // Connect to the given address.
        let stream = TcpStream::connect(addr).await?;

        // Split the stream into the reader and writer.
        let (reader, writer) = stream.into_split();

        // Create the transmitter and receiver.
        let (transmitter_worker, transmitter_handle) = transmitter::Transmitter::new(writer);
        let (receiver_worker, receiver_handle) = receiver::Receiver::new(reader);

        // Create the worker and the handle.
        let worker = Worker::new(receiver_worker, transmitter_worker);
        let handle = Handle::new(transmitter_handle, receiver_handle);

        // Return the handle and the worker.
        Ok((handle, worker))
    }
}

/// This struct represents the client worker.
pub struct Worker<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    receiver_worker: receiver::Worker<R>,
    transmitter_worker: transmitter::Worker<W>,
}

impl<R, W> Worker<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// Create a new worker.
    pub(self) fn new(
        receiver_worker: receiver::Worker<R>,
        transmitter_worker: transmitter::Worker<W>,
    ) -> Self {
        Self {
            receiver_worker,
            transmitter_worker,
        }
    }

    /// Run the worker.
    pub async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Error> {
        // Run the receiver and transmitter workers, exiting when one of them exits.
        select!(
            x = self.receiver_worker.run(cancellation_token.clone()) => x,
            x = self.transmitter_worker.run(cancellation_token) => x
        )
    }
}

pub struct Handle {
    tag_generator: TagGenerator,
    transmitter_handle: transmitter::Handle,
    receiver_handle: receiver::Handle,
}

impl Handle {
    /// Create a new client.
    pub(self) fn new(transmitter_handle: transmitter::Handle, receiver_handle: receiver::Handle) -> Self {
        Self {
            tag_generator: TagGenerator::new(),
            transmitter_handle,
            receiver_handle,
        }
    }

    /// Write the given command and call the given closure when the reply is received.
    pub async fn write_command_closure<F>(
        &self,
        command: Command,
        value: Vec<u8>,
        closure: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(Vec<u8>) + Send + Sync + 'static,
    {
        // Generate the tag of the command and create the packet.
        let tag = self.tag_generator.generate();
        let packet = Packet::Command(command, tag, value);

        // Subscribe to the reply.
        self.receiver_handle
            .subscribers()
            .subscribe_to_reply_with_closure(tag, closure)
            .await;

        // Write the packet to the transmitter.
        self.transmitter_handle.write_packet(packet).await?;

        // Return success.
        Ok(())
    }

    /// Write the given command and return a receiver that will receive the reply.
    pub async fn write_command_channel(
        &self,
        command: Command,
        value: Vec<u8>,
    ) -> Result<tokio::sync::oneshot::Receiver<Vec<u8>>, Error> {
        // Generate the tag of the command and create the packet.
        let tag = self.tag_generator.generate();
        let packet = Packet::Command(command, tag, value);

        // Subscribe to the reply.
        let receiver = self
            .receiver_handle
            .subscribers()
            .subscribe_to_reply_with_channel(tag)
            .await;

        // Write the packet to the transmitter.
        self.transmitter_handle.write_packet(packet).await?;

        // Return the receiver.
        Ok(receiver)
    }

    /// Subscribe to the given event in a way that the closure gets called when it's sent.
    pub async fn subscribe_to_event_with_closure<F>(
        &self,
        event: Event,
        closure: F,
    ) -> Result<SubscriberId, Error>
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        self.receiver_handle
            .subscribers()
            .subscribe_to_event_with_closure(event, closure)
            .await
    }

    /// Subscribe to the given event and return a receiver that will receive the events.
    pub async fn subscribe_to_event_with_channel(
        &self,
        event: Event,
    ) -> Result<(SubscriberId, mpsc::Receiver<Vec<u8>>), Error> {
        // Subscribe to the event.
        self.receiver_handle
            .subscribers()
            .subscribe_to_event_with_channel(event)
            .await
    }
}
