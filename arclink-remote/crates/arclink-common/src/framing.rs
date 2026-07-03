use crate::errors::ProtocolError;
use crate::protocol::PROTOCOL_MAGIC;
use std::io::{Read, Write};

/// Encodes a message into a length-prefixed frame.
/// Structure: [Magic (4B)] [Payload Length (4B, Big-Endian)] [Serialized Payload]
pub fn encode_frame<T: serde::Serialize>(msg: &T) -> Result<Vec<u8>, ProtocolError> {
    let payload = bincode::serialize(msg)
        .map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;

    if payload.len() > 10 * 1024 * 1024 { // 10MB limit
        return Err(ProtocolError::MessageTooLarge);
    }

    let mut frame = Vec::with_capacity(8 + payload.len());
    frame.extend_from_slice(&PROTOCOL_MAGIC);
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Decodes a serialized frame payload into a type T.
pub fn decode_payload<T: serde::de::DeserializeOwned>(payload: &[u8]) -> Result<T, ProtocolError> {
    bincode::deserialize(payload)
        .map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
}

/// Synchronously reads a complete frame from a reader.
pub fn read_frame_sync<R: Read>(mut reader: R) -> Result<Vec<u8>, ProtocolError> {
    let mut header = [0u8; 8];
    reader.read_exact(&mut header).map_err(|_| ProtocolError::ConnectionReset)?;

    let magic = &header[0..4];
    if magic != PROTOCOL_MAGIC {
        return Err(ProtocolError::InvalidMagic);
    }

    let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    if len > 10 * 1024 * 1024 { // 10MB limit
        return Err(ProtocolError::MessageTooLarge);
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).map_err(|_| ProtocolError::ConnectionReset)?;

    Ok(payload)
}

/// Synchronously writes a complete message to a writer.
pub fn write_frame_sync<W: Write, T: serde::Serialize>(mut writer: W, msg: &T) -> Result<(), ProtocolError> {
    let frame = encode_frame(msg)?;
    writer.write_all(&frame).map_err(|_| ProtocolError::ConnectionReset)?;
    writer.flush().map_err(|_| ProtocolError::ConnectionReset)?;
    Ok(())
}
