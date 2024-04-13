use crate::{error::Error, net::PacketWriter, proto::Packet};

use tokio::{
    io::{AsyncWrite, BufWriter},
    select,
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

/// This struct represents the client transmitter.
pub(crate) struct Transmitter<W>
where
    W: AsyncWrite + Unpin,
{
    _marker: std::marker::PhantomData<W>,
}

impl<W> Transmitter<W>
where
    W: AsyncWrite + Unpin,
{
    /// The capacity of the instruction channel.
    pub(self) const INSTRUCTION_CHANNEL_CAPACITY: usize = 64_usize;

    /// Create a new transmitter with the given writer.
    pub(super) fn new(writer: W) -> (Worker<W>, Handle) {
        // Create the instruction channel.
        let (instruction_sender, instruction_receiver) =
            mpsc::channel(Self::INSTRUCTION_CHANNEL_CAPACITY);

        // Create the worker and handle.
        let handle = Handle::new(instruction_sender);
        let worker = Worker::new(instruction_receiver, writer);

        // Return the worker and handle.
        (worker, handle)
    }
}

/// This enum represents an instruction that can be sent to the worker.
pub(self) enum Instruction {
    WritePacket(Packet),
}

/// This struct represents the worker that will perform the transmitting.
pub(super) struct Worker<W>
where
    W: AsyncWrite + Unpin,
{
    instruction_receiver: mpsc::Receiver<Instruction>,
    buf_writer: BufWriter<W>,
}

impl<W> Worker<W>
where
    W: AsyncWrite + Unpin,
{
    /// Create a new worker.
    pub(self) fn new(instruction_receiver: mpsc::Receiver<Instruction>, writer: W) -> Self {
        Self {
            instruction_receiver,
            buf_writer: BufWriter::new(writer),
        }
    }

    /// Write the given packet to the buffered writer.
    pub(self) async fn write_packet(
        &mut self,
        packet: Packet,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Error> {
        select! {
            x = PacketWriter::write(&mut self.buf_writer, &packet) => x,
            _ = cancellation_token.cancelled() => Err(Error::Cancelled),
        }
    }

    /// Read an instruction from the instruction receiver.
    pub(self) async fn read_instruction_from_receiver(
        &mut self,
        cancellation_token: &CancellationToken,
    ) -> Result<Option<Instruction>, Error> {
        select! {
            x = self.instruction_receiver.recv() => Ok(x),
            _ = cancellation_token.cancelled() => Err(Error::Cancelled),
        }
    }

    /// Run the worker.
    pub(super) async fn run(&mut self, cancellation_token: CancellationToken) -> Result<(), Error> {
        // Keep reading instructions until the cancellation token is triggered.
        while let Some(instruction) = self
            .read_instruction_from_receiver(&cancellation_token)
            .await?
        {
            // Call the appropriate method based on the instruction.
            match instruction {
                Instruction::WritePacket(packet) => {
                    self.write_packet(packet, &cancellation_token).await?
                }
            }
        }

        Ok(())
    }
}

/// This struct represents handle to the worker.
#[derive(Clone)]
pub(super) struct Handle {
    instruction_sender: mpsc::Sender<Instruction>,
}

impl Handle {
    /// Create a new worker handle.
    pub(self) fn new(instruction_sender: mpsc::Sender<Instruction>) -> Self {
        Self { instruction_sender }
    }

    /// Send the given instruction to the worker.
    pub(self) async fn send_instruction(&self, instruction: Instruction) -> Result<(), Error> {
        // Send the instruction to the worker.
        self.instruction_sender
            .send(instruction)
            .await
            .map_err(|_| Error::Generic("Failed to send instruction to worker.".into()))?;

        // Return success.
        Ok(())
    }

    /// Send the write packet instruction to the worker.
    pub(crate) async fn write_packet(&self, packet: Packet) -> Result<(), Error> {
        // Create the instruction.
        let instruction = Instruction::WritePacket(packet);

        // Send the instruction to the worker.
        self.send_instruction(instruction).await?;

        // Return success.
        Ok(())
    }
}

