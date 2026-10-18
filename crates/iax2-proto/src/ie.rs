//! Information Elements (IE) according to RFC 5456 Section 8.6

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IeType {
    CalledNumber = 1,
    CallingNumber = 2,
    CallingAni = 3,
    CallingName = 4,
    CalledContext = 5,
    Username = 6,
    Password = 7,
    Capability = 8,
    Format = 9,
    Language = 10,
    Version = 11,
    Adsicpe = 12,
    Dnid = 13,
    Authmeth = 14,
    Challenge = 15,
    Md5Result = 16,
    ApparentAddr = 18,
    Refresh = 19,
    DpStatus = 20,
    Cause = 21,
    IaxUnknown = 22,
    MsgCount = 23,
    AutoAnswer = 24,
    Musiconhold = 25,
    Unknown(u8),
}

impl From<u8> for IeType {
    fn from(val: u8) -> Self {
        match val {
            1 => IeType::CalledNumber,
            2 => IeType::CallingNumber,
            3 => IeType::CallingAni,
            4 => IeType::CallingName,
            5 => IeType::CalledContext,
            6 => IeType::Username,
            7 => IeType::Password,
            8 => IeType::Capability,
            9 => IeType::Format,
            10 => IeType::Language,
            11 => IeType::Version,
            12 => IeType::Adsicpe,
            13 => IeType::Dnid,
            14 => IeType::Authmeth,
            15 => IeType::Challenge,
            16 => IeType::Md5Result,
            18 => IeType::ApparentAddr,
            19 => IeType::Refresh,
            20 => IeType::DpStatus,
            21 => IeType::Cause,
            22 => IeType::IaxUnknown,
            23 => IeType::MsgCount,
            24 => IeType::AutoAnswer,
            25 => IeType::Musiconhold,
            other => IeType::Unknown(other),
        }
    }
}

impl From<IeType> for u8 {
    fn from(ie: IeType) -> Self {
        match ie {
            IeType::CalledNumber => 1,
            IeType::CallingNumber => 2,
            IeType::CallingAni => 3,
            IeType::CallingName => 4,
            IeType::CalledContext => 5,
            IeType::Username => 6,
            IeType::Password => 7,
            IeType::Capability => 8,
            IeType::Format => 9,
            IeType::Language => 10,
            IeType::Version => 11,
            IeType::Adsicpe => 12,
            IeType::Dnid => 13,
            IeType::Authmeth => 14,
            IeType::Challenge => 15,
            IeType::Md5Result => 16,
            IeType::ApparentAddr => 18,
            IeType::Refresh => 19,
            IeType::DpStatus => 20,
            IeType::Cause => 21,
            IeType::IaxUnknown => 22,
            IeType::MsgCount => 23,
            IeType::AutoAnswer => 24,
            IeType::Musiconhold => 25,
            IeType::Unknown(val) => val,
        }
    }
}

/// A borrowed view of a single Information Element from an IAX2 packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InformationElement<'a> {
    pub ie_type: IeType,
    pub data: &'a [u8],
}

impl<'a> InformationElement<'a> {
    pub fn new(ie_type: IeType, data: &'a [u8]) -> Self {
        Self { ie_type, data }
    }

    /// Read an IE data slice as a UTF-8 string if valid
    pub fn as_str(&self) -> Option<&'a str> {
        core::str::from_utf8(self.data).ok()
    }

    /// Read as big-endian u32 (used by Format, Capability, etc.)
    pub fn as_u32(&self) -> Option<u32> {
        if self.data.len() == 4 {
            Some(u32::from_be_bytes([
                self.data[0],
                self.data[1],
                self.data[2],
                self.data[3],
            ]))
        } else {
            None
        }
    }

    /// Read as big-endian u16 (used by Refresh, Version, etc.)
    pub fn as_u16(&self) -> Option<u16> {
        if self.data.len() == 2 {
            Some(u16::from_be_bytes([self.data[0], self.data[1]]))
        } else {
            None
        }
    }
}

/// Iterator over Information Elements in a byte slice
pub struct IeIterator<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> IeIterator<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, offset: 0 }
    }
}

impl<'a> Iterator for IeIterator<'a> {
    type Item = InformationElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + 2 > self.buffer.len() {
            return None;
        }

        let ie_type = IeType::from(self.buffer[self.offset]);
        let data_len = self.buffer[self.offset + 1] as usize;
        let start = self.offset + 2;
        let end = start + data_len;

        if end > self.buffer.len() {
            // Malformed/truncated IE
            return None;
        }

        self.offset = end;
        Some(InformationElement {
            ie_type,
            data: &self.buffer[start..end],
        })
    }
}
