pub enum Contracts {
    RefExchange,
    RefFarm,
}

impl Contracts {
    pub fn value(&self) -> &str {
        match *self {
            Contracts::RefExchange => "ref-finance-101.testnet",
            Contracts::RefFarm => "v2.ref-farming.testnet",
        }
    }
}

// https://stackoverflow.com/a/36928678
pub enum Methods {
    NumPools,
    GetPools,
    ListSeeds,
    ListFarmsBySeeds,
    WhitelistedTokens,
    FtMetadata,
}

impl Methods {
    pub fn value(&self) -> String {
        match *self {
            // Exchange Contract
            Methods::NumPools => String::from("get_number_of_pools"), // ex
            Methods::GetPools => String::from("get_pools"),
            Methods::WhitelistedTokens => String::from("get_whitelisted_tokens"),

            // Farm Contract
            Methods::ListSeeds => String::from("list_seeds"),
            Methods::ListFarmsBySeeds => String::from("list_farms_by_seed"),

            // Token Contract
            Methods::FtMetadata => String::from("ft_metadata"),
        }
    }
}
