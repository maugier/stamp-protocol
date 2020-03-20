use {
    std::{
        convert::TryFrom,
        error::Error,
        fmt,
        io,
        net::{
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

#[derive(Debug)]
pub struct UnsupportedPacketFormat(SocketAddr);

impl fmt::Display for UnsupportedPacketFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unsupported packet received from {}", self.0)   
    }
}

impl Error for UnsupportedPacketFormat {
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
        let mut buf = Vec::with_capacity(112);

        let (len, source, local) = self.sock.recv_sas(&mut buf)?;

        match len {
            44 => { 
                let reply = SenderPacket::try_from(buf)?.reflect();
                self.sock.send_sas(&*reply, &source, &local)?;
            },
            // 112 => todo!(),
            _  => Err(UnsupportedPacketFormat(source)),
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
