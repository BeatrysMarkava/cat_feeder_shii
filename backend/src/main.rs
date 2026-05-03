use std::str::FromStr;

use actix_cors::Cors;
use actix_web::{
    App, HttpResponse, HttpServer, Responder, Result, delete, get, patch, post,
    web::{Data, Json, Path, Query},
};
use common::{
    CreateScheduleRequest, DeviceCommandCompleteRequest, DeviceHeartbeatRequest,
    DeviceRegisterRequest, DeviceScheduleResponse, DeviceStatusResponse, FeederCommand,
    FeedingReportRequest, FeedingSchedule, FeedNowRequest, UpdateScheduleRequest,
};
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow, PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use time::{
    Date, Duration, OffsetDateTime, Time, format_description::well_known::Rfc3339,
    macros::format_description,
};

const TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[hour]:[minute]");
const DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[year]-[month]-[day]");
const DEFAULT_DEVICE_KEY: &str = "barsik-esp32c6";
const DEFAULT_DEVICE_NAME: &str = "Barsik ESP32-C6";
const DEFAULT_TIMEZONE: &str = "Europe/Bratislava";
const COMMAND_CLAIM_TIMEOUT_SECONDS: i64 = 30;
const COMMAND_MAX_ATTEMPTS: i32 = 3;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, FromRow)]
struct ScheduleRow {
    id: i64,
    feeding_date: Option<String>,
    feeding_day: String,
    feeding_time: String,
    timezone: String,
    portion: i32,
    enabled: bool,
    created_at: String,
}

#[derive(Debug, FromRow)]
struct DeviceRow {
    device_key: String,
    name: String,
    firmware_version: Option<String>,
    hopper_level: Option<i32>,
    ip_address: Option<String>,
    last_seen_at: Option<String>,
}

#[derive(Debug, FromRow)]
struct CommandRow {
    id: i64,
    command_type: String,
    portion: i32,
    status: String,
    retry_count: i32,
    max_attempts: i32,
    created_at: String,
    claimed_at: Option<String>,
    completed_at: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceKeyQuery {
    device_key: String,
}

impl TryFrom<ScheduleRow> for FeedingSchedule {
    type Error = actix_web::Error;

    fn try_from(value: ScheduleRow) -> Result<Self, Self::Error> {
        let portion = u8::try_from(value.portion).map_err(|_| {
            actix_web::error::ErrorInternalServerError("stored portion is out of range")
        })?;

        Ok(Self {
            id: value.id,
            next_run_at: next_run_at(
                value.feeding_date.as_deref(),
                &value.feeding_day,
                &value.feeding_time,
            )?,
            feeding_date: value.feeding_date,
            feeding_day: value.feeding_day,
            feeding_time: value.feeding_time,
            timezone: value.timezone,
            portion,
            enabled: value.enabled,
            created_at: value.created_at,
        })
    }
}

impl TryFrom<DeviceRow> for DeviceStatusResponse {
    type Error = actix_web::Error;

    fn try_from(value: DeviceRow) -> Result<Self, Self::Error> {
        let hopper_level = value
            .hopper_level
            .map(u8::try_from)
            .transpose()
            .map_err(|_| actix_web::error::ErrorInternalServerError("stored hopper level is out of range"))?;

        Ok(Self {
            device_key: value.device_key,
            name: value.name,
            online: value.last_seen_at.is_some(),
            hopper_level,
            firmware_version: value.firmware_version,
            ip_address: value.ip_address,
            last_seen_at: value.last_seen_at,
        })
    }
}

impl TryFrom<CommandRow> for FeederCommand {
    type Error = actix_web::Error;

