use std::io;

use crate::wire::*;


/// A **OPT** _(options)_ pseudo-record, which is used to extend the DNS
/// protocol with additional flags such as DNSSEC stuff.
///
/// # Pseudo-record?
///
/// Unlike all the other record types, which are used to return data about a
/// domain name, the OPT record type is used to add more options to the
/// request, including data about the client or the server. It can exist, with
/// a payload, as a query or a response, though it’s usually encountered in
/// the Additional section. Its purpose is to add more room to the DNS wire
/// format, as backwards compatibility makes it impossible to simply add more
/// flags to the header.
///
/// The fact that this isn’t a standard record type is annoying for a DNS
/// implementation. It re-purposes the ‘class’ and ‘TTL’ fields of the
/// `Answer` struct, as they only have meaning when associated with a domain
/// name. This means that the parser has to treat the OPT type specially,
/// switching to `Opt::read` as soon as the rtype is detected. It also means
/// the output has to deal with missing classes and TTLs.
///
/// # References
///
/// - [RFC 6891](https://tools.ietf.org/html/rfc6891) — Extension Mechanisms for DNS (April 2013)
#[derive(PartialEq, Debug, Clone)]
pub struct OPT {

    /// The maximum size of a UDP packet that the client supports.
    pub udp_payload_size: u16,

    /// The bits that form an extended rcode when non-zero.
    pub higher_bits: u8,

    /// The version number of the DNS extension mechanism.
    pub edns0_version: u8,

    /// Sixteen bits worth of flags.
    pub flags: u16,

    /// The payload of the OPT record.
    pub data: Vec<u8>,
}

impl OPT {

    /// The record type number associated with OPT.
    pub const RR_TYPE: u16 = 41;

    /// Reads from the given cursor to parse an OPT record.
    ///
    /// The buffer will have slightly more bytes to read for an OPT record
    /// than for a typical one: we will not have encountered the ‘class’ or
    /// ‘ttl’ fields, which have different meanings for this record type.
    /// See §6.1.3 of the RFC, “OPT Record TTL Field Use”.
    ///
    /// Unlike the `Wire::read` function, this does not require a length.
    pub fn read(c: &mut Cursor<&[u8]>) -> Result<Self, WireError> {
        let udp_payload_size = c.read_u16::<BigEndian>()?;  // replaces the class field
        let higher_bits = c.read_u8()?;                     // replaces the ttl field...
        let edns0_version = c.read_u8()?;                   // ...as does this...
        let flags = c.read_u16::<BigEndian>()?;             // ...as does this

        let data_length = c.read_u16::<BigEndian>()?;
        let mut data = Vec::new();
        for _ in 0 .. data_length {
            data.push(c.read_u8()?);
        }

        Ok(OPT { udp_payload_size, higher_bits, edns0_version, flags, data })
    }

    /// Serialises this OPT record into a vector of bytes.
    ///
    /// This is necessary for OPT records to be sent in the Additional section
    /// of requests.
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(32);

        bytes.write_u16::<BigEndian>(self.udp_payload_size)?;
        bytes.write_u8(self.higher_bits)?;
        bytes.write_u8(self.edns0_version)?;
        bytes.write_u16::<BigEndian>(self.flags)?;
        bytes.write_u16::<BigEndian>(self.data.len() as u16)?;
        for b in &self.data {
            bytes.write_u8(*b)?;
        }

        Ok(bytes)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses() {
        let buf = &[ 0x05, 0xAC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];

        assert_eq!(OPT::read(&mut Cursor::new(buf)).unwrap(),
                   OPT {
                       udp_payload_size: 1452,
                       higher_bits: 0,
                       edns0_version: 0,
                       flags: 0,
                       data: vec![],
                   });
    }

    #[test]
    fn empty() {
        assert_eq!(OPT::read(&mut Cursor::new(&[])),
                   Err(WireError::IO));
    }
}
