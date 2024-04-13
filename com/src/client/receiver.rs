use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{AsyncRead, BufReader},
    sync::{oneshot, RwLock},
};

use crate::proto::Tag;

pub(super) struct Receiver<R>
where
    R: AsyncRead + Unpin,
{
    _marker: std::marker::PhantomData<R>,
}

/// This enum represents a reply subscriber.
pub(self) enum ReplySubscriber {
    /// A channel that will receive the reply.
    Channel(oneshot::Sender<Vec<u8>>),
    /// A closure that will receive the reply.
    Closure(Box<dyn FnOnce(Vec<u8>)>),
}

pub(self) struct Subscribers {
    reply_subscribers: Arc<RwLock<HashMap<Tag, ReplySubscriber>>>,
}

impl Subscribers {
    /// subscribe to the reply that has the given tag using a newly created channel.
    pub(crate) async fn subscribe_to_reply_with_channel(
        &self,
        tag: Tag,
    ) -> oneshot::Receiver<Vec<u8>> {
        // Create the channel.
        let (channel_sender, channel_receiver) = oneshot::channel();

        // Insert the channel into the reply subscribers.
        let mut reply_subscribers = self.reply_subscribers.write().await;
        reply_subscribers
            .entry(tag)
            .or_insert_with(|| ReplySubscriber::Channel(channel_sender));

        // Return the receiver.
        channel_receiver
    }

    /// Subscribe to the reply that has the given tag using the given closure.
    pub(crate) async fn subscribe_to_reply_with_closure<F>(&self, tag: Tag, closure: F)
    where
        F: FnOnce(Vec<u8>) + 'static,
    {
        // Insert the closure into the reply subscribers.
        let mut reply_subscribers = self.reply_subscribers.write().await;
        reply_subscribers
            .entry(tag)
            .or_insert_with(|| ReplySubscriber::Closure(Box::new(closure)));
    }
}

pub(super) struct Worker<R>
where
    R: AsyncRead + Unpin,
{
    buf_reader: BufReader<R>,
}

impl<R> Worker<R>
where
    R: AsyncRead + Unpin,
{
    /// Create a new worker.
    pub fn new(reader: R) -> Self {
        Self {
            buf_reader: BufReader::new(reader),
        }
    }
}

pub(super) struct Handle {}
