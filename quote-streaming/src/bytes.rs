use rkyv::api::high::{HighSerializer, HighValidator};
use rkyv::bytecheck::CheckBytes;
use rkyv::de::Pool;
use rkyv::rancor::Strategy;
use rkyv::ser::allocator::ArenaHandle;
use rkyv::util::AlignedVec;
use rkyv::{Archive, Deserialize, Serialize, rancor};

pub(crate) fn to_bytes(
    value: &impl for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rancor::Error>>,
) -> Result<Vec<u8>, rancor::Error> {
    Ok(rkyv::to_bytes::<rancor::Error>(value)?.to_vec())
}

pub(crate) fn from_bytes<T>(value: &[u8]) -> Result<T, rancor::Error>
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<HighValidator<'a, rancor::Error>>
        + Deserialize<T, Strategy<Pool, rancor::Error>>,
{
    rkyv::from_bytes::<T, rancor::Error>(value)
}
