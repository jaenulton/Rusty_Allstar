//! RFC 5456 Constants and Enumerations for IAX2

/// IAX2 Frame Types (RFC 5456 Section 8.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    Dtmf = 1,
    Voice = 2,
    Video = 3,
    Control = 4,
    Null = 5,
    Iax = 6,
    Text = 7,
    Image = 8,
    Html = 9,
    ComfortNoise = 10,
    Unknown(u8),
}

impl From<u8> for FrameType {
    fn from(val: u8) -> Self {
        match val {
            1 => FrameType::Dtmf,
            2 => FrameType::Voice,
            3 => FrameType::Video,
            4 => FrameType::Control,
            5 => FrameType::Null,
            6 => FrameType::Iax,
            7 => FrameType::Text,
            8 => FrameType::Image,
            9 => FrameType::Html,
            10 => FrameType::ComfortNoise,
            other => FrameType::Unknown(other),
        }
    }
}

impl From<FrameType> for u8 {
    fn from(ft: FrameType) -> Self {
        match ft {
            FrameType::Dtmf => 1,
            FrameType::Voice => 2,
            FrameType::Video => 3,
            FrameType::Control => 4,
            FrameType::Null => 5,
            FrameType::Iax => 6,
            FrameType::Text => 7,
            FrameType::Image => 8,
            FrameType::Html => 9,
            FrameType::ComfortNoise => 10,
            FrameType::Unknown(val) => val,
        }
    }
}

/// IAX Control Subclasses (When FrameType == Iax, RFC 5456 Section 8.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IaxCommand {
    New = 1,
    Ping = 2,
    Pong = 3,
    Ack = 4,
    Hangup = 5,
    Reject = 6,
    Accept = 7,
    AuthReq = 8,
    AuthRep = 9,
    Inval = 10,
    LagRq = 11,
    LagRp = 12,
    RegReq = 13,
    RegAuth = 14,
    RegAck = 15,
    RegRej = 16,
    RegRel = 17,
    VNak = 18,
    DpReq = 19,
    DpRep = 20,
    Dial = 21,
    TxReq = 22,
    TxCnt = 23,
    TxAcc = 24,
    TxRej = 25,
    TxReady = 26,
    TxRel = 27,
    TxKey = 28,
    AppTime = 29,
    Poke = 30,
    Page = 31,
    Mwi = 32,
    Unsupported = 33,
    Transfer = 34,
    Unknown(u8),
}

impl From<u8> for IaxCommand {
    fn from(val: u8) -> Self {
        match val {
            1 => IaxCommand::New,
            2 => IaxCommand::Ping,
            3 => IaxCommand::Pong,
            4 => IaxCommand::Ack,
            5 => IaxCommand::Hangup,
            6 => IaxCommand::Reject,
            7 => IaxCommand::Accept,
            8 => IaxCommand::AuthReq,
            9 => IaxCommand::AuthRep,
            10 => IaxCommand::Inval,
            11 => IaxCommand::LagRq,
            12 => IaxCommand::LagRp,
            13 => IaxCommand::RegReq,
            14 => IaxCommand::RegAuth,
            15 => IaxCommand::RegAck,
            16 => IaxCommand::RegRej,
            17 => IaxCommand::RegRel,
            18 => IaxCommand::VNak,
            19 => IaxCommand::DpReq,
            20 => IaxCommand::DpRep,
            21 => IaxCommand::Dial,
            22 => IaxCommand::TxReq,
            23 => IaxCommand::TxCnt,
            24 => IaxCommand::TxAcc,
            25 => IaxCommand::TxRej,
            26 => IaxCommand::TxReady,
            27 => IaxCommand::TxRel,
            28 => IaxCommand::TxKey,
            29 => IaxCommand::AppTime,
            30 => IaxCommand::Poke,
            31 => IaxCommand::Page,
            32 => IaxCommand::Mwi,
            33 => IaxCommand::Unsupported,
            34 => IaxCommand::Transfer,
            other => IaxCommand::Unknown(other),
        }
    }
}

impl From<IaxCommand> for u8 {
    fn from(cmd: IaxCommand) -> Self {
        match cmd {
            IaxCommand::New => 1,
            IaxCommand::Ping => 2,
            IaxCommand::Pong => 3,
            IaxCommand::Ack => 4,
            IaxCommand::Hangup => 5,
            IaxCommand::Reject => 6,
            IaxCommand::Accept => 7,
            IaxCommand::AuthReq => 8,
            IaxCommand::AuthRep => 9,
            IaxCommand::Inval => 10,
            IaxCommand::LagRq => 11,
            IaxCommand::LagRp => 12,
            IaxCommand::RegReq => 13,
            IaxCommand::RegAuth => 14,
            IaxCommand::RegAck => 15,
            IaxCommand::RegRej => 16,
            IaxCommand::RegRel => 17,
            IaxCommand::VNak => 18,
            IaxCommand::DpReq => 19,
            IaxCommand::DpRep => 20,
            IaxCommand::Dial => 21,
            IaxCommand::TxReq => 22,
            IaxCommand::TxCnt => 23,
            IaxCommand::TxAcc => 24,
            IaxCommand::TxRej => 25,
            IaxCommand::TxReady => 26,
            IaxCommand::TxRel => 27,
            IaxCommand::TxKey => 28,
            IaxCommand::AppTime => 29,
            IaxCommand::Poke => 30,
            IaxCommand::Page => 31,
            IaxCommand::Mwi => 32,
            IaxCommand::Unsupported => 33,
            IaxCommand::Transfer => 34,
            IaxCommand::Unknown(val) => val,
        }
    }
}

/// Standard Audio Codec Formats (RFC 5456 Section 8.6.1)
pub mod audio_format {
    pub const G723_1: u32 = 1 << 0;
    pub const GSM: u32 = 1 << 1;
    pub const ULAW: u32 = 1 << 2; // G.711 μ-law (Standard AllStar default)
    pub const ALAW: u32 = 1 << 3; // G.711 A-law
    pub const G726: u32 = 1 << 4;
    pub const ADPCM: u32 = 1 << 5;
    pub const SLINEAR: u32 = 1 << 6;
    pub const LPC10: u32 = 1 << 7;
    pub const G729: u32 = 1 << 8;
    pub const SPEEX: u32 = 1 << 9;
    pub const ILBC: u32 = 1 << 10;
}
