use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use tokio::{
    io::{AsyncRead, BufReader},
    select,
    sync::{mpsc, oneshot, RwLock},
};
use tokio_util::sync::CancellationToken;

use crate::{
    error::Error,
    net::PacketReader,
    proto::{EventCode, Packet, Tag},
};

/// This struct represents a subscriber id.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubscriberId(u64);

impl SubscriberId {
    /// Create a new subscriber id.
    #[inline(always)]
    pub fn new(inner: u64) -> Self {
        Self(inner)
    }

    /// Get the inner value of the subscriber id.
    #[inline(always)]
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// This struct represents the subscriber id generator.
#[derive(Clone)]
pub(self) struct SubscriberIdGenerator {
    counter: Arc<AtomicU64>,
}

impl SubscriberIdGenerator {
    /// Create a new subscriber id generator.
    pub(self) fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Generate a new subscriber id.
    pub(self) fn generate(&self) -> SubscriberId {
        SubscriberId::new(self.counter.fetch_add(1, Ordering::Relaxed))
    }
}

/// This struct represents the receiver.
pub(super) struct Receiver<R>
where
    R: AsyncRead + Unpin,
{
    _marker: std::marker::PhantomData<R>,
}

impl<R> Receiver<R>
where
    R: AsyncRead + Unpin,
{
    /// Create a new receiver for the given reader.
    pub(super) fn new(reader: R) -> (Worker<R>, Handle) {
        // Create the subscribers.
        let subscribers = Subscribers::new();

        // Create the worker and handle.
        let worker = Worker::new(reader, subscribers.clone());
        let handle = Handle::new(subscribers);

        // Return the worker and handle.
        (worker, handle)
    }
}

/// This enum represents a reply subscriber.
pub(self) enum ReplySubscriber {
    /// A channel that will receive the reply.
    Channel(oneshot::Sender<Vec<u8>>),
    /// A closure that will receive the reply.
    Closure(Box<dyn FnOnce(Vec<u8>) + Send + Sync + 'static>),
}

/// This enum represents an event subscriber.
pub(self) enum EventSubscriber {
    /// A channel that will receive the event.
    Channel(mpsc::Sender<Vec<u8>>),
    /// A closure that will receive the event.
    Closure(Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>),
}

/// This struct is a clonable representation of the subscribers.
#[derive(Clone)]
pub(crate) struct Subscribers {
    reply_subscribers: Arc<RwLock<HashMap<Tag, ReplySubscriber>>>,
    event_subscribers:
        Arc<RwLock<HashMap<EventCode, Arc<RwLock<Vec<(SubscriberId, EventSubscriber)>>>>>>,
    subscriber_id_generator: SubscriberIdGenerator,
}

impl Subscribers {
    /// Create a new subscribers.
    pub(self) fn new() -> Self {
        Self {
            reply_subscribers: Arc::new(RwLock::new(HashMap::new())),
            event_subscribers: Arc::new(RwLock::new(HashMap::new())),
            subscriber_id_generator: SubscriberIdGenerator::new(),
        }
    }

    /// Takes the reply subscriber that has the given tag.
    pub(self) async fn take_reply_subscriber_with_tag(&self, tag: Tag) -> Option<ReplySubscriber> {
        let mut reply_subscribers = self.reply_subscribers.write().await;
        reply_subscribers.remove(&tag)
    }

    /// Get the event subscribers that subscribed to the given event.
    pub(self) async fn get_event_subscribers_with_tag(
        &self,
        event: EventCode,
    ) -> Option<Arc<RwLock<Vec<(SubscriberId, EventSubscriber)>>>> {
        let event_subscribers = self.event_subscribers.read().await;
        event_subscribers.get(&event).map(|x| x.clone())
    }

