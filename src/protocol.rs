use quarxtor_core::net_core::{
    FrameKind, FrameHeader, Frame,
    HelloPayload, GetBlocksPayload, PushBlocksPayload,
    GetObjectPayload, PushObjectPayload,
    ProtocolVersion, NetError, NetResult,
};

/// -----------------------------
/// Transport Trait (абстракция)
/// -----------------------------
pub trait Transport {
    fn send(&mut self, data: &[u8]) -> NetResult<()>;
    fn recv_exact(&mut self, len: usize) -> NetResult<Vec<u8>>;
}

/// -----------------------------
/// Encoding / Decoding Utils
/// -----------------------------

fn encode_u16(x: u16) -> [u8; 2] { x.to_be_bytes() }
fn encode_u32(x: u32) -> [u8; 4] { x.to_be_bytes() }
fn encode_u64(x: u64) -> [u8; 8] { x.to_be_bytes() }

fn decode_u16(b: &[u8]) -> u16 {
    u16::from_be_bytes([b[0], b[1]])
}
fn decode_u32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}
fn decode_u64(b: &[u8]) -> u64 {
    u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
}

/// -----------------------------
/// FrameHeader encode/decode (свободные функции)
/// -----------------------------

fn encode_frame_header(h: &FrameHeader) -> Vec<u8> {
    let mut v = Vec::with_capacity(1 + 1 + 4);

    let kind_byte = match h.kind {
        FrameKind::Hello      => 1,
        FrameKind::Caps       => 2,
        FrameKind::GetBlocks  => 3,
        FrameKind::PushBlocks => 4,
        FrameKind::GetObject  => 5,
        FrameKind::PushObject => 6,
        FrameKind::Ping       => 7,
        FrameKind::Pong       => 8,
    };

    v.push(kind_byte);
    v.push(h.flags);
    v.extend_from_slice(&encode_u32(h.length));
    v
}

fn decode_frame_header(buf: &[u8]) -> NetResult<FrameHeader> {
    if buf.len() < 6 {
        return Err(NetError::InvalidFrame);
    }

    let kind = match buf[0] {
        1 => FrameKind::Hello,
        2 => FrameKind::Caps,
        3 => FrameKind::GetBlocks,
        4 => FrameKind::PushBlocks,
        5 => FrameKind::GetObject,
        6 => FrameKind::PushObject,
        7 => FrameKind::Ping,
        8 => FrameKind::Pong,
        _ => return Err(NetError::InvalidFrame),
    };

    Ok(FrameHeader {
        kind,
       flags: buf[1],
       length: decode_u32(&buf[2..6]),
    })
}

/// -----------------------------
/// Frame encode/decode (свободные функции)
/// -----------------------------

fn encode_frame(frame: &Frame) -> Vec<u8> {
    let mut res = Vec::new();
    res.extend_from_slice(&encode_frame_header(&frame.header));
    res.extend_from_slice(&frame.payload);
    res
}

fn decode_frame(header: FrameHeader, payload: Vec<u8>) -> NetResult<Frame> {
    if payload.len() != header.length as usize {
        return Err(NetError::InvalidFrame);
    }
    Ok(Frame { header, payload })
}

/// -----------------------------
/// Payload Encoders
/// -----------------------------

pub fn encode_hello(p: &HelloPayload) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&encode_u64(p.node));
    v.extend_from_slice(&encode_u16(p.version.major));
    v.extend_from_slice(&encode_u16(p.version.minor));
    v
}

pub fn decode_hello(b: &[u8]) -> NetResult<HelloPayload> {
    if b.len() < 12 {
        return Err(NetError::DecodeError);
    }
    Ok(HelloPayload {
        node: decode_u64(&b[0..8]),
       version: ProtocolVersion {
           major: decode_u16(&b[8..10]),
       minor: decode_u16(&b[10..12]),
       }
    })
}

pub fn encode_get_blocks(p: &GetBlocksPayload) -> Vec<u8> {
    let mut v = Vec::new();
    for id in &p.ids {
        v.extend_from_slice(&encode_u64(*id));
    }
    v
}

pub fn decode_get_blocks(b: &[u8]) -> NetResult<GetBlocksPayload> {
    if b.len() % 8 != 0 {
        return Err(NetError::DecodeError);
    }
    let mut ids = Vec::new();
    for chunk in b.chunks(8) {
        ids.push(decode_u64(chunk));
    }
    Ok(GetBlocksPayload { ids })
}

pub fn encode_push_blocks(p: &PushBlocksPayload) -> Vec<u8> {
    p.raw.clone()
}

pub fn decode_push_blocks(b: &[u8]) -> NetResult<PushBlocksPayload> {
    Ok(PushBlocksPayload { raw: b.to_vec() })
}

pub fn encode_get_object(p: &GetObjectPayload) -> Vec<u8> {
    encode_u64(p.id).to_vec()
}

pub fn decode_get_object(b: &[u8]) -> NetResult<GetObjectPayload> {
    if b.len() != 8 {
        return Err(NetError::DecodeError);
    }
    Ok(GetObjectPayload { id: decode_u64(b) })
}

pub fn encode_push_object(p: &PushObjectPayload) -> Vec<u8> {
    p.raw.clone()
}

pub fn decode_push_object(b: &[u8]) -> NetResult<PushObjectPayload> {
    Ok(PushObjectPayload { raw: b.to_vec() })
}

/// -----------------------------
/// Sending / Receiving Frames
/// -----------------------------
pub fn send_frame<T: Transport>(t: &mut T, frame: &Frame) -> NetResult<()> {
    let encoded = encode_frame(frame);
    t.send(&encoded)
}

pub fn recv_frame<T: Transport>(t: &mut T) -> NetResult<Frame> {
    // читаем заголовок (6 байт)
    let hdr_bytes = t.recv_exact(6)?;
    let header = decode_frame_header(&hdr_bytes)?;

    // читаем payload
    let payload = t.recv_exact(header.length as usize)?;

    decode_frame(header, payload)
}
