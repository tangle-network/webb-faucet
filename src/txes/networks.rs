use webb_proposals::TypedChainId;

pub enum Network {
    Sepolia,
    Goerli,
    OptimismGoerli,
    ArbitrumGoerli,
    PolygonMumbai,
    ScrollAlpha,
    MoonbaseAlpha,
    AvalancheFuji,
    Hermes,
    Athena,
    Demeter,

    // Substrate
    Tangle,
}

impl Network {
    pub fn from_string(network: &str) -> Option<Self> {
        match network {
            "sepolia" => Some(Self::Sepolia),
            "goerli" => Some(Self::Goerli),
            "optimism-goerli" => Some(Self::OptimismGoerli),
            "arbitrum-goerli" => Some(Self::ArbitrumGoerli),
            "polygon-mumbai" => Some(Self::PolygonMumbai),
            "scroll-alpha" => Some(Self::ScrollAlpha),
            "moonbase-alpha" => Some(Self::MoonbaseAlpha),
            "avalanche-fuji" => Some(Self::AvalancheFuji),
            "hermes" => Some(Self::Hermes),
            "athena" => Some(Self::Athena),
            "demeter" => Some(Self::Demeter),
            _ => None,
        }
    }

    pub fn from_evm_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            11155111 => Some(Self::Sepolia),
            5 => Some(Self::Goerli),
            420 => Some(Self::OptimismGoerli),
            421613 => Some(Self::ArbitrumGoerli),
            80001 => Some(Self::PolygonMumbai),
            534353 => Some(Self::ScrollAlpha),
            1287 => Some(Self::MoonbaseAlpha),
            43113 => Some(Self::AvalancheFuji),
            3884533461 => Some(Self::Athena),
            3884533462 => Some(Self::Hermes),
            3884533463 => Some(Self::Demeter),
            _ => None,
        }
    }

    pub fn to_evm_chain_id(&self) -> Option<u64> {
        match self {
            Self::Sepolia => Some(11155111),
            Self::Goerli => Some(5),
            Self::OptimismGoerli => Some(420),
            Self::ArbitrumGoerli => Some(421613),
            Self::PolygonMumbai => Some(80001),
            Self::ScrollAlpha => Some(534353),
            Self::MoonbaseAlpha => Some(1287),
            Self::AvalancheFuji => Some(43113),
            Self::Athena => Some(3884533461),
            Self::Hermes => Some(3884533462),
            Self::Demeter => Some(3884533463),
            _ => None,
        }
    }

    pub fn from_substrate_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            1081 => Some(Self::Tangle),
            _ => None,
        }
    }

    pub fn to_substrate_chain_id(&self) -> Option<u64> {
        match self {
            Self::Tangle => Some(1081),
            _ => None,
        }
    }

    pub fn from_typed_chain_id(typed_chain_id: TypedChainId) -> Option<Self> {
        match typed_chain_id {
            TypedChainId::Evm(chain_id) => Self::from_evm_chain_id(chain_id.into()),
            TypedChainId::Substrate(chain_id) => Self::from_substrate_chain_id(chain_id.into()),
            _ => None,
        }
    }

    pub fn to_typed_chain_id(&self) -> Option<TypedChainId> {
        match self {
            Self::Tangle => Some(TypedChainId::Substrate(1081)),
            Self::ArbitrumGoerli => Some(TypedChainId::Evm(421613)),
            Self::Athena => Some(TypedChainId::Evm(3884533461)),
            Self::Demeter => Some(TypedChainId::Evm(3884533463)),
            Self::Goerli => Some(TypedChainId::Evm(5)),
            Self::Hermes => Some(TypedChainId::Evm(3884533462)),
            Self::MoonbaseAlpha => Some(TypedChainId::Evm(1287)),
            Self::OptimismGoerli => Some(TypedChainId::Evm(420)),
            Self::PolygonMumbai => Some(TypedChainId::Evm(80001)),
            Self::ScrollAlpha => Some(TypedChainId::Evm(534353)),
            Self::Sepolia => Some(TypedChainId::Evm(11155111)),
            Self::AvalancheFuji => Some(TypedChainId::Evm(43113)),
        }
    }
}
