
mod ntp;

use {
    std::{
        error::Error,
        fmt,
        io,
        mem,
        net::{
            IpAddr,
            Ipv6Addr,
            SocketAddr,
            ToSocketAddrs,
            UdpSocket,
        },
    },
    ntp::Timestamp,
    udp_sas::UdpSas,
};


#[derive(Debug)]
pub enum PacketError {
    IncorrectLength,
    MBZViolation,
}   

impl PacketError {
    fn from_source(self, source: SocketAddr, local: IpAddr) -> RequestError {
        RequestError { source, local, reason: self }
    }
}

#[derive(Debug)]
pub struct RequestError {
    source: SocketAddr,
    local: IpAddr,
    reason: PacketError,
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unsupported packet received from {} to {} ({:?})",
            self.source, self.local, self.reason )   
    }
}

impl Error for RequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> { None }   
}

const PORT_NUMBER: u16 = 862;

type Sequence = [u8; 4];

#[repr(packed)]
#[derive(Debug)]
pub struct UnauthenticatedPacket {
    sequence: Sequence,
    timestamp: Timestamp,
    error: [u8; 2],
    mbz_0: [u8; 2],
    receive: Timestamp,
    sender_sequence: Sequence,
    sender_timestamp: Timestamp,
    sender_error: [u8; 2],
    mbz_1: [u8; 2],
    ttl: u8,
    mbz_2: [u8; 3],
    tail: [u8]
}

impl UnauthenticatedPacket {
    fn from_buffer(buf: &mut [u8]) -> Result<&mut UnauthenticatedPacket, PacketError> {
        if buf.len() < 44 {
            return Err(PacketError::IncorrectLength);
        }

        Ok(unsafe { mem::transmute(buf) })

    }
}


pub struct StatelessReflector {
    sock: UdpSocket
}



impl StatelessReflector {
    pub fn new() -> Result<Self, io::Error> {
        Self::bind((Ipv6Addr::UNSPECIFIED, PORT_NUMBER))
    }

    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, io::Error> {
        let sock = UdpSocket::bind_sas(addr)?;
        sock.set_ttl(255)?;
        Ok(Self { sock })
    }

    pub fn run(mut self) {
        loop {
            if let Err(msg) = self.reply() {
                eprintln!("{}", msg)
            }
        }
    }

    fn reply(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buf = [1; 4096];

        let (len, source, local) = self.sock.recv_sas(&mut buf)?;
        let ts = Timestamp::now()?;
    
        let packet = &mut buf[..len];

        {
            let mut packet = UnauthenticatedPacket::from_buffer(packet)
                .map_err(|e| e.from_source(source, local))?;

            packet.mbz_0 = [0; 2];
            packet.mbz_1 = [0; 2];
            packet.mbz_2 = [0; 3];

            packet.sender_sequence = packet.sequence;
            packet.sender_timestamp = packet.timestamp;
            packet.sender_error = packet.error;

            packet.receive = ts;
            packet.timestamp = Timestamp::now()?;
        }

        self.sock.send_sas(packet, &source, &local)?;
        Ok(())
    }
}   



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