    fn try_from(value: CommandRow) -> Result<Self, Self::Error> {
        let portion = u8::try_from(value.portion).map_err(|_| {
            actix_web::error::ErrorInternalServerError("stored command portion is out of range")
        })?;
        let retry_count = u8::try_from(value.retry_count).map_err(|_| {
            actix_web::error::ErrorInternalServerError("stored retry count is out of range")
        })?;
        let max_attempts = u8::try_from(value.max_attempts).map_err(|_| {
            actix_web::error::ErrorInternalServerError("stored max attempts is out of range")
        })?;

        Ok(Self {
            id: value.id,
            command_type: value.command_type,
            portion,
            status: value.status,
            retry_count,
            max_attempts,
            created_at: value.created_at,
            claimed_at: value.claimed_at,
            completed_at: value.completed_at,
            message: value.message,
        })
    }
}

fn normalize_feeding_day(value: &str) -> Result<String> {
    let trimmed = value.trim();
    let normalized = match trimmed.to_ascii_lowercase().as_str() {
        "monday" => "Monday",
        "tuesday" => "Tuesday",
        "wednesday" => "Wednesday",
        "thursday" => "Thursday",
        "friday" => "Friday",
        "saturday" => "Saturday",
        "sunday" => "Sunday",
        _ => {
            return Err(actix_web::error::ErrorBadRequest(
                "feeding_day must be a weekday name",
            ));
        }
    };

    Ok(String::from(normalized))
}

fn normalize_feeding_time(value: &str) -> Result<String> {
    Time::parse(value.trim(), TIME_FORMAT)
        .map(|parsed| parsed.format(TIME_FORMAT))
        .map_err(|_| actix_web::error::ErrorBadRequest("feeding_time must be HH:MM"))
        .and_then(|result| {
            result.map_err(|_| actix_web::error::ErrorInternalServerError("failed to format time"))
        })
}

fn normalize_feeding_date(value: Option<&str>) -> Result<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    Date::parse(value, DATE_FORMAT)
        .map(|date| date.format(DATE_FORMAT))
        .map_err(|_| actix_web::error::ErrorBadRequest("feeding_date must be YYYY-MM-DD"))
        .and_then(|result| {
            result.map(Some).map_err(|_| {
                actix_web::error::ErrorInternalServerError("failed to format feeding_date")
            })
        })
}

fn weekday_from_date(value: &str) -> Result<String> {
    let date = Date::parse(value, DATE_FORMAT)
        .map_err(|_| actix_web::error::ErrorBadRequest("feeding_date must be YYYY-MM-DD"))?;
    let day = match date.weekday().number_days_from_monday() {
        0 => "Monday",
        1 => "Tuesday",
        2 => "Wednesday",
        3 => "Thursday",
        4 => "Friday",
        5 => "Saturday",
        _ => "Sunday",
    };

    Ok(String::from(day))
}

fn normalize_timezone(value: Option<&str>) -> Result<String> {
    let timezone = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_TIMEZONE);

    if timezone.len() > 64 {
        return Err(actix_web::error::ErrorBadRequest(
            "timezone must be at most 64 characters",
        ));
    }

    Ok(String::from(timezone))
}

fn weekday_index(value: &str) -> Result<i64> {
    match value {
        "Monday" => Ok(0),
        "Tuesday" => Ok(1),
        "Wednesday" => Ok(2),
        "Thursday" => Ok(3),
        "Friday" => Ok(4),
        "Saturday" => Ok(5),
        "Sunday" => Ok(6),
        _ => Err(actix_web::error::ErrorBadRequest(
            "feeding_day must be a weekday name",
        )),
    }
}

fn next_run_at(feeding_date: Option<&str>, feeding_day: &str, feeding_time: &str) -> Result<String> {
    let now = OffsetDateTime::now_utc();
    let target_time = Time::parse(feeding_time, TIME_FORMAT)
        .map_err(|_| actix_web::error::ErrorBadRequest("feeding_time must be HH:MM"))?;

    if let Some(feeding_date) = feeding_date {
        let target_date = Date::parse(feeding_date, DATE_FORMAT)
            .map_err(|_| actix_web::error::ErrorBadRequest("feeding_date must be YYYY-MM-DD"))?;
        let candidate = target_date.with_time(target_time).assume_utc();
        return candidate
            .format(&Rfc3339)
            .map_err(|_| actix_web::error::ErrorInternalServerError("failed to format next_run_at"));
    }

    let target_day = weekday_index(feeding_day)?;
    let current_day = i64::from(now.weekday().number_days_from_monday());
    let mut days_until = (target_day - current_day + 7) % 7;
    let mut candidate = (now.date() + Duration::days(days_until))
        .with_time(target_time)
        .assume_utc();

    if candidate <= now {
        days_until += 7;
        candidate = (now.date() + Duration::days(days_until))
            .with_time(target_time)
            .assume_utc();
    }

    candidate
        .format(&Rfc3339)
        .map_err(|_| actix_web::error::ErrorInternalServerError("failed to format next_run_at"))
}

