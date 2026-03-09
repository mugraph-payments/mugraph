#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCardanoNetwork {
    value: String,
}

impl std::fmt::Display for InvalidCardanoNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unknown network: {}. Use mainnet, preprod, preview, or testnet",
            self.value
        )
    }
}

impl std::error::Error for InvalidCardanoNetwork {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardanoNetwork {
    Mainnet,
    Preprod,
    Preview,
    Testnet,
}

impl CardanoNetwork {
    pub fn parse(value: &str) -> Result<Self, InvalidCardanoNetwork> {
        match value {
            "mainnet" => Ok(Self::Mainnet),
            "preprod" => Ok(Self::Preprod),
            "preview" => Ok(Self::Preview),
            "testnet" => Ok(Self::Testnet),
            _ => Err(InvalidCardanoNetwork {
                value: value.to_string(),
            }),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Preprod => "preprod",
            Self::Preview => "preview",
            Self::Testnet => "testnet",
        }
    }

    pub fn network_byte(self) -> u8 {
        match self {
            Self::Mainnet => 1,
            Self::Preprod => 0,
            Self::Preview => 2,
            Self::Testnet => 3,
        }
    }

    pub fn address_hrp(self) -> &'static str {
        match self {
            Self::Mainnet => "addr",
            Self::Preprod | Self::Preview | Self::Testnet => "addr_test",
        }
    }

    pub fn address_network_id(self) -> u8 {
        match self {
            Self::Mainnet => 1,
            Self::Preprod | Self::Preview | Self::Testnet => 0,
        }
    }

    pub fn blockfrost_base_url(self) -> &'static str {
        match self {
            Self::Mainnet => "https://cardano-mainnet.blockfrost.io/api/v0",
            Self::Preprod => "https://cardano-preprod.blockfrost.io/api/v0",
            Self::Preview => "https://cardano-preview.blockfrost.io/api/v0",
            Self::Testnet => "https://cardano-testnet.blockfrost.io/api/v0",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CardanoNetwork;

    #[test]
    fn cardano_network_roundtrips_to_exact_lowercase_strings() {
        for raw in ["mainnet", "preprod", "preview", "testnet"] {
            let network = CardanoNetwork::parse(raw).expect("known network must parse");
            assert_eq!(network.as_str(), raw);
        }
    }
}
