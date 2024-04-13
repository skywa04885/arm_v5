use std::marker::PhantomData;

use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::{
    error::Error,
    proto::{Command, Event, Packet, Tag},
};

/// This struct is meant to read packets from a buffered reader.
pub(crate) struct PacketReader<R>
where
    R: AsyncRead + Unpin,
{
    _marker: PhantomData<R>,
}

impl<R> PacketReader<R>
where
    R: AsyncRead + Unpin,
{
    /// Read the value of a packet from the given buffered reader.
    pub(self) async fn read_value(buf_reader: &mut BufReader<R>) -> Result<Vec<u8>, Error> {
        // Read the length of the value.
        let len = buf_reader.read_u32().await?;

        // Allocate a new vector to contain the value and read it from the reader.
        let mut value = Vec::<u8>::with_capacity(len as usize);
        _ = buf_reader.read_exact(&mut value).await?;

        // Return the read value.
        Ok(value)
    }

    /// Read a tag from the given buffered reader.
    pub(self) async fn read_tag(buf_reader: &mut BufReader<R>) -> Result<Tag, Error> {
        Ok(Tag::new(buf_reader.read_u64().await?))
    }

    /// Read an event from the given buffered reader.
    pub(self) async fn read_event(buf_reader: &mut BufReader<R>) -> Result<Packet, Error> {
        let event = Event::new(buf_reader.read_u32().await?);
        let value = Self::read_value(buf_reader).await?;

        Ok(Packet::Event(event, value))
    }

    /// Read a command from the given buffered reader.
    pub(self) async fn read_command(buf_reader: &mut BufReader<R>) -> Result<Packet, Error> {
        let command = Command::new(buf_reader.read_u32().await?);
        let tag = Self::read_tag(buf_reader).await?;
        let value = Self::read_value(buf_reader).await?;

        Ok(Packet::Command(command, tag, value))
    }

    /// Read a reply from the given buffered reader.
    pub(self) async fn read_reply(buf_reader: &mut BufReader<R>) -> Result<Packet, Error> {
        let tag = Self::read_tag(buf_reader).await?;
        let value = Self::read_value(buf_reader).await?;

        Ok(Packet::Reply(tag, value))
    }

    /// Read a packet from the given buffered reader.
    pub(crate) async fn read(buf_reader: &mut BufReader<R>) -> Result<Packet, Error> {
        // Read the identifier so we know what packet we're dealing with.
        let identifier = buf_reader.read_u8().await?;

        // Call the read method belonging to the identifier.
        match identifier {
            Packet::EVENT_IDENTIFIER => Self::read_event(buf_reader).await,
            Packet::COMMAND_IDENTIFIER => Self::read_command(buf_reader).await,
            Packet::REPLY_IDENTIFIER => Self::read_reply(buf_reader).await,
            _ => Err(Error::Generic(
                format!("Invalid identifier: {}", identifier).into(),
            )),
        }
    }
}
