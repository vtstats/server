// Automatically generated rust module for 'continuation.proto' file

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
pub struct A {
    pub f1: i32,
}

impl<'a> MessageRead<'a> for A {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.f1 = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for A {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.f1) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_int32(*&self.f1))?;
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct B<'a> {
    pub video: Cow<'a, str>,
    pub f6: i32,
    pub a: continuation::A,
}

impl<'a> MessageRead<'a> for B<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(26) => msg.video = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(48) => msg.f6 = r.read_int32(bytes)?,
                Ok(130) => msg.a = r.read_message::<continuation::A>(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for B<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.video).len())
        + 1 + sizeof_varint(*(&self.f6) as u64)
        + 2 + sizeof_len((&self.a).get_size())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(26, |w| w.write_string(&**&self.video))?;
        w.write_with_tag(48, |w| w.write_int32(*&self.f6))?;
        w.write_with_tag(130, |w| w.write_message(&self.a))?;
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Continuation<'a> {
    pub b: continuation::B<'a>,
}

impl<'a> MessageRead<'a> for Continuation<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(957547474) => msg.b = r.read_message::<continuation::B>(bytes)?,
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
        + 5 + sizeof_len((&self.b).get_size())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(957547474, |w| w.write_message(&self.b))?;
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct C<'a> {
    pub channel_id: Cow<'a, str>,
    pub video_id: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for C<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.channel_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.video_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for C<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.channel_id).len())
        + 1 + sizeof_len((&self.video_id).len())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.channel_id))?;
        w.write_with_tag(18, |w| w.write_string(&**&self.video_id))?;
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct D<'a> {
    pub c: continuation::C<'a>,
}

impl<'a> MessageRead<'a> for D<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(42) => msg.c = r.read_message::<continuation::C>(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for D<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.c).get_size())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(42, |w| w.write_message(&self.c))?;
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct E<'a> {
    pub video_id: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for E<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.video_id = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for E<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.video_id).len())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.video_id))?;
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct F<'a> {
    pub e: continuation::E<'a>,
}

impl<'a> MessageRead<'a> for F<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(389502058) => msg.e = r.read_message::<continuation::E>(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for F<'a> {
    fn get_size(&self) -> usize {
        0
        + 5 + sizeof_len((&self.e).get_size())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(389502058, |w| w.write_message(&self.e))?;
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Video<'a> {
    pub d: continuation::D<'a>,
    pub f: continuation::F<'a>,
    pub s4: i64,
}

impl<'a> MessageRead<'a> for Video<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.d = r.read_message::<continuation::D>(bytes)?,
                Ok(26) => msg.f = r.read_message::<continuation::F>(bytes)?,
                Ok(32) => msg.s4 = r.read_int64(bytes)?,
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
        + 1 + sizeof_len((&self.d).get_size())
        + 1 + sizeof_len((&self.f).get_size())
        + 1 + sizeof_varint(*(&self.s4) as u64)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_message(&self.d))?;
        w.write_with_tag(26, |w| w.write_message(&self.f))?;
        w.write_with_tag(32, |w| w.write_int64(*&self.s4))?;
        Ok(())
    }
}

