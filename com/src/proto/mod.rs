#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Event(u32);

impl Event {
    #[inline(always)]
    pub fn new(inner: u32) -> Self {
        Self(inner)
    }

    #[inline(always)]
    pub fn inner(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Command(u32);

impl Command {
    #[inline(always)]
    pub fn new(inner: u32) -> Self {
        Self(inner)
    }

    #[inline(always)]
    pub fn inner(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag(u64);

impl Tag {
    #[inline(always)]
    pub fn new(inner: u64) -> Self {
        Self(inner)
    }

    #[inline(always)]
    pub fn inner(&self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub enum Packet {
    Event(Event, Vec<u8>),
    Command(Command, Tag, Vec<u8>),
    Reply(Tag, Vec<u8>),
}

impl Packet {
    pub const EVENT_IDENTIFIER: u8 = 0x00_u8;
    pub const COMMAND_IDENTIFIER: u8 = 0x01_u8;
    pub const REPLY_IDENTIFIER: u8 = 0x02_u8;
}