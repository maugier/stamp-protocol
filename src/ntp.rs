
use {
    byteorder::{ByteOrder, BigEndian},
    std::{
        convert::TryFrom,
        error::Error,
        time::{UNIX_EPOCH, SystemTimeError},
        num::TryFromIntError,
        fmt,
    },
    //lazy_static::lazy_static,
};

/*
lazy_static! {
    pub static ref NTP_EPOCH: SystemTime = UNIX_EPOCH.checked_sub(Duration::from_secs(2_208_988_800)).unwrap();
}
*/

pub const NTP_EPOCH_OFFSET: u64 = 2_208_988_800;
pub const FRACTION_PER_NANOSECOND: f64 = 4294967296e-9;

#[derive(Debug)]
pub enum TimestampError {
    OutOfRange,
    SystemTimeError(SystemTimeError),
}

impl From<SystemTimeError> for TimestampError {
    fn from(e: SystemTimeError) -> Self { TimestampError::SystemTimeError(e) }
}

impl From<TryFromIntError> for TimestampError {
    fn from(_: TryFromIntError) -> Self { TimestampError::OutOfRange }
}

impl fmt::Display for TimestampError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use TimestampError::*;
        match self {
            OutOfRange => write!(fmt, "Current time out of representable range"),
            SystemTimeError(e) => write!(fmt, "Error computing system time: {}", e),
        }
    }
}

impl Error for TimestampError {

}

#[repr(transparent)]
#[derive(Debug,Clone,Copy)]
pub struct Timestamp([u8; 8]); 

impl Timestamp {
    pub fn now() -> Result<Self, TimestampError> {

        let stamp = UNIX_EPOCH.elapsed()?;

        let seconds = u32::try_from(stamp.as_secs() + NTP_EPOCH_OFFSET)? ;
        let fraction = (stamp.subsec_nanos() as f64 * FRACTION_PER_NANOSECOND) as u32;
        let ts = ((seconds as u64) << 32) | (fraction as u64);

        let mut buf = [0; 8];
        BigEndian::write_u64(&mut buf, ts);
        Ok(Timestamp(buf))
    }

}

