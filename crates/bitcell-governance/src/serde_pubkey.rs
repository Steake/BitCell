//! Custom serialization for public keys

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serialize a 33-byte public key
pub fn serialize<S>(pubkey: &[u8; 33], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if serializer.is_human_readable() {
        hex::encode(pubkey).serialize(serializer)
    } else {
        serializer.serialize_bytes(pubkey)
    }
}

/// Deserialize a 33-byte public key
pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 33], D::Error>
where
    D: Deserializer<'de>,
{
    if deserializer.is_human_readable() {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 33 {
            return Err(serde::de::Error::custom("invalid public key length"));
        }
        let mut array = [0u8; 33];
        array.copy_from_slice(&bytes);
        Ok(array)
    } else {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        if bytes.len() != 33 {
            return Err(serde::de::Error::custom("invalid public key length"));
        }
        let mut array = [0u8; 33];
        array.copy_from_slice(&bytes);
        Ok(array)
    }
}
