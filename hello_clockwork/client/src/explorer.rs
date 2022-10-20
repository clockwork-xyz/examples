const EXPLORER: &str = "https://explorer.solana.com";
const CK_EXPLORER: &str = "https://explorer.clockwork.xyz";

pub struct Explorer {
    cluster: &'static str,
    custom_rpc: &'static str,
}

impl Explorer {
    pub const fn devnet() -> Self {
        Self {
            cluster: "devnet",
            custom_rpc: "",
        }
    }

    #[allow(dead_code)] // because of conditional compilation
    pub const fn localnet() -> Self {
        Self {
            cluster: "custom",
            custom_rpc: "http://localhost:8899",
        }
    }

    /// Ex: https://explorer.solana.com/tx/{tx}
    ///     ?cluster=custom
    ///     &customUrl=http://localhost:8899
    pub fn tx_url<T: std::fmt::Display>(&self, tx: T) -> String {
        let url = format!("{}/tx/{}?cluster={}", EXPLORER, tx, self.cluster);
        if self.cluster == "custom" {
            url + "&customUrl=" + self.custom_rpc
        } else {
            url
        }
    }

    /// Ex: https://explorer.clockwork.xyz/queue/{queue}
    ///     ?network=custom
    ///     &customRPC=http://localhost:8899
    pub fn queue_url<T: std::fmt::Display>(&self, queue: T) -> String {
        let url = format!("{}/queue/{}?network={}", CK_EXPLORER, queue, self.cluster);
        if self.cluster == "custom" {
            url + "&customRPC=" + self.custom_rpc
        } else {
            url
        }
    }
}