    /// Subscribe to the event that has the given event.
    pub(self) async fn subscribe_to_event(
        &self,
        event: EventCode,
        subscriber: EventSubscriber,
    ) -> Result<SubscriberId, Error> {
        // Generate the subscriber id.
        let subscriber_id = self.subscriber_id_generator.generate();

        // Acquire the lock for the event subscribers.
        let mut event_subscribers = self.event_subscribers.write().await;

        // Get the list of subscribers for the given event.
        let mut subscribers = event_subscribers
            .entry(event)
            .or_insert_with(|| Arc::new(RwLock::new(Vec::new())))
            .write()
            .await;

        // Add the subscriber to the list of subscribers.
        subscribers.push((subscriber_id, subscriber));

        // Return the subscriber id.
        Ok(subscriber_id)
    }

    /// Unsubscribe the subscriber with the given id from the given event.
    pub(super) async fn unsubscribe_from_event(
        &self,
        event: EventCode,
        subscriber_id: SubscriberId,
    ) -> Result<(), Error> {
        // Acquire the lock for the event subscribers.
        let event_subscribers = self.event_subscribers.read().await;

        // Get all the subscribers of the event.
        if let Some(subscribers) = event_subscribers.get(&event).map(|x| x.clone()) {
            // Acquire a lock on the subscribers list.
            let mut subscribers = subscribers.write().await;

            // Get the initial length of the subscribers vector so we can determine if items were removed.
            let initial_len = subscribers.len();

            // Remove the subscriber that has the given id.
            subscribers.retain(|(x, _)| *x != subscriber_id);

            // Check if items were removed, if not, return an error.
            if initial_len == subscribers.len() {
                Err(Error::Generic(
                    format!(
                        "No subscriber with id {} found in subscriber vector for event {}",
                        subscriber_id.inner(),
                        event.inner()
                    )
                    .into(),
                ))
            } else {
                Ok(())
            }
        } else {
            Err(Error::Generic(
                format!("No subscriber vector found for event {}", event.inner()).into(),
            ))
        }
    }

    /// subscribe to the event using a newly created channel.
    pub(super) async fn subscribe_to_event_with_channel(
        &self,
        event: EventCode,
    ) -> Result<(SubscriberId, mpsc::Receiver<Vec<u8>>), Error> {
        // Create the channel.
        let (channel_sender, channel_receiver) = mpsc::channel(64_usize);

        // Subscribe to the event.
        let subscriber_id = self
            .subscribe_to_event(event, EventSubscriber::Channel(channel_sender))
            .await?;

        // Return the receiver.
        Ok((subscriber_id, channel_receiver))
    }

    /// Subscribe to the reply that has the given event using the given closure.
    pub(super) async fn subscribe_to_event_with_closure<F>(
        &self,
        event: EventCode,
        closure: F,
    ) -> Result<SubscriberId, Error>
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        // Subscribe to the event.
        let subscriber_id = self
            .subscribe_to_event(event, EventSubscriber::Closure(Box::new(closure)))
            .await?;

        // Return the subscriber id.
        Ok(subscriber_id)
    }

    /// Subscribe to the reply that has the given tag.
    pub(self) async fn subscribe_to_reply(
        &self,
        tag: Tag,
        subscriber: ReplySubscriber,
    ) -> Result<(), Error> {
        // Insert the channel into the reply subscribers.
        let mut reply_subscribers = self.reply_subscribers.write().await;
        reply_subscribers.entry(tag).or_insert(subscriber);

        // Return success.
        Ok(())
    }

    /// Subscribe to the reply using a newly created channel.
    pub(super) async fn subscribe_to_reply_with_channel(
        &self,
        tag: Tag,
    ) -> Result<oneshot::Receiver<Vec<u8>>, Error> {
        // Create the channel.
        let (channel_sender, channel_receiver) = oneshot::channel();

        // Subscribe.
        self.subscribe_to_reply(tag, ReplySubscriber::Channel(channel_sender))
            .await?;

        // Return the receiver.
        Ok(channel_receiver)
    }

