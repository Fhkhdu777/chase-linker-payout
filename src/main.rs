use std::{
    collections::HashMap, convert::Infallible, env, net::SocketAddr, sync::Arc, time::Duration,
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, sse::Event as SseEvent, sse::KeepAlive, sse::Sse},
    routing::{get, post},
};
use chrono::NaiveDateTime;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, postgres::PgPoolOptions};
use tokio::sync::{Mutex, RwLock, broadcast, watch};
use tokio::time::{self, MissedTickBehavior};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use uuid::Uuid;

use reqwest::Client;

mod frontend;

const ELIGIBLE_TRADERS_QUERY: &str = r#"
    SELECT DISTINCT
        u."id",
        u."email",
        u."numericId",
        u."balanceRub",
        u."frozenRub",
        u."payoutBalance"
    FROM "Payout" p
    JOIN "TraderMerchant" tm
        ON tm."merchantId" = p."merchantId"
    JOIN "User" u
        ON u."id" = tm."traderId"
    WHERE p."direction" = 'OUT'
      AND p."status" = 'CREATED'
      AND (p."traderId" IS NULL OR p."traderId" = u."id")
      AND tm."isMerchantEnabled" = TRUE
      AND tm."isFeeOutEnabled" = TRUE
      AND COALESCE(u."balanceRub", 0) > 0
      AND u."trafficEnabled" = TRUE
      AND u."banned" = FALSE
    ORDER BY u."numericId"
"#;

const UNASSIGNED_PAYOUTS_QUERY: &str = r#"
    SELECT
        p."id",
        p."numericId",
        p."amount",
        p."bank",
        p."externalReference"
    FROM "Payout" p
    LEFT JOIN "AggregatorPayout" ap
        ON ap."payoutId" = p."id"
    WHERE p."direction" = 'OUT'
      AND p."status" = 'CREATED'
      AND p."acceptedAt" IS NULL
      AND p."traderId" IS NULL
      AND ap."payoutId" IS NULL
    ORDER BY p."createdAt"
"#;

