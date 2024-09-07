#[cfg(feature = "plugins")]
use extism::{convert::Msgpack, host_fn, FromBytes, ToBytes};

/// Used to keep track of things created by plugins.
/// This can include things like events, commands, etc.
#[derive(Hash, PartialEq, Eq, ToBytes, FromBytes, serde::Serialize, serde::Deserialize)]
#[cfg(feature = "plugins")]
#[encoding(Msgpack)] // TODO: Switch to protocal buffers for smaller size
struct Identifier {
    namespace: String,
    path: String,
}

#[cfg(feature = "plugins")]
host_fn!(new(namespace: String, path: String) -> Result<Identifier, _> {
    Ok(Identifier { namespace, path })
});
