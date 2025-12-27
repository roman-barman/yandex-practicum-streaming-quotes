use std::net::IpAddr;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) struct ClientAddress {
    address: IpAddr,
    port: u16,
}

impl ClientAddress {
    pub(crate) fn new(address: IpAddr, port: u16) -> Self {
        Self { address, port }
    }

    pub(crate) fn address(&self) -> (IpAddr, u16) {
        (self.address, self.port)
    }
}

impl std::fmt::Display for ClientAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.address, self.port)
    }
}