#[derive(Debug, FromRow, Clone)]
struct TraderRecord {
    id: String,
    email: String,
    #[sqlx(rename = "numericId")]
    numeric_id: i32,
    #[sqlx(rename = "balanceRub")]
    balance_rub: Option<f64>,
    #[sqlx(rename = "frozenRub")]
    frozen_rub: Option<f64>,
    #[sqlx(rename = "payoutBalance")]
    payout_balance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Trader {
    id: String,
    email: String,
    numeric_id: i32,
    balance_rub: Option<f64>,
    frozen_rub: Option<f64>,
    payout_balance: Option<f64>,
    max_amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub(crate) struct UnassignedPayout {
    id: String,
    #[sqlx(rename = "numericId")]
    #[serde(rename = "numericId")]
    numeric_id: i32,
    amount: Option<f64>,
    bank: Option<String>,
    #[sqlx(rename = "externalReference")]
    #[serde(rename = "externalReference")]
    external_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PayoutDealListItem {
    id: String,
    #[sqlx(rename = "numericId")]
    #[serde(rename = "numericId")]
    numeric_id: i32,
    amount: f64,
    #[sqlx(rename = "amountUsdt")]
    #[serde(rename = "amountUsdt")]
    amount_usdt: f64,
    status: String,
    wallet: String,
    bank: String,
    #[sqlx(rename = "externalReference")]
    #[serde(rename = "externalReference")]
    external_reference: Option<String>,
    #[sqlx(rename = "merchantId")]
    #[serde(rename = "merchantId")]
    merchant_id: String,
    #[sqlx(rename = "traderId")]
    #[serde(rename = "traderId")]
    trader_id: Option<String>,
    #[sqlx(rename = "createdAt")]
    #[serde(rename = "createdAt")]
    created_at: NaiveDateTime,
    #[sqlx(rename = "cancelReason")]
    #[serde(rename = "cancelReason")]
    cancel_reason: Option<String>,
    #[sqlx(rename = "cancelReasonCode")]
    #[serde(rename = "cancelReasonCode")]
    cancel_reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PayoutPagination {
    total: i64,
    page: u32,
    per_page: u32,
    total_pages: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PayoutListResponse {
    items: Vec<PayoutDealListItem>,
    pagination: PayoutPagination,
}

#[derive(Debug, Clone)]
struct PayoutListData {
    items: Vec<PayoutDealListItem>,
    total: i64,
    page: u32,
    per_page: u32,
}

impl PayoutListData {
    fn into_response(self) -> PayoutListResponse {
        let total_pages = if self.total == 0 {
            0
        } else {
            ((self.total as f64) / (self.per_page as f64)).ceil() as u32
        };
        PayoutListResponse {
            items: self.items,
            pagination: PayoutPagination {
                total: self.total,
                page: self.page,
                per_page: self.per_page,
                total_pages,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PayoutListQuery {
    search: Option<String>,
    wallet: Option<String>,
    amount: Option<f64>,
    status: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
    sort: Option<String>,
    order: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortField {
    CreatedAt,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
struct PayoutListFilters {
    search: Option<String>,
    wallet: Option<String>,
    amount: Option<f64>,
    status: Option<String>,
    page: u32,
    per_page: u32,
    sort: SortField,
    order: SortOrder,
}

impl Default for PayoutListFilters {
    fn default() -> Self {
        Self {
            search: None,
            wallet: None,
            amount: None,
            status: None,
            page: 1,
            per_page: 25,
            sort: SortField::CreatedAt,
            order: SortOrder::Desc,
        }
    }
}

impl PayoutListQuery {
    fn into_filters(self) -> PayoutListFilters {
        let mut filters = PayoutListFilters::default();

        filters.page = self.page.unwrap_or(1).max(1);
        filters.per_page = self
            .per_page
            .unwrap_or(filters.per_page)
            .clamp(1, 200);

        filters.search = self
            .search
            .and_then(|value| {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            });

        filters.wallet = self
            .wallet
            .and_then(|value| {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            });

        filters.amount = self.amount.filter(|value| !value.is_nan());

        filters.status = self
            .status
            .and_then(|value| {
                let upper = value.trim().to_uppercase();
                if upper.is_empty() {
                    None
                } else {
                    Some(upper)
                }
            });

        filters.sort = match self.sort.as_deref() {
            Some("status") => SortField::Status,
            _ => SortField::CreatedAt,
        };

        filters.order = match self.order.as_deref() {
            Some(value) if value.eq_ignore_ascii_case("asc") => SortOrder::Asc,
            Some(value) if value.eq_ignore_ascii_case("desc") => SortOrder::Desc,
            _ => {
                if filters.sort == SortField::Status {
                    SortOrder::Asc
                } else {
                    SortOrder::Desc
                }
            }
        };

        filters
    }
}

#[derive(Debug, Clone, FromRow)]
struct PayoutDetails {
    id: String,
    numeric_id: i32,
    amount: f64,
    amount_usdt: f64,
    status: String,
    wallet: String,
    bank: String,
    external_reference: Option<String>,
    merchant_id: String,
    merchant_webhook_url: Option<String>,
    merchant_metadata: Option<Value>,
    proof_files: Option<Vec<String>>,
    dispute_files: Option<Vec<String>>,
    dispute_message: Option<String>,
    cancel_reason: Option<String>,
    cancel_reason_code: Option<String>,
    trader_id: Option<String>,
    merchant_api_key: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CancelPayoutResponse {
    success: bool,
    status: String,
    callback_dispatched: bool,
    callback_error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CancelPayoutRequest {
    reason: Option<String>,
    reason_code: Option<String>,
}

#[derive(Debug)]
struct CallbackDispatchResult {
    delivered: bool,
    status_code: Option<u16>,
    response_body: Option<String>,
    error: Option<String>,
    url: Option<String>,
}

impl CallbackDispatchResult {
    fn not_attempted(reason: impl Into<String>, url: Option<String>) -> Self {
        Self {
            delivered: false,
            status_code: None,
            response_body: None,
            error: Some(reason.into()),
            url,
        }
    }

    fn was_delivered(&self) -> bool {
        self.delivered
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PayoutCallbackPayload {
    event: String,
    payout: PayoutCallbackBody,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PayoutCallbackBody {
    id: String,
    bank: String,
    amount: f64,
    status: String,
    wallet: String,
    metadata: Value,
    #[serde(rename = "numericId")]
    numeric_id: i32,
    #[serde(rename = "amountUsdt")]
    amount_usdt: f64,
    #[serde(default)]
    proof_files: Vec<String>,
    #[serde(rename = "cancelReason")]
    cancel_reason: Option<String>,
    #[serde(default)]
    dispute_files: Vec<String>,
    #[serde(rename = "disputeMessage")]
    dispute_message: Option<String>,
    #[serde(rename = "cancelReasonCode")]
    cancel_reason_code: Option<String>,
    #[serde(rename = "externalReference")]
    external_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AutoDistributionConfig {
    enabled: bool,
    interval_seconds: u64,
}

impl Default for AutoDistributionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ServerEvent {
    #[serde(rename = "type")]
    event_type: String,
    message: Option<String>,
}

impl ServerEvent {
    fn new(event_type: impl Into<String>, message: Option<String>) -> Self {
        Self {
            event_type: event_type.into(),
            message,
        }
    }

    fn payouts_updated(source: &str) -> Self {
        Self::new("payouts-updated", Some(format!("source={}", source)))
    }

    fn traders_updated() -> Self {
        Self::new("traders-updated", None)
    }

    fn settings_updated() -> Self {
        Self::new("settings-updated", None)
    }

    fn limits_updated() -> Self {
        Self::new("limits-updated", None)
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    pool: PgPool,
    auto_config: Arc<RwLock<AutoDistributionConfig>>,
    auto_config_tx: watch::Sender<AutoDistributionConfig>,
    limits: Arc<RwLock<HashMap<String, f64>>>,
    round_robin: Arc<Mutex<usize>>,
    event_tx: broadcast::Sender<ServerEvent>,
    http_client: Client,
}

type ApiResult<T> = Result<T, (StatusCode, String)>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL environment variable is not set")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    let initial_config = AutoDistributionConfig::default();
    let (config_tx, config_rx) = watch::channel(initial_config.clone());
    let (event_tx, _) = broadcast::channel(100);
    let http_client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .context("Failed to build HTTP client")?;

    let state = AppState {
        pool: pool.clone(),
        auto_config: Arc::new(RwLock::new(initial_config.clone())),
        auto_config_tx: config_tx.clone(),
        limits: Arc::new(RwLock::new(HashMap::new())),
        round_robin: Arc::new(Mutex::new(0)),
        event_tx: event_tx.clone(),
        http_client,
    };

    tokio::spawn(auto_distribution_worker(
        pool.clone(),
        config_rx,
        Arc::clone(&state.limits),
        Arc::clone(&state.round_robin),
        event_tx.clone(),
    ));

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/events", get(events))
        .route("/api/traders", get(get_traders))
        .route("/api/payouts", get(get_unassigned_payouts))
        .route("/api/deals", get(get_all_payouts))
        .route("/api/payouts/:id/assign", post(assign_payout))
        .route("/api/payouts/:id/cancel", post(cancel_payout))
        .route(
            "/api/settings/auto-distribution",
            get(get_auto_settings).post(update_auto_settings),
        )
        .route("/api/traders/:id/limit", post(update_trader_limit))
        .with_state(state);

    let addr: SocketAddr = ([0, 0, 0, 0], 5555).into();
    println!("Server running on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind TCP listener")?;

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}

async fn serve_index(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let traders = load_traders_with_limits(&state)
        .await
        .map_err(internal_error)?;
    let payouts = fetch_unassigned_payouts(&state.pool)
        .await
        .map_err(internal_error)?;
    let default_filters = PayoutListFilters::default();
    let deals = fetch_payouts_page(&state.pool, &default_filters)
        .await
        .map_err(internal_error)?
        .into_response();
    let settings = read_auto_settings(&state).await;
    let snapshot = frontend::DashboardSnapshot {
        traders,
        payouts,
        deals,
        settings,
    };
    Ok(Html(frontend::render_dashboard_page(snapshot)))
}

async fn events(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<SseEvent, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => match SseEvent::default().json_data(event) {
            Ok(evt) => Some(Ok(evt)),
            Err(err) => {
                eprintln!("Failed to serialize SSE event: {err}");
                None
            }
        },
        Err(err) => {
            eprintln!("SSE subscriber lagged: {err}");
            None
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn get_traders(State(state): State<AppState>) -> ApiResult<Json<Vec<Trader>>> {
    let traders = load_traders_with_limits(&state)
        .await
        .map_err(internal_error)?;
    Ok(Json(traders))
}

async fn get_unassigned_payouts(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<UnassignedPayout>>> {
    fetch_unassigned_payouts(&state.pool)
        .await
        .map(Json)
        .map_err(internal_error)
}

async fn get_all_payouts(
    Query(params): Query<PayoutListQuery>,
    State(state): State<AppState>,
) -> ApiResult<Json<PayoutListResponse>> {
    let filters = params.into_filters();
    fetch_payouts_page(&state.pool, &filters)
        .await
        .map(|data| Json(data.into_response()))
        .map_err(internal_error)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssignPayoutRequest {
    trader_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssignPayoutResponse {
    success: bool,
}

async fn assign_payout(
    Path(payout_id): Path<String>,
   State(state): State<AppState>,
    Json(request): Json<AssignPayoutRequest>,
) -> ApiResult<Json<AssignPayoutResponse>> {
    assign_payout_internal(&state, &payout_id, &request.trader_id).await?;
    Ok(Json(AssignPayoutResponse { success: true }))
}

async fn cancel_payout(
    Path(payout_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<CancelPayoutRequest>,
) -> ApiResult<Json<CancelPayoutResponse>> {
    let reason = request
        .reason
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    let reason_code = request
        .reason_code
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    let mut payout = sqlx::query_as::<_, PayoutDetails>(
        r#"
        SELECT
            p."id",
            p."numericId" AS "numeric_id",
            p."amount",
            p."amountUsdt" AS "amount_usdt",
            p."status"::text AS "status",
            p."wallet",
            p."bank",
            p."externalReference" AS "external_reference",
            p."merchantId" AS "merchant_id",
            p."merchantWebhookUrl" AS "merchant_webhook_url",
            p."merchantMetadata" AS "merchant_metadata",
            p."proofFiles" AS "proof_files",
            p."disputeFiles" AS "dispute_files",
            p."disputeMessage" AS "dispute_message",
            p."cancelReason" AS "cancel_reason",
            p."cancelReasonCode" AS "cancel_reason_code",
            p."traderId" AS "trader_id",
            m."apiKeyPublic" AS "merchant_api_key"
        FROM "Payout" p
        LEFT JOIN "Merchant" m
            ON m."id" = p."merchantId"
        WHERE p."id" = $1
        FOR UPDATE
        "#,
    )
    .bind(&payout_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;

    let mut payout = match payout {
        Some(payout) => payout,
        None => {
            tx.rollback().await.ok();
            return Err((StatusCode::NOT_FOUND, "Payout not found".to_string()));
        }
    };

    match payout.status.as_str() {
        "CANCELLED" => {
            tx.rollback().await.ok();
            return Err((
                StatusCode::BAD_REQUEST,
                "Payout is already cancelled".to_string(),
            ));
        }
        "COMPLETED" | "SUCCESS" | "FAILED" => {
            tx.rollback().await.ok();
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Payout with status {} cannot be cancelled", payout.status),
            ));
        }
        _ => {}
    }

    let reason_ref = reason.as_deref();
    let reason_code_ref = reason_code.as_deref();

    let update_result = sqlx::query!(
        r#"
        UPDATE "Payout"
        SET "status" = 'CANCELLED',
            "cancelledAt" = CURRENT_TIMESTAMP,
            "cancelReason" = COALESCE($2, "cancelReason"),
            "cancelReasonCode" = COALESCE($3, "cancelReasonCode")
        WHERE "id" = $1
        "#,
        payout_id,
        reason_ref,
        reason_code_ref
    )
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    if update_result.rows_affected() == 0 {
        tx.rollback().await.ok();
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to cancel payout".to_string(),
        ));
    }

    if let Some(reason_value) = reason.clone() {
        payout.cancel_reason = Some(reason_value);
    }

    if let Some(code_value) = reason_code.clone() {
        payout.cancel_reason_code = Some(code_value);
    }

    payout.status = "CANCELLED".to_string();

    tx.commit().await.map_err(internal_error)?;

    let payload = build_cancel_callback_payload(&payout);
    let callback_result = dispatch_payout_callback(&state, &payout, &payload)
        .await
        .map_err(internal_error)?;

    let _ = state
        .event_tx
        .send(ServerEvent::payouts_updated("manual-cancel"));

    Ok(Json(CancelPayoutResponse {
        success: true,
        status: "CANCELED".to_string(),
        callback_dispatched: callback_result.was_delivered(),
        callback_error: callback_result.error.clone(),
    }))
}

fn build_cancel_callback_payload(payout: &PayoutDetails) -> PayoutCallbackPayload {
    let metadata = payout
        .merchant_metadata
        .clone()
        .unwrap_or_else(|| Value::Object(Default::default()));
    let proof_files = payout.proof_files.clone().unwrap_or_default();
    let dispute_files = payout.dispute_files.clone().unwrap_or_default();

    PayoutCallbackPayload {
        event: "CANCELED".to_string(),
        payout: PayoutCallbackBody {
            id: payout.id.clone(),
            bank: payout.bank.clone(),
            amount: payout.amount,
            status: "CANCELED".to_string(),
            wallet: payout.wallet.clone(),
            metadata,
            numeric_id: payout.numeric_id,
            amount_usdt: payout.amount_usdt,
            proof_files,
            cancel_reason: payout.cancel_reason.clone(),
            dispute_files,
            dispute_message: payout.dispute_message.clone(),
            cancel_reason_code: payout.cancel_reason_code.clone(),
            external_reference: payout.external_reference.clone(),
        },
    }
}

async fn dispatch_payout_callback(
    state: &AppState,
    payout: &PayoutDetails,
    payload: &PayoutCallbackPayload,
) -> Result<CallbackDispatchResult> {
    let webhook_url = payout
        .merchant_webhook_url
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let webhook_url = match webhook_url {
        Some(url) => url,
        None => {
            let result = CallbackDispatchResult::not_attempted(
                "Merchant webhook URL is not configured",
                Some("(missing-webhook-url)".to_string()),
            );
            log_payout_callback(
                &state.pool,
                payout,
                "(missing-webhook-url)",
                payload,
                &result,
            )
            .await?;
            return Ok(result);
        }
    };

    let api_key = payout
        .merchant_api_key
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let api_key = match api_key {
        Some(key) => key,
        None => {
            let result = CallbackDispatchResult::not_attempted(
                "Merchant API key is not configured",
                Some(webhook_url.clone()),
            );
            log_payout_callback(&state.pool, payout, &webhook_url, payload, &result).await?;
            return Ok(result);
        }
    };

    let response = state
        .http_client
        .post(&webhook_url)
        .header("x-merchant-api-key", api_key)
        .json(payload)
        .send()
        .await;

    let dispatch_result = match response {
        Ok(resp) => {
            let status = resp.status();
            let status_code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            CallbackDispatchResult {
                delivered: status.is_success(),
                status_code: Some(status_code),
                response_body: if body.is_empty() { None } else { Some(body) },
                error: if status.is_success() {
                    None
                } else {
                    Some(format!("HTTP {}", status_code))
                },
                url: Some(webhook_url.clone()),
            }
        }
        Err(err) => CallbackDispatchResult {
            delivered: false,
            status_code: None,
            response_body: None,
            error: Some(err.to_string()),
            url: Some(webhook_url.clone()),
        },
    };

    log_payout_callback(&state.pool, payout, &webhook_url, payload, &dispatch_result).await?;
    Ok(dispatch_result)
}

async fn log_payout_callback(
    pool: &PgPool,
    payout: &PayoutDetails,
    url: &str,
    payload: &PayoutCallbackPayload,
    result: &CallbackDispatchResult,
) -> Result<()> {
    let payload_value =
        serde_json::to_value(payload).context("Failed to serialize callback payload")?;

    let response_text = result.response_body.as_deref();
    let error_text = result.error.as_deref();
    let status_code = result.status_code.map(|code| i32::from(code));

    sqlx::query!(
        r#"
        INSERT INTO "PayoutCallbackHistory"
            ("id", "payoutId", "url", "payload", "response", "statusCode", "error")
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        Uuid::new_v4().to_string(),
        payout.id,
        url,
        payload_value,
        response_text,
        status_code,
        error_text
    )
    .execute(pool)
    .await
    .context("Failed to record payout callback log")?;

    Ok(())
}

async fn get_auto_settings(
    State(state): State<AppState>,
) -> ApiResult<Json<AutoDistributionConfig>> {
    Ok(Json(read_auto_settings(&state).await))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateAutoSettingsRequest {
    enabled: bool,
    interval_seconds: u64,
}

async fn update_auto_settings(
    State(state): State<AppState>,
    Json(request): Json<UpdateAutoSettingsRequest>,
) -> ApiResult<Json<AutoDistributionConfig>> {
    let updated =
        update_auto_settings_internal(&state, request.enabled, request.interval_seconds).await?;
    Ok(Json(updated))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateLimitRequest {
    max_amount: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateLimitResponse {
    trader_id: String,
    max_amount: Option<f64>,
}

async fn update_trader_limit(
    Path(trader_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<UpdateLimitRequest>,
) -> ApiResult<Json<UpdateLimitResponse>> {
    let sanitized = update_trader_limit_internal(&state, &trader_id, request.max_amount).await?;
    Ok(Json(UpdateLimitResponse {
        trader_id,
        max_amount: sanitized,
    }))
}

async fn fetch_traders(pool: &PgPool) -> Result<Vec<TraderRecord>> {
    sqlx::query_as::<_, TraderRecord>(ELIGIBLE_TRADERS_QUERY)
        .fetch_all(pool)
        .await
        .context("Failed to fetch eligible traders")
}

async fn fetch_unassigned_payouts(pool: &PgPool) -> Result<Vec<UnassignedPayout>> {
    sqlx::query_as::<_, UnassignedPayout>(UNASSIGNED_PAYOUTS_QUERY)
        .fetch_all(pool)
        .await
        .context("Failed to fetch unassigned payouts")
}

async fn fetch_payouts_page(pool: &PgPool, filters: &PayoutListFilters) -> Result<PayoutListData> {
    let mut count_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT COUNT(*)::bigint AS total FROM "Payout" p WHERE p."direction" = 'OUT'"#,
    );
    apply_payout_filters(&mut count_builder, filters);

    let total: i64 = count_builder
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .context("Failed to count payouts")?;

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT
            p."id",
            p."numericId",
            p."amount",
            p."amountUsdt",
            p."status"::text AS "status",
            p."wallet",
            p."bank",
            p."externalReference",
            p."merchantId",
            p."traderId",
            p."createdAt",
            p."cancelReason",
            p."cancelReasonCode"
        FROM "Payout" p
        WHERE p."direction" = 'OUT'
        "#,
    );

    apply_payout_filters(&mut builder, filters);
    apply_payout_sort(&mut builder, filters);

    let per_page = filters.per_page.min(200);
    let offset = ((filters.page.saturating_sub(1)) as i64) * per_page as i64;

    builder.push(" LIMIT ").push_bind(per_page as i64);
    builder.push(" OFFSET ").push_bind(offset.max(0));

    let items = builder
        .build_query_as::<PayoutDealListItem>()
        .fetch_all(pool)
        .await
        .context("Failed to fetch payouts list")?;

    Ok(PayoutListData {
        items,
        total,
        page: filters.page,
        per_page,
    })
}

fn apply_payout_filters(builder: &mut QueryBuilder<Postgres>, filters: &PayoutListFilters) {
    if let Some(search) = filters.search.as_ref() {
        let like = format!("%{}%", search);
        builder.push(" AND (");
        builder
            .push("p.\"id\" ILIKE ")
            .push_bind(like.clone());
        builder
            .push(" OR p.\"externalReference\" ILIKE ")
            .push_bind(like.clone());
        builder
            .push(" OR p.\"numericId\"::text ILIKE ")
            .push_bind(like);
        builder.push(")");
    }

    if let Some(wallet) = filters.wallet.as_ref() {
        let like = format!("%{}%", wallet);
        builder
            .push(" AND p.\"wallet\" ILIKE ")
            .push_bind(like);
    }

    if let Some(amount) = filters.amount {
        builder.push(" AND p.\"amount\" = ").push_bind(amount);
    }

    if let Some(status) = filters.status.as_ref() {
        builder.push(" AND p.\"status\" = ").push_bind(status.clone());
        builder.push("::\"PayoutStatus\"");
    }
}

fn apply_payout_sort(builder: &mut QueryBuilder<Postgres>, filters: &PayoutListFilters) {
    match filters.sort {
        SortField::Status => {
            builder.push(" ORDER BY p.\"status\" ");
            match filters.order {
                SortOrder::Asc => {
                    builder.push("ASC");
                }
                SortOrder::Desc => {
                    builder.push("DESC");
                }
            }
            builder.push(", p.\"createdAt\" DESC");
        }
        SortField::CreatedAt => {
            builder.push(" ORDER BY p.\"createdAt\" ");
            match filters.order {
                SortOrder::Asc => {
                    builder.push("ASC");
                }
                SortOrder::Desc => {
                    builder.push("DESC");
                }
            }
        }
    }
}

async fn auto_distribution_worker(
    pool: PgPool,
    mut config_rx: watch::Receiver<AutoDistributionConfig>,
    limits: Arc<RwLock<HashMap<String, f64>>>,
    round_robin: Arc<Mutex<usize>>,
    event_tx: broadcast::Sender<ServerEvent>,
) {
    let mut current = config_rx.borrow().clone();
    let mut interval = build_interval(current.interval_seconds);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if current.enabled {
                    if let Err(err) = distribute_payouts_evenly(
                        &pool,
                        Arc::clone(&limits),
                        Arc::clone(&round_robin),
                        &event_tx,
                    ).await {
                        eprintln!("[auto] Distribution error: {err:?}");
                    }
                }
            }
            changed = config_rx.changed() => {
                if changed.is_err() {
                    break;
                }
                current = config_rx.borrow().clone();
                interval = build_interval(current.interval_seconds);
                println!(
                    "[settings] Updated auto distribution config: enabled={}, interval={}s",
                    current.enabled,
                    current.interval_seconds
                );
            }
        }
    }
}

fn build_interval(seconds: u64) -> time::Interval {
    let mut interval = time::interval(Duration::from_secs(seconds.max(1)));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    interval
}

async fn distribute_payouts_evenly(
    pool: &PgPool,
    limits: Arc<RwLock<HashMap<String, f64>>>,
    round_robin: Arc<Mutex<usize>>,
    event_tx: &broadcast::Sender<ServerEvent>,
) -> Result<()> {
    let traders = fetch_traders(pool).await?;
    if traders.is_empty() {
        println!("[auto] No eligible traders available. Skipping distribution.");
        return Ok(());
    }

    let payouts = fetch_unassigned_payouts(pool).await?;
    if payouts.is_empty() {
        println!("[auto] No unassigned payouts to distribute.");
        return Ok(());
    }

    let limits_snapshot = {
        let limits_guard = limits.read().await;
        limits_guard.clone()
    };

    let mut round_robin_guard = round_robin.lock().await;
    let mut current_index = *round_robin_guard;

    let mut assignments: Vec<(String, String, i32, i32)> = Vec::new();

    for payout in &payouts {
        let amount = payout.amount.unwrap_or_default();
        if amount <= 0.0 {
            continue;
        }

        let mut selected: Option<(usize, &TraderRecord)> = None;

        for offset in 0..traders.len() {
            let idx = (current_index + offset) % traders.len();
            let trader = &traders[idx];
            let allowed = limits_snapshot
                .get(&trader.id)
                .copied()
                .map_or(true, |max| amount <= max);

            if allowed {
                selected = Some((idx, trader));
                current_index = (idx + 1) % traders.len();
                break;
            }
        }

        if let Some((_, trader)) = selected {
            assignments.push((
                payout.id.clone(),
                trader.id.clone(),
                payout.numeric_id,
                trader.numeric_id,
            ));
        } else {
            println!(
                "[auto] Skipped payout {} (amount {:.2}) - no trader accepts this amount",
                payout.id, amount
            );
        }
    }

    if assignments.is_empty() {
        println!("[auto] No assignments created in this cycle.");
        *round_robin_guard = current_index;
        return Ok(());
    }

    let mut tx = pool.begin().await?;
    let mut applied = 0u64;

    for (payout_id, trader_id, payout_numeric, trader_numeric) in &assignments {
        let result = sqlx::query(
            r#"
            UPDATE "Payout"
            SET "traderId" = $1,
                "acceptanceTime" = 40
            WHERE "id" = $2
              AND "traderId" IS NULL
              AND "direction" = 'OUT'
              AND "status" = 'CREATED'
              AND "acceptedAt" IS NULL
              AND NOT EXISTS (
                  SELECT 1
                  FROM "AggregatorPayout" ap
                  WHERE ap."payoutId" = "Payout"."id"
              )
            "#,
        )
        .bind(trader_id)
        .bind(payout_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() > 0 {
            applied += 1;
            println!(
                "[auto] Assigned payout {} (numericId {}) to trader {} (numericId {})",
                payout_id, payout_numeric, trader_id, trader_numeric
            );
        }
    }

    tx.commit().await?;
    *round_robin_guard = current_index;
    drop(round_robin_guard);

    if applied > 0 {
        let _ = event_tx.send(ServerEvent::payouts_updated("auto"));
        println!("[auto] Distribution cycle completed with {applied} assignments.");
    } else {
        println!("[auto] Distribution cycle completed without changes.");
    }

    Ok(())
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::fmt::Display,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub(crate) async fn load_traders_with_limits(state: &AppState) -> Result<Vec<Trader>> {
    let records = fetch_traders(&state.pool).await?;
    let limits = state.limits.read().await;

    let traders = records
        .into_iter()
        .map(|record| Trader {
            max_amount: limits.get(&record.id).copied(),
            id: record.id,
            email: record.email,
            numeric_id: record.numeric_id,
            balance_rub: record.balance_rub,
            frozen_rub: record.frozen_rub,
            payout_balance: record.payout_balance,
        })
        .collect();

    Ok(traders)
}

pub(crate) async fn read_auto_settings(state: &AppState) -> AutoDistributionConfig {
    state.auto_config.read().await.clone()
}

pub(crate) async fn assign_payout_internal(
    state: &AppState,
    payout_id: &str,
    trader_id: &str,
) -> ApiResult<()> {
    if trader_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Trader ID is required".to_string()));
    }

    let mut conn = state.pool.acquire().await.map_err(internal_error)?;

    let result = sqlx::query(
        r#"
        UPDATE "Payout"
        SET "traderId" = $1,
            "acceptanceTime" = 40
        WHERE "id" = $2
          AND "direction" = 'OUT'
          AND "status" = 'CREATED'
          AND "acceptedAt" IS NULL
          AND "traderId" IS NULL
          AND NOT EXISTS (
              SELECT 1
              FROM "AggregatorPayout" ap
              WHERE ap."payoutId" = "Payout"."id"
          )
        "#,
    )
    .bind(trader_id)
    .bind(payout_id)
    .execute(&mut *conn)
    .await
    .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Payout is not eligible for assignment".to_string(),
        ));
    }

    println!("[manual] Assigned payout {payout_id} to trader {trader_id}");

    let _ = state.event_tx.send(ServerEvent::payouts_updated("manual"));

    Ok(())
}

pub(crate) async fn update_auto_settings_internal(
    state: &AppState,
    enabled: bool,
    interval_seconds: u64,
) -> ApiResult<AutoDistributionConfig> {
    let interval = interval_seconds.max(1);

    let new_config = AutoDistributionConfig {
        enabled,
        interval_seconds: interval,
    };

    {
        let mut cfg = state.auto_config.write().await;
        *cfg = new_config.clone();
    }

    state
        .auto_config_tx
        .send(new_config.clone())
        .map_err(|err| internal_error(err))?;

    println!(
        "[settings] Auto distribution {} with interval {} seconds",
        if new_config.enabled {
            "enabled"
        } else {
            "disabled"
        },
        new_config.interval_seconds
    );

    let _ = state.event_tx.send(ServerEvent::settings_updated());

    Ok(new_config)
}

pub(crate) async fn update_trader_limit_internal(
    state: &AppState,
    trader_id: &str,
    max_amount: Option<f64>,
) -> ApiResult<Option<f64>> {
    let sanitized = max_amount.filter(|value| *value > 0.0);

    {
        let mut limits = state.limits.write().await;
        if let Some(value) = sanitized {
            limits.insert(trader_id.to_string(), value);
        } else {
            limits.remove(trader_id);
        }
    }

    println!(
        "[settings] Updated trader limit: trader={} limit={:?}",
        trader_id, sanitized
    );

    let _ = state.event_tx.send(ServerEvent::limits_updated());

    Ok(sanitized)
}