fn now_rfc3339() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| actix_web::error::ErrorInternalServerError("failed to format timestamp"))
}

fn validate_device_key(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(actix_web::error::ErrorBadRequest(
            "device_key must be 1..64 characters",
        ));
    }

    Ok(String::from(trimmed))
}

fn validate_portion(value: u8) -> Result<u8> {
    if !(1..=5).contains(&value) {
        return Err(actix_web::error::ErrorBadRequest(
            "portion must be between 1 and 5",
        ));
    }

    Ok(value)
}

fn log_api(action: &str, detail: impl AsRef<str>) {
    println!("[api] {action}: {}", detail.as_ref());
}

async fn init_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS feeding_schedules (
            id BIGSERIAL PRIMARY KEY,
            feeding_date TEXT,
            feeding_day TEXT NOT NULL DEFAULT 'Monday',
            feeding_time TEXT NOT NULL DEFAULT '08:00',
            timezone TEXT NOT NULL DEFAULT 'Europe/Bratislava',
            portion INTEGER NOT NULL CHECK (portion BETWEEN 1 AND 5),
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("ALTER TABLE feeding_schedules ADD COLUMN IF NOT EXISTS feeding_day TEXT NOT NULL DEFAULT 'Monday'")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE feeding_schedules ADD COLUMN IF NOT EXISTS feeding_date TEXT")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE feeding_schedules ADD COLUMN IF NOT EXISTS feeding_time TEXT NOT NULL DEFAULT '08:00'")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE feeding_schedules ADD COLUMN IF NOT EXISTS timezone TEXT NOT NULL DEFAULT 'Europe/Bratislava'")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_name = 'feeding_schedules'
                    AND column_name = 'scheduled_for'
            ) THEN
                ALTER TABLE feeding_schedules ALTER COLUMN scheduled_for DROP NOT NULL;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS feeder_devices (
            device_key TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            firmware_version TEXT,
            hopper_level INTEGER CHECK (hopper_level BETWEEN 0 AND 100),
            ip_address TEXT,
            last_seen_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS feeder_commands (
            id BIGSERIAL PRIMARY KEY,
            device_key TEXT NOT NULL REFERENCES feeder_devices(device_key) ON DELETE CASCADE,
            command_type TEXT NOT NULL,
            portion INTEGER NOT NULL CHECK (portion BETWEEN 1 AND 5),
            status TEXT NOT NULL,
            retry_count INTEGER NOT NULL DEFAULT 0,
            max_attempts INTEGER NOT NULL DEFAULT 3,
            message TEXT,
            created_at TEXT NOT NULL,
            claimed_at TEXT,
            completed_at TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("ALTER TABLE feeder_commands ADD COLUMN IF NOT EXISTS retry_count INTEGER NOT NULL DEFAULT 0")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE feeder_commands ADD COLUMN IF NOT EXISTS max_attempts INTEGER NOT NULL DEFAULT 3")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE feeder_commands ADD COLUMN IF NOT EXISTS message TEXT")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE feeder_commands ADD COLUMN IF NOT EXISTS claimed_at TEXT")
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE feeder_commands ADD COLUMN IF NOT EXISTS completed_at TEXT")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS feeding_history (
            id BIGSERIAL PRIMARY KEY,
            device_key TEXT NOT NULL REFERENCES feeder_devices(device_key) ON DELETE CASCADE,
            portion INTEGER NOT NULL CHECK (portion BETWEEN 1 AND 5),
            source TEXT NOT NULL,
            fed_at TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[get("/api/health")]
async fn health() -> impl Responder {
    log_api("health", "ok");
    Json(HealthResponse { status: "ok" })
}

async fn default_device_key(state: &Data<AppState>) -> Result<String> {
    let row = sqlx::query("SELECT device_key FROM feeder_devices ORDER BY created_at ASC LIMIT 1")
        .fetch_optional(&state.db)
        .await
        .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    if let Some(row) = row {
        let device_key: String = row.get("device_key");
        return Ok(device_key);
    }

    let device_key = String::from(DEFAULT_DEVICE_KEY);
    let now = now_rfc3339()?;
    sqlx::query(
        r#"
        INSERT INTO feeder_devices (device_key, name, created_at, updated_at)
        VALUES ($1, $2, $3, $3)
        ON CONFLICT (device_key) DO NOTHING
        "#,
    )
    .bind(&device_key)
    .bind(DEFAULT_DEVICE_NAME)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    Ok(device_key)
}

async fn release_stale_commands(state: &Data<AppState>) -> Result<()> {
    let now = now_rfc3339()?;

    let failed = sqlx::query(
        r#"
        UPDATE feeder_commands
        SET status = 'failed',
            retry_count = retry_count + 1,
            message = 'command timed out after all attempts',
            completed_at = $2
        WHERE status = 'claimed'
            AND claimed_at IS NOT NULL
            AND claimed_at::timestamptz < now() - ($1 * interval '1 second')
            AND retry_count + 1 >= max_attempts
        "#,
    )
    .bind(COMMAND_CLAIM_TIMEOUT_SECONDS)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    let retried = sqlx::query(
        r#"
        UPDATE feeder_commands
        SET status = 'pending',
            retry_count = retry_count + 1,
            message = 'command timed out and was queued again',
            claimed_at = NULL
        WHERE status = 'claimed'
            AND claimed_at IS NOT NULL
            AND claimed_at::timestamptz < now() - ($1 * interval '1 second')
            AND retry_count + 1 < max_attempts
        "#,
    )
    .bind(COMMAND_CLAIM_TIMEOUT_SECONDS)
    .execute(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    if failed.rows_affected() > 0 || retried.rows_affected() > 0 {
        log_api(
            "stale command sweep",
            format!(
                "retried={}, failed={}",
                retried.rows_affected(),
                failed.rows_affected()
            ),
        );
    }

    Ok(())
}

async fn get_device_status_by_key(
    state: &Data<AppState>,
    device_key: &str,
) -> Result<DeviceStatusResponse> {
    let row = sqlx::query_as::<_, DeviceRow>(
        r#"
        SELECT device_key, name, firmware_version, hopper_level, ip_address, last_seen_at
        FROM feeder_devices
        WHERE device_key = $1
        "#,
    )
    .bind(device_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("device not registered"))?;

    DeviceStatusResponse::try_from(row)
}

#[get("/api/device/status")]
async fn device_status(state: Data<AppState>) -> Result<Json<DeviceStatusResponse>> {
    let device_key = default_device_key(&state).await?;
    let status = get_device_status_by_key(&state, &device_key).await?;
    log_api(
        "device status",
        format!(
            "device_key={}, online={}, hopper_level={:?}",
            status.device_key, status.online, status.hopper_level
        ),
    );
    Ok(Json(status))
}

#[post("/api/device/register")]
async fn register_device(
    state: Data<AppState>,
    payload: Json<DeviceRegisterRequest>,
) -> Result<Json<DeviceStatusResponse>> {
    let device_key = validate_device_key(&payload.device_key)?;
    let name = payload
        .name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| String::from(DEFAULT_DEVICE_NAME));
    let now = now_rfc3339()?;
    log_api(
        "device register received",
        format!(
            "device_key={}, name={}, firmware={:?}",
            device_key, name, payload.firmware_version
        ),
    );

    sqlx::query(
        r#"
        INSERT INTO feeder_devices (
            device_key, name, firmware_version, last_seen_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $4, $4)
        ON CONFLICT (device_key) DO UPDATE
        SET name = EXCLUDED.name,
            firmware_version = EXCLUDED.firmware_version,
            last_seen_at = EXCLUDED.last_seen_at,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&device_key)
    .bind(&name)
    .bind(&payload.firmware_version)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    let status = get_device_status_by_key(&state, &device_key).await?;
    log_api("device register response", format!("device_key={}", status.device_key));
    Ok(Json(status))
}

#[post("/api/device/heartbeat")]
async fn device_heartbeat(
    state: Data<AppState>,
    payload: Json<DeviceHeartbeatRequest>,
) -> Result<Json<DeviceStatusResponse>> {
    let device_key = validate_device_key(&payload.device_key)?;
    log_api(
        "device heartbeat received",
        format!(
            "device_key={}, hopper_level={:?}, ip={:?}, firmware={:?}",
            device_key, payload.hopper_level, payload.ip_address, payload.firmware_version
        ),
    );
    if let Some(level) = payload.hopper_level {
        if level > 100 {
            return Err(actix_web::error::ErrorBadRequest(
                "hopper_level must be between 0 and 100",
            ));
        }
    }
    let now = now_rfc3339()?;

    sqlx::query(
        r#"
        INSERT INTO feeder_devices (
            device_key, name, firmware_version, hopper_level, ip_address, last_seen_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $6, $6)
        ON CONFLICT (device_key) DO UPDATE
        SET firmware_version = COALESCE(EXCLUDED.firmware_version, feeder_devices.firmware_version),
            hopper_level = COALESCE(EXCLUDED.hopper_level, feeder_devices.hopper_level),
            ip_address = COALESCE(EXCLUDED.ip_address, feeder_devices.ip_address),
            last_seen_at = EXCLUDED.last_seen_at,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&device_key)
    .bind(DEFAULT_DEVICE_NAME)
    .bind(&payload.firmware_version)
    .bind(payload.hopper_level.map(i32::from))
    .bind(&payload.ip_address)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    let status = get_device_status_by_key(&state, &device_key).await?;
    log_api(
        "device heartbeat response",
        format!(
            "device_key={}, hopper_level={:?}, last_seen_at={:?}",
            status.device_key, status.hopper_level, status.last_seen_at
        ),
    );
    Ok(Json(status))
}

async fn load_schedules(state: &Data<AppState>) -> Result<Vec<FeedingSchedule>> {
    let rows = sqlx::query_as::<_, ScheduleRow>(
        r#"
        SELECT id, feeding_date, feeding_day, feeding_time, timezone, portion, enabled, created_at
        FROM feeding_schedules
        ORDER BY
            CASE feeding_day
                WHEN 'Monday' THEN 1
                WHEN 'Tuesday' THEN 2
                WHEN 'Wednesday' THEN 3
                WHEN 'Thursday' THEN 4
                WHEN 'Friday' THEN 5
                WHEN 'Saturday' THEN 6
                WHEN 'Sunday' THEN 7
                ELSE 8
            END,
            feeding_date ASC NULLS LAST,
            feeding_time ASC,
            id ASC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    let schedules = rows
        .into_iter()
        .map(FeedingSchedule::try_from)
        .collect::<Result<Vec<_>>>()?;

    Ok(schedules)
}

#[get("/api/schedules")]
async fn list_schedules(state: Data<AppState>) -> Result<Json<Vec<FeedingSchedule>>> {
    let schedules = load_schedules(&state).await?;
    log_api("list schedules", format!("count={}", schedules.len()));
    Ok(Json(schedules))
}

#[get("/api/device/schedule")]
async fn device_schedule(state: Data<AppState>) -> Result<Json<DeviceScheduleResponse>> {
    let schedules = load_schedules(&state).await?;
    log_api(
        "device schedule response",
        format!("count={}", schedules.len()),
    );
    Ok(Json(DeviceScheduleResponse {
        server_time: now_rfc3339()?,
        schedules,
    }))
}

#[post("/api/schedules")]
async fn create_schedule(
    state: Data<AppState>,
    payload: Json<CreateScheduleRequest>,
) -> Result<HttpResponse> {
    let feeding_day = normalize_feeding_day(&payload.feeding_day)?;
    let feeding_time = normalize_feeding_time(&payload.feeding_time)?;
    let feeding_date = normalize_feeding_date(payload.feeding_date.as_deref())?;
    let feeding_day = match &feeding_date {
        Some(date) => weekday_from_date(date)?,
        None => feeding_day,
    };
    let timezone = normalize_timezone(payload.timezone.as_deref())?;

    let portion = validate_portion(payload.portion)?;
    log_api(
        "create schedule received",
        format!(
            "day={}, time={}, timezone={}, portion={}",
            feeding_day, feeding_time, timezone, portion
        ),
    );

    let created_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| actix_web::error::ErrorInternalServerError("failed to format created_at"))?;

    let result = sqlx::query(
        r#"
        INSERT INTO feeding_schedules (feeding_date, feeding_day, feeding_time, timezone, portion, enabled, created_at)
        VALUES ($1, $2, $3, $4, $5, TRUE, $6)
        RETURNING id
        "#,
    )
    .bind(&feeding_date)
    .bind(&feeding_day)
    .bind(&feeding_time)
    .bind(&timezone)
    .bind(i32::from(portion))
    .bind(&created_at)
    .fetch_one(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    let id: i64 = result.get("id");

    let schedule = FeedingSchedule {
        id,
        next_run_at: next_run_at(feeding_date.as_deref(), &feeding_day, &feeding_time)?,
        feeding_date,
        feeding_day,
        feeding_time,
        timezone,
        portion,
        enabled: true,
        created_at,
    };
    log_api(
        "create schedule response",
        format!("id={}, next_run_at={}", schedule.id, schedule.next_run_at),
    );
    Ok(HttpResponse::Created().json(schedule))
}

#[post("/api/commands/feed-now")]
async fn create_feed_now_command(
    state: Data<AppState>,
    payload: Json<FeedNowRequest>,
) -> Result<HttpResponse> {
    let portion = validate_portion(payload.portion)?;
    let device_key = default_device_key(&state).await?;
    let now = now_rfc3339()?;
    log_api(
        "feed-now command received",
        format!("device_key={}, portion={}", device_key, portion),
    );

    let row = sqlx::query_as::<_, CommandRow>(
        r#"
        INSERT INTO feeder_commands (
            device_key, command_type, portion, status, retry_count, max_attempts, created_at
        )
        VALUES ($1, 'feed_now', $2, 'pending', 0, $3, $4)
        RETURNING id, command_type, portion, status, retry_count, max_attempts, created_at, claimed_at, completed_at, message
        "#,
    )
    .bind(&device_key)
    .bind(i32::from(portion))
    .bind(COMMAND_MAX_ATTEMPTS)
    .bind(&now)
    .fetch_one(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    let command = FeederCommand::try_from(row)?;
    log_api(
        "feed-now command response",
        format!(
            "id={}, status={}, retry={}/{}",
            command.id, command.status, command.retry_count, command.max_attempts
        ),
    );
    Ok(HttpResponse::Accepted().json(command))
}

#[get("/api/device/commands/next")]
async fn next_device_command(
    state: Data<AppState>,
    query: Query<DeviceKeyQuery>,
) -> Result<Json<Option<FeederCommand>>> {
    let device_key = validate_device_key(&query.device_key)?;
    let now = now_rfc3339()?;
    log_api(
        "device next-command received",
        format!("device_key={device_key}"),
    );
    release_stale_commands(&state).await?;

    let row = sqlx::query_as::<_, CommandRow>(
        r#"
        UPDATE feeder_commands
        SET status = 'claimed', claimed_at = $2
        WHERE id = (
            SELECT id
            FROM feeder_commands
            WHERE device_key = $1 AND status = 'pending'
            ORDER BY created_at ASC
            LIMIT 1
        )
        RETURNING id, command_type, portion, status, retry_count, max_attempts, created_at, claimed_at, completed_at, message
        "#,
    )
    .bind(&device_key)
    .bind(&now)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    let command = row.map(FeederCommand::try_from).transpose()?;
    match &command {
        Some(command) => log_api(
            "device next-command response",
            format!(
                "id={}, status={}, retry={}/{}",
                command.id, command.status, command.retry_count, command.max_attempts
            ),
        ),
        None => log_api("device next-command response", "no pending commands"),
    }
    Ok(Json(command))
}

#[patch("/api/device/commands/{id}")]
async fn complete_device_command(
    state: Data<AppState>,
    command_id: Path<i64>,
    payload: Json<DeviceCommandCompleteRequest>,
) -> Result<Json<FeederCommand>> {
    let device_key = validate_device_key(&payload.device_key)?;
    let status = match payload.status.as_str() {
        "completed" | "failed" => payload.status.as_str(),
        _ => {
            return Err(actix_web::error::ErrorBadRequest(
                "status must be completed or failed",
            ));
        }
    };
    let now = now_rfc3339()?;
    log_api(
        "device command completion received",
        format!(
            "id={}, device_key={}, status={}, message={:?}",
            *command_id, device_key, status, payload.message
        ),
    );

    let row = sqlx::query_as::<_, CommandRow>(
        r#"
        UPDATE feeder_commands
        SET status = $1, message = $2, completed_at = $3
        WHERE id = $4 AND device_key = $5
        RETURNING id, command_type, portion, status, retry_count, max_attempts, created_at, claimed_at, completed_at, message
        "#,
    )
    .bind(status)
    .bind(&payload.message)
    .bind(&now)
    .bind(*command_id)
    .bind(&device_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("command not found"))?;

    let command = FeederCommand::try_from(row)?;
    log_api(
        "device command completion response",
        format!(
            "id={}, status={}, completed_at={:?}",
            command.id, command.status, command.completed_at
        ),
    );
    Ok(Json(command))
}

#[get("/api/commands/{id}")]
async fn command_status(
    state: Data<AppState>,
    command_id: Path<i64>,
) -> Result<Json<FeederCommand>> {
    release_stale_commands(&state).await?;
    log_api("command status received", format!("id={}", *command_id));

    let row = sqlx::query_as::<_, CommandRow>(
        r#"
        SELECT id, command_type, portion, status, retry_count, max_attempts, created_at, claimed_at, completed_at, message
        FROM feeder_commands
        WHERE id = $1
        "#,
    )
    .bind(*command_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("command not found"))?;

    let command = FeederCommand::try_from(row)?;
    log_api(
        "command status response",
        format!(
            "id={}, status={}, retry={}/{}, message={:?}",
            command.id, command.status, command.retry_count, command.max_attempts, command.message
        ),
    );
    Ok(Json(command))
}

#[post("/api/device/feedings")]
async fn report_feeding(
    state: Data<AppState>,
    payload: Json<FeedingReportRequest>,
) -> Result<HttpResponse> {
    let device_key = validate_device_key(&payload.device_key)?;
    let portion = validate_portion(payload.portion)?;
    let fed_at = match &payload.fed_at {
        Some(value) if !value.trim().is_empty() => value.clone(),
        _ => now_rfc3339()?,
    };
    let created_at = now_rfc3339()?;
    log_api(
        "feeding report received",
        format!(
            "device_key={}, portion={}, source={}, fed_at={}",
            device_key, portion, payload.source, fed_at
        ),
    );

    sqlx::query(
        r#"
        INSERT INTO feeding_history (device_key, portion, source, fed_at, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(&device_key)
    .bind(i32::from(portion))
    .bind(&payload.source)
    .bind(&fed_at)
    .bind(&created_at)
    .execute(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    log_api("feeding report response", "created");
    Ok(HttpResponse::Created().finish())
}

#[patch("/api/schedules/{id}")]
async fn update_schedule(
    state: Data<AppState>,
    schedule_id: Path<i64>,
    payload: Json<UpdateScheduleRequest>,
) -> Result<Json<FeedingSchedule>> {
    let mut current = sqlx::query_as::<_, ScheduleRow>(
        r#"
        SELECT id, feeding_date, feeding_day, feeding_time, timezone, portion, enabled, created_at
        FROM feeding_schedules
        WHERE id = $1
        "#,
    )
    .bind(*schedule_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("schedule not found"))?;

    if let Some(feeding_day) = &payload.feeding_day {
        current.feeding_day = normalize_feeding_day(feeding_day)?;
    }

    if let Some(feeding_date) = &payload.feeding_date {
        current.feeding_date = normalize_feeding_date(Some(feeding_date))?;
        if let Some(date) = &current.feeding_date {
            current.feeding_day = weekday_from_date(date)?;
        }
    }

    if let Some(feeding_time) = &payload.feeding_time {
        current.feeding_time = normalize_feeding_time(feeding_time)?;
    }

    if let Some(timezone) = &payload.timezone {
        current.timezone = normalize_timezone(Some(timezone))?;
    }

    if let Some(portion) = payload.portion {
        current.portion = i32::from(validate_portion(portion)?);
    }

    if let Some(enabled) = payload.enabled {
        current.enabled = enabled;
    }
    log_api(
        "update schedule received",
        format!(
            "id={}, day={}, time={}, timezone={}, portion={}, enabled={}",
            current.id,
            current.feeding_day,
            current.feeding_time,
            current.timezone,
            current.portion,
            current.enabled
        ),
    );

    sqlx::query(
        r#"
        UPDATE feeding_schedules
        SET feeding_date = $1, feeding_day = $2, feeding_time = $3, timezone = $4, portion = $5, enabled = $6
        WHERE id = $7
        "#,
    )
    .bind(&current.feeding_date)
    .bind(&current.feeding_day)
    .bind(&current.feeding_time)
    .bind(&current.timezone)
    .bind(current.portion)
    .bind(current.enabled)
    .bind(current.id)
    .execute(&state.db)
    .await
    .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    let schedule = FeedingSchedule::try_from(current)?;
    log_api(
        "update schedule response",
        format!(
            "id={}, enabled={}, next_run_at={}",
            schedule.id, schedule.enabled, schedule.next_run_at
        ),
    );
    Ok(Json(schedule))
}

#[delete("/api/schedules/{id}")]
async fn delete_schedule(state: Data<AppState>, schedule_id: Path<i64>) -> Result<HttpResponse> {
    log_api("delete schedule received", format!("id={}", *schedule_id));
    let result = sqlx::query("DELETE FROM feeding_schedules WHERE id = $1")
        .bind(*schedule_id)
        .execute(&state.db)
        .await
        .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("schedule not found"));
    }

    log_api("delete schedule response", format!("id={} deleted", *schedule_id));
    Ok(HttpResponse::NoContent().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| String::from("postgres://postgres:postgres@localhost:5432/barsik"));

    let connect_options = PgConnectOptions::from_str(&database_url)
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    init_db(&pool)
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    let server_state = Data::new(AppState { db: pool });

    println!("Barsik backend is running at http://127.0.0.1:8081");
    println!("Using database at {database_url}");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:8080")
            .allowed_origin("http://localhost:8080")
            .allow_any_header()
            .allow_any_method();

        App::new()
            .wrap(cors)
            .app_data(server_state.clone())
            .service(health)
            .service(device_status)
            .service(register_device)
            .service(device_heartbeat)
            .service(device_schedule)
            .service(create_feed_now_command)
            .service(command_status)
            .service(next_device_command)
            .service(complete_device_command)
            .service(report_feeding)
            .service(list_schedules)
            .service(create_schedule)
            .service(update_schedule)
            .service(delete_schedule)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
