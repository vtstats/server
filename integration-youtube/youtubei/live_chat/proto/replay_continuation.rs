// Automatically generated rust module for 'replay_continuation.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Continuation<'a> {
    pub f156074452: Option<replay_continuation::ContinuationA<'a>>,
}

impl<'a> MessageRead<'a> for Continuation<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(1248595618) => msg.f156074452 = Some(r.read_message::<replay_continuation::ContinuationA>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Continuation<'a> {
    fn get_size(&self) -> usize {
        0
        + self.f156074452.as_ref().map_or(0, |m| 5 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.f156074452 { w.write_with_tag(1248595618, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ContinuationA<'a> {
    pub f3: Cow<'a, str>,
    pub f8: i32,
    pub f10: Option<replay_continuation::ContinuationB>,
    pub f14: Option<replay_continuation::ContinuationC>,
}

impl<'a> MessageRead<'a> for ContinuationA<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(26) => msg.f3 = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(64) => msg.f8 = r.read_int32(bytes)?,
                Ok(82) => msg.f10 = Some(r.read_message::<replay_continuation::ContinuationB>(bytes)?),
                Ok(114) => msg.f14 = Some(r.read_message::<replay_continuation::ContinuationC>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ContinuationA<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.f3 == "" { 0 } else { 1 + sizeof_len((&self.f3).len()) }
        + if self.f8 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.f8) as u64) }
        + self.f10.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.f14.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.f3 != "" { w.write_with_tag(26, |w| w.write_string(&**&self.f3))?; }
        if self.f8 != 0i32 { w.write_with_tag(64, |w| w.write_int32(*&self.f8))?; }
        if let Some(ref s) = self.f10 { w.write_with_tag(82, |w| w.write_message(s))?; }
        if let Some(ref s) = self.f14 { w.write_with_tag(114, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ContinuationB {
    pub f4: i32,
    pub f22: i32,
    pub f31: i32,
}

impl<'a> MessageRead<'a> for ContinuationB {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(32) => msg.f4 = r.read_int32(bytes)?,
                Ok(176) => msg.f22 = r.read_int32(bytes)?,
                Ok(248) => msg.f31 = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ContinuationB {
    fn get_size(&self) -> usize {
        0
        + if self.f4 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.f4) as u64) }
        + if self.f22 == 0i32 { 0 } else { 2 + sizeof_varint(*(&self.f22) as u64) }
        + if self.f31 == 0i32 { 0 } else { 2 + sizeof_varint(*(&self.f31) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.f4 != 0i32 { w.write_with_tag(32, |w| w.write_int32(*&self.f4))?; }
        if self.f22 != 0i32 { w.write_with_tag(176, |w| w.write_int32(*&self.f22))?; }
        if self.f31 != 0i32 { w.write_with_tag(248, |w| w.write_int32(*&self.f31))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ContinuationC {
    pub f1: i32,
    pub f3: i32,
    pub f4: i32,
}

impl<'a> MessageRead<'a> for ContinuationC {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.f1 = r.read_int32(bytes)?,
                Ok(24) => msg.f3 = r.read_int32(bytes)?,
                Ok(32) => msg.f4 = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ContinuationC {
    fn get_size(&self) -> usize {
        0
        + if self.f1 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.f1) as u64) }
        + if self.f3 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.f3) as u64) }
        + if self.f4 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.f4) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.f1 != 0i32 { w.write_with_tag(8, |w| w.write_int32(*&self.f1))?; }
        if self.f3 != 0i32 { w.write_with_tag(24, |w| w.write_int32(*&self.f3))?; }
        if self.f4 != 0i32 { w.write_with_tag(32, |w| w.write_int32(*&self.f4))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Video<'a> {
    pub f1: Option<replay_continuation::VideoA<'a>>,
    pub f3: Option<replay_continuation::VideoC<'a>>,
    pub f4: i32,
    pub f6: i32,
}

impl<'a> MessageRead<'a> for Video<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.f1 = Some(r.read_message::<replay_continuation::VideoA>(bytes)?),
                Ok(26) => msg.f3 = Some(r.read_message::<replay_continuation::VideoC>(bytes)?),
                Ok(32) => msg.f4 = r.read_int32(bytes)?,
                Ok(48) => msg.f6 = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Video<'a> {
    fn get_size(&self) -> usize {
        0
        + self.f1.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.f3.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + if self.f4 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.f4) as u64) }
        + if self.f6 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.f6) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.f1 { w.write_with_tag(10, |w| w.write_message(s))?; }
        if let Some(ref s) = self.f3 { w.write_with_tag(26, |w| w.write_message(s))?; }
        if self.f4 != 0i32 { w.write_with_tag(32, |w| w.write_int32(*&self.f4))?; }
        if self.f6 != 0i32 { w.write_with_tag(48, |w| w.write_int32(*&self.f6))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct VideoA<'a> {
    pub f5: Option<replay_continuation::VideoB<'a>>,
}

impl<'a> MessageRead<'a> for VideoA<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(42) => msg.f5 = Some(r.read_message::<replay_continuation::VideoB>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for VideoA<'a> {
    fn get_size(&self) -> usize {
        0
        + self.f5.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.f5 { w.write_with_tag(42, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct VideoB<'a> {
    pub f1: Cow<'a, str>,
    pub f2: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for VideoB<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.f1 = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.f2 = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for VideoB<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.f1 == "" { 0 } else { 1 + sizeof_len((&self.f1).len()) }
        + if self.f2 == "" { 0 } else { 1 + sizeof_len((&self.f2).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.f1 != "" { w.write_with_tag(10, |w| w.write_string(&**&self.f1))?; }
        if self.f2 != "" { w.write_with_tag(18, |w| w.write_string(&**&self.f2))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct VideoC<'a> {
    pub f48687757: Option<replay_continuation::VideoD<'a>>,
}

impl<'a> MessageRead<'a> for VideoC<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(389502058) => msg.f48687757 = Some(r.read_message::<replay_continuation::VideoD>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for VideoC<'a> {
    fn get_size(&self) -> usize {
        0
        + self.f48687757.as_ref().map_or(0, |m| 5 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.f48687757 { w.write_with_tag(389502058, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct VideoD<'a> {
    pub f1: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for VideoD<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.f1 = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for VideoD<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.f1 == "" { 0 } else { 1 + sizeof_len((&self.f1).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.f1 != "" { w.write_with_tag(10, |w| w.write_string(&**&self.f1))?; }
        Ok(())
    }
}

