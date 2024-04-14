use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream, ToSocketAddrs,
    },
    select,
    sync::{mpsc, oneshot},
};
use tokio_util::sync::CancellationToken;

use crate::{
    error::Error,
    proto::{CommandCode, EventCode, Packet, Tag},
};

use self::receiver::SubscriberId;

pub mod receiver;
pub mod transmitter;

/// This trait means that the thing implementing it is a command.
pub trait Command: Serialize {
    /// Get the command code.
    fn code(&self) -> CommandCode;
}

/// This trait means that the thing implemting it is a reply.
pub trait Reply: DeserializeOwned {}

/// This trait means that the thing implementing it is an event.
pub trait Event: DeserializeOwned {
    /// Get the event code.
    fn code(&self) -> EventCode;
}

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
    pub(self) fn new(
        transmitter_handle: transmitter::Handle,
        receiver_handle: receiver::Handle,
    ) -> Self {
        Self {
            tag_generator: TagGenerator::new(),
            transmitter_handle,
            receiver_handle,
        }
    }

    pub async fn write_serializable_command_with_cancellation<C, R>(
        &self,
        command: C,
        cancellation_token: &CancellationToken,
    ) -> Result<R, Error>
    where
        C: Command,
        R: Reply,
    {
        select! {
            result = self.write_serializable_command::<C, R>(command) => result,
            _ = cancellation_token.cancelled() => Err(Error::Cancelled),
        }
    }

    pub async fn write_serializable_command<C, R>(&self, command: C) -> Result<R, Error>
    where
        C: Command,
        R: Reply,
    {
        let code = command.code();
        let value = rmp_serde::to_vec(&command).map_err(|_| Error::SerdeSerError)?;

        let vec = self
            .write_command_reply_to_channel(code, value)
            .await?
            .await
            .map_err(|_| Error::Generic("Failed to receive reply.".into()))?;

        rmp_serde::from_slice(&vec).map_err(|_| Error::DeserializeError)
    }

    /// Write the given serializable command and reply to the given closure.
    pub async fn write_serializable_command_reply_to_closure<S, F, R>(
        &self,
        command: S,
        closure: F,
    ) -> Result<(), Error>
    where
        S: Command,
        F: FnOnce(Result<R, Error>) + Send + Sync + 'static,
        R: Reply,
    {
        // Get the command code.
        let code = command.code();

        // Serialize the command to a byte vector.
        let value = rmp_serde::to_vec(&command).map_err(|_| Error::SerdeSerError)?;

        // Write the serialized command and return it's result.
        self.write_command_reply_to_closure(code, value, move |x| {
            // Decode the received reply and call the closure with either the error or the result.
            closure(rmp_serde::from_slice(&x).map_err(|_| Error::DeserializeError))
        })
        .await
    }

    /// Write the given command and call the given closure when the reply is received.
    pub async fn write_command_reply_to_closure<F>(
        &self,
        code: CommandCode,
        value: Vec<u8>,
        closure: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(Vec<u8>) + Send + Sync + 'static,
    {
        // Generate the tag of the command and create the packet.
        let tag = self.tag_generator.generate();
        let packet = Packet::Command(code, tag, value);

        // Subscribe to the reply.
        self.receiver_handle
            .subscribers()
            .subscribe_to_reply_with_closure(tag, closure)
            .await?;

        // Write the packet to the transmitter.
        self.transmitter_handle.write_packet(packet).await?;

        // Return success.
        Ok(())
    }

    /// Write the given command and return a receiver that will receive the reply.
    pub async fn write_command_reply_to_channel(
        &self,
        code: CommandCode,
        value: Vec<u8>,
    ) -> Result<oneshot::Receiver<Vec<u8>>, Error> {
        // Generate the tag of the command and create the packet.
        let tag = self.tag_generator.generate();
        let packet = Packet::Command(code, tag, value);

        // Subscribe to the reply.
        let receiver = self
            .receiver_handle
            .subscribers()
            .subscribe_to_reply_with_channel(tag)
            .await?;

        // Write the packet to the transmitter.
        self.transmitter_handle.write_packet(packet).await?;

        // Return the receiver.
        Ok(receiver)
    }

    /// Subscribe to the given event in a way that the closure gets called when it's sent.
    pub async fn subscribe_to_event_with_closure<F, E>(
        &self,
        code: EventCode,
        closure: F,
    ) -> Result<SubscriberId, Error>
    where
        F: Fn(Result<E, Error>) + Send + Sync + 'static,
        E: Event,
    {
        self.receiver_handle
            .subscribers()
            .subscribe_to_event_with_closure(code, move |x| {
                closure(rmp_serde::from_slice(&x).map_err(|_| Error::DeserializeError))
            })
            .await
    }

    /// Unsubscribe the subscriber that has the given id from the given event.
    pub async fn unsubscribe_from_event(
        &self,
        code: EventCode,
        subscriber_id: SubscriberId,
    ) -> Result<(), Error> {
        self.receiver_handle
            .subscribers()
            .unsubscribe_from_event(code, subscriber_id)
            .await
    }
}
