use hickory_proto::{
    op::{header::MessageType, op_code::OpCode, query::Query, Message, ResponseCode},
    rr::{record_type::RecordType, Name, RData},
};
use std::io::{Error, ErrorKind};
use std::{net::IpAddr, str::FromStr};

#[allow(dead_code)]
pub fn build_dns_query(domain: &str, query_type: RecordType, used_by_tcp: bool) -> std::io::Result<Vec<u8>> {
    use rand::{rngs::StdRng, Rng, SeedableRng};
    let name = Name::from_str(domain).map_err(|e| Error::new(ErrorKind::InvalidInput, e.to_string()))?;
    let query = Query::query(name, query_type);
    let mut msg = Message::new();
    msg.add_query(query)
        .set_id(StdRng::from_entropy().gen())
        .set_op_code(OpCode::Query)
        .set_message_type(MessageType::Query)
        .set_recursion_desired(true);
    let mut msg_buf = msg.to_vec().map_err(|e| Error::new(ErrorKind::InvalidInput, e.to_string()))?;
    if used_by_tcp {
        let mut buf = (msg_buf.len() as u16).to_be_bytes().to_vec();
        buf.append(&mut msg_buf);
        Ok(buf)
    } else {
        Ok(msg_buf)
    }
}

pub fn extract_ipaddr_from_dns_message(message: &Message) -> std::io::Result<IpAddr> {
    if message.response_code() != ResponseCode::NoError {
        return Err(Error::new(ErrorKind::InvalidData, format!("{:?}", message.response_code())));
    }
    let mut cname = None;
    for answer in message.answers() {
        match answer.data().ok_or(Error::new(ErrorKind::InvalidData, "No answer data"))? {
            RData::A(addr) => {
                return Ok(IpAddr::V4((*addr).into()));
            }
            RData::AAAA(addr) => {
                return Ok(IpAddr::V6((*addr).into()));
            }
            RData::CNAME(name) => {
                cname = Some(name.to_utf8());
            }
            _ => {}
        }
    }
    if let Some(cname) = cname {
        return Err(Error::new(ErrorKind::InvalidData, format!("CNAME: {}", cname)));
    }
    Err(Error::new(ErrorKind::InvalidData, format!("{:?}", message.answers())))
}

pub fn extract_domain_from_dns_message(message: &Message) -> std::io::Result<String> {
    let err = Error::new(ErrorKind::InvalidData, "DnsRequest no query body");
    let query = message.queries().first().ok_or(err)?;
    let name = query.name().to_string();
    Ok(name)
}

pub fn parse_data_to_dns_message(data: &[u8], used_by_tcp: bool) -> std::io::Result<Message> {
    if used_by_tcp {
        let err = Error::new(ErrorKind::InvalidData, "invalid dns data");
        if data.len() < 2 {
            return Err(err);
        }
        let len = u16::from_be_bytes([data[0], data[1]]) as usize;
        let data = data.get(2..len + 2).ok_or(err)?;
        return parse_data_to_dns_message(data, false);
    }
    let message = Message::from_vec(data).map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
    Ok(message)
}
