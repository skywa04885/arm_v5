use std::marker::PhantomData;

use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::{
    error::Error,
    proto::{CommandCode, EventCode, Packet, Tag},
};

/// This struct is meant to write packets to a buffered reader.
pub(crate) struct PacketWriter<W>
where
    W: AsyncWrite + Unpin,
{
    _marker: PhantomData<W>,
}

impl<W> PacketWriter<W>
where
    W: AsyncWrite + Unpin,
{
    /// Write the given value to the given buffered writer.
    ///
    /// # Arguments
    ///
    /// * `buf_writer` - The buffered writer to write to.
    /// * `value` - The value to write.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the write operation is successful, otherwise returns an `Error`.
    pub(self) async fn write_value(
        buf_writer: &mut BufWriter<W>,
        value: &Vec<u8>,
    ) -> Result<(), Error> {
        buf_writer.write_u32(value.len() as u32).await?;
        buf_writer.write_all(value).await?;

        Ok(())
    }

    /// Write the given tag to the given buffered writer.
    ///
    /// # Arguments
    ///
    /// * `buf_writer` - The buffered writer to write to.
    /// * `tag` - The tag to write.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the write operation is successful, otherwise returns an `Error`.
    pub(self) async fn write_tag(buf_writer: &mut BufWriter<W>, tag: &Tag) -> Result<(), Error> {
        buf_writer.write_u64(tag.inner()).await?;

        Ok(())
    }

    /// Write the given event to the given buffered writer.
    ///
    /// # Arguments
    ///
    /// * `buf_writer` - The buffered writer to write to.
    /// * `event` - The event code to write.
    /// * `value` - The value to write.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the write operation is successful, otherwise returns an `Error`.
    pub(self) async fn write_event(
        buf_writer: &mut BufWriter<W>,
        event: &EventCode,
        value: &Vec<u8>,
    ) -> Result<(), Error> {
        buf_writer.write_u8(Packet::EVENT_IDENTIFIER).await?;
        buf_writer.write_u32(event.inner()).await?;

        Self::write_value(buf_writer, value).await?;

        buf_writer.flush().await?;

        Ok(())
    }

    /// Write the given command to the given buffered writer.
    ///
    /// # Arguments
    ///
    /// * `buf_writer` - The buffered writer to write to.
    /// * `command` - The command code to write.
    /// * `tag` - The tag to write.
    /// * `value` - The value to write.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the write operation is successful, otherwise returns an `Error`.
    pub(self) async fn write_command(
        buf_writer: &mut BufWriter<W>,
        command: &CommandCode,
        tag: &Tag,
        value: &Vec<u8>,
    ) -> Result<(), Error> {
        buf_writer.write_u8(Packet::COMMAND_IDENTIFIER).await?;
        buf_writer.write_u32(command.inner()).await?;

        Self::write_tag(buf_writer, tag).await?;
        Self::write_value(buf_writer, value).await?;

        buf_writer.flush().await?;

        Ok(())
    }

    /// Write the given reply to the given buffered writer.
    ///
    /// # Arguments
    ///
    /// * `buf_writer` - The buffered writer to write to.
    /// * `tag` - The tag to write.
    /// * `value` - The value to write.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the write operation is successful, otherwise returns an `Error`.
    pub(self) async fn write_reply(
        buf_writer: &mut BufWriter<W>,
        tag: &Tag,
        value: &Vec<u8>,
    ) -> Result<(), Error> {
        buf_writer.write_u8(Packet::REPLY_IDENTIFIER).await?;

        Self::write_tag(buf_writer, tag).await?;
        Self::write_value(buf_writer, value).await?;

        buf_writer.flush().await?;

        Ok(())
    }

    /// Write the given packet to the given buffered writer.
    ///
    /// # Arguments
    ///
    /// * `buf_writer` - The buffered writer to write to.
    /// * `packet` - The packet to write.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the write operation is successful, otherwise returns an `Error`.
    pub(crate) async fn write(buf_writer: &mut BufWriter<W>, packet: &Packet) -> Result<(), Error> {
        match packet {
            Packet::Event(event, value) => Self::write_event(buf_writer, event, value).await,
            Packet::Command(command, tag, value) => {
                Self::write_command(buf_writer, command, tag, value).await
            }
            Packet::Reply(tag, vec) => Self::write_reply(buf_writer, tag, vec).await,
        }
    }
}
