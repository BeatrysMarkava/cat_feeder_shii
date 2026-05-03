use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedingSchedule {
    pub id: i64,
    pub feeding_date: Option<String>,
    pub feeding_day: String,
    pub feeding_time: String,
    pub timezone: String,
    pub next_run_at: String,
    pub portion: u8,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateScheduleRequest {
    pub feeding_date: Option<String>,
    pub feeding_day: String,
    pub feeding_time: String,
    pub timezone: Option<String>,
    pub portion: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateScheduleRequest {
    pub feeding_date: Option<String>,
    pub feeding_day: Option<String>,
    pub feeding_time: Option<String>,
    pub timezone: Option<String>,
    pub portion: Option<u8>,
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceRegisterRequest {
    pub device_key: String,
    pub name: Option<String>,
    pub firmware_version: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceHeartbeatRequest {
    pub device_key: String,
    pub hopper_level: Option<u8>,
    pub firmware_version: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceStatusResponse {
    pub device_key: String,
    pub name: String,
    pub online: bool,
    pub hopper_level: Option<u8>,
    pub firmware_version: Option<String>,
    pub ip_address: Option<String>,
    pub last_seen_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceScheduleResponse {
    pub server_time: String,
    pub schedules: Vec<FeedingSchedule>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedNowRequest {
    pub portion: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeederCommand {
    pub id: i64,
    pub command_type: String,
    pub portion: u8,
    pub status: String,
    pub retry_count: u8,
    pub max_attempts: u8,
    pub created_at: String,
    pub claimed_at: Option<String>,
    pub completed_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceCommandCompleteRequest {
    pub device_key: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedingReportRequest {
    pub device_key: String,
    pub portion: u8,
    pub fed_at: Option<String>,
    pub source: String,
}
