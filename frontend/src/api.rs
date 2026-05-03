use common::{
    CreateScheduleRequest, FeedNowRequest, FeederCommand, FeedingSchedule, UpdateScheduleRequest,
};
use gloo_net::http::Request;

const API_BASE: &str = "http://127.0.0.1:8081/api";

async fn read_error(response: gloo_net::http::Response) -> String {
    response
        .text()
        .await
        .unwrap_or_else(|_| String::from("request failed"))
}

pub async fn fetch_schedules() -> Result<Vec<FeedingSchedule>, String> {
    let response = Request::get(&format!("{API_BASE}/schedules"))
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if !response.ok() {
        return Err(read_error(response).await);
    }

    response
        .json::<Vec<FeedingSchedule>>()
        .await
        .map_err(|err| err.to_string())
}

pub async fn create_schedule(payload: &CreateScheduleRequest) -> Result<FeedingSchedule, String> {
    let response = Request::post(&format!("{API_BASE}/schedules"))
        .json(payload)
        .map_err(|err| err.to_string())?
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if !response.ok() {
        return Err(read_error(response).await);
    }

    response
        .json::<FeedingSchedule>()
        .await
        .map_err(|err| err.to_string())
}

pub async fn update_schedule(
    schedule_id: i64,
    payload: &UpdateScheduleRequest,
) -> Result<FeedingSchedule, String> {
    let response = Request::patch(&format!("{API_BASE}/schedules/{schedule_id}"))
        .json(payload)
        .map_err(|err| err.to_string())?
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if !response.ok() {
        return Err(read_error(response).await);
    }

    response
        .json::<FeedingSchedule>()
        .await
        .map_err(|err| err.to_string())
}

pub async fn delete_schedule(schedule_id: i64) -> Result<(), String> {
    let response = Request::delete(&format!("{API_BASE}/schedules/{schedule_id}"))
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if !response.ok() {
        return Err(read_error(response).await);
    }

    Ok(())
}

pub async fn create_feed_now_command(portion: u8) -> Result<FeederCommand, String> {
    let payload = FeedNowRequest { portion };
    let response = Request::post(&format!("{API_BASE}/commands/feed-now"))
        .json(&payload)
        .map_err(|err| err.to_string())?
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if !response.ok() {
        return Err(read_error(response).await);
    }

    response
        .json::<FeederCommand>()
        .await
        .map_err(|err| err.to_string())
}

pub async fn fetch_command(command_id: i64) -> Result<FeederCommand, String> {
    let response = Request::get(&format!("{API_BASE}/commands/{command_id}"))
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if !response.ok() {
        return Err(read_error(response).await);
    }

    response
        .json::<FeederCommand>()
        .await
        .map_err(|err| err.to_string())
}
