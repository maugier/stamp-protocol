use {
    std::{
        convert::TryFrom,
        error::Error,
        fmt,
        io,
        net::{
            IpAddr,
            Ipv6Addr,
            SocketAddr,
            ToSocketAddrs,
            UdpSocket,
        },
        ops::Deref,
        time::SystemTime,
    },
    byteorder::{ReadBytesExt, WriteBytesExt, BigEndian},
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

type Sequence = u32;
type Timestamp = u64;

pub struct SenderPacket<'a>(&'a mut [u8]);


impl<'a> TryFrom<&'a mut [u8]> for SenderPacket<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a mut [u8]) -> Result<Self,PacketError> {
        if packet.len() != 44 {
            return Err(PacketError::IncorrectLength)
        }

        if &packet[14..44] != &[0; 30] {
            return Err(PacketError::MBZViolation)
        }

        Ok(Self(packet))
    }

}

impl<'a> SenderPacket<'a> {
    fn reflect(self) -> ReflectorPacket<'a> {
        let packet = self.0;

        ReflectorPacket(packet)
    }
}

pub struct ReflectorPacket<'a> (&'a mut [u8]);

impl<'a> Deref for ReflectorPacket<'a> {
    type Target = [u8];
    fn deref(&self) -> &[u8] { self.0 }
}

pub struct StatelessReflector {
    sock: UdpSocket
}

pub struct ErrorMeasure(u16);
    


impl StatelessReflector {
    pub fn new() -> Result<Self, io::Error> {
        Self::bind((Ipv6Addr::UNSPECIFIED, PORT_NUMBER))
    }

    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, io::Error> {
        Ok(Self { sock: UdpSocket::bind_sas(addr)? })
    }

    pub fn run(mut self) {
        loop {
            if let Err(msg) = self.reply() {
                eprintln!("{}", msg)
            }
        }
    }

    fn reply(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buf = [1; 112];

        let (len, source, local) = self.sock.recv_sas(&mut buf)?;

        match len {
            44 => { 
                let buf = &mut buf[..44];
                let reply = SenderPacket::try_from(buf)
                    .map_err(|e| e.from_source(source, local))?
                    .reflect();

                self.sock.send_sas(&*reply, &source, &local)?;
                Ok(())
                    
            },
            _  => Err(Box::new(PacketError::IncorrectLength.from_source(source, local) )),
        }
    }
}   



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
