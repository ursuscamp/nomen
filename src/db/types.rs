use anyhow::anyhow;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Blockchain {
    id: i64,
    nsid: String,
    blockhash: String,
    txid: String,
    vout: i64,
    height: i64,
    #[sqlx(try_from = "String")]
    status: BlockchainStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockchainStatus {
    Accepted,
    Rejected,
}

impl TryFrom<String> for BlockchainStatus {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            _ => Err(anyhow!("Invalid blockchain status {value}")),
        }
    }
}
