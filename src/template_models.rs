use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct IndexModel{
    pub data_table: String,
    pub chat_bot: String,
}

#[derive(Serialize, Deserialize)]
pub struct DataTableModel{
    pub ip: String,
    pub visitor_rank: u32,
    pub db_init_time: String,
    pub total_visitors: u32,
    pub total_bots: u32,
    pub bot_validation_id: String
}