    /// Subscribe to the reply that has the given tag using the given closure.
    pub(super) async fn subscribe_to_reply_with_closure<F>(
        &self,
        tag: Tag,
        closure: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(Vec<u8>) + Send + Sync + 'static,
    {
        // Subscribe.
        self.subscribe_to_reply(tag, ReplySubscriber::Closure(Box::new(closure)))
            .await?;

        // Return the receiver.
        Ok(())
    }

    /// Unsubscribe from the reply with the given tag.
    pub(super) async fn unsubscribe_from_reply(&self, tag: Tag) -> Result<(), Error> {
        // Acquire a write lock to the write subscribers.
        let mut reply_subscribers = self.reply_subscribers.write().await;

        // Remove the subscriber, and return either success or error depending on if
        //  it was removed.
        if let Some(_) = reply_subscribers.remove(&tag) {
            Err(Error::Generic(
                format!("Could not find reply subscriber for tag: {}", tag.inner()).into(),
            ))
        } else {
            Ok(())
        }
    }
}

pub(super) struct Worker<R>
where
    R: AsyncRead + Unpin,
{
    buf_reader: BufReader<R>,
    subscribers: Subscribers,
}

impl<R> Worker<R>
where
    R: AsyncRead + Unpin,
{
    /// Create a new worker.
    pub(self) fn new(reader: R, subscribers: Subscribers) -> Self {
        Self {
            buf_reader: BufReader::new(reader),
            subscribers,
        }
    }

    /// Handle the given event.
    pub(self) async fn handle_event(&mut self, event: EventCode, value: Vec<u8>) -> Result<(), Error> {
        if let Some(subscribers) = self.subscribers.get_event_subscribers_with_tag(event).await {
            // Acquire the lock for the subscribers.
            let subscribers = subscribers.read().await;

            // Iterate over the subscribers and send the event to them.
            for subscriber in subscribers.iter() {
                // Match the subscriber.
                match subscriber {
                    // Send the event to the channel if it is not closed.
                    (_, EventSubscriber::Channel(sender)) if !sender.is_closed() => {
                        _ = sender.send(value.clone()).await;
                    }
                    // Call the closure with the event.
                    (_, EventSubscriber::Closure(closure)) => {
                        closure(value.clone());
                    }
                    // Do nothing if the channel is closed.
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Handle the given reply.
    pub(self) async fn handle_reply(&mut self, tag: Tag, value: Vec<u8>) -> Result<(), Error> {
        // Take the reply subscriber with the given tag.
        if let Some(subscriber) = self.subscribers.take_reply_subscriber_with_tag(tag).await {
            // Match the subscriber.
            match subscriber {
                // Send the value to the channel if it is not closed.
                ReplySubscriber::Channel(sender) if !sender.is_closed() => {
                    _ = sender.send(value);
                }
                // Call the closure with the value.
                ReplySubscriber::Closure(closure) => closure(value),
                // Do nothing if the channel is closed.
                _ => {}
            }
        }

        Ok(())
    }

    /// Read a packet from the buffered reader.
    pub(self) async fn read_packet(
        &mut self,
        cancellation_token: &CancellationToken,
    ) -> Result<Packet, Error> {
        select! {
            x = PacketReader::read(&mut self.buf_reader) => x,
            _ = cancellation_token.cancelled() => Err(Error::Cancelled),
        }
    }

    /// Run the worker.
    pub(super) async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Error> {
        loop {
            // Read the packet from the buffered reader.
            let packet = self.read_packet(&cancellation_token).await?;

            // Call the appropriate handler for the packet.
            match packet {
                // Handle the event.
                Packet::Event(event, value) => self.handle_event(event, value).await?,
                // Handle the reply.
                Packet::Reply(tag, value) => self.handle_reply(tag, value).await?,
                // Return an error if a command packet is received.
                _ => {
                    return Err(Error::Generic(
                        "Received command packet, which is not allowed for a client.".into(),
                    ))
                }
            }
        }
    }
}

/// This struct represents handle to the worker.
pub(super) struct Handle {
    subscribers: Subscribers,
}

impl Handle {
    /// Create a new handle.
    pub fn new(subscribers: Subscribers) -> Self {
        Self { subscribers }
    }

    /// Get the subscribers.
    pub fn subscribers(&self) -> &Subscribers {
        &self.subscribers
    }
}
