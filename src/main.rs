use std::{
    collections::HashMap, convert::Infallible, env, net::SocketAddr, sync::Arc, time::Duration,
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, sse::Event as SseEvent, sse::KeepAlive, sse::Sse},
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, postgres::PgPoolOptions};
use tokio::sync::{Mutex, RwLock, broadcast, watch};
use tokio::time::{self, MissedTickBehavior};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

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
      AND tm."feeOut" IS NOT NULL
      AND tm."feeOut" > 0
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

    let state = AppState {
        pool: pool.clone(),
        auto_config: Arc::new(RwLock::new(initial_config.clone())),
        auto_config_tx: config_tx.clone(),
        limits: Arc::new(RwLock::new(HashMap::new())),
        round_robin: Arc::new(Mutex::new(0)),
        event_tx: event_tx.clone(),
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
        .route("/api/payouts/:id/assign", post(assign_payout))
        .route(
            "/api/settings/auto-distribution",
            get(get_auto_settings).post(update_auto_settings),
        )
        .route("/api/traders/:id/limit", post(update_trader_limit))
        .with_state(state);

    let addr: SocketAddr = ([0, 0, 0, 0], 3000).into();
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
    let settings = read_auto_settings(&state).await;
    let snapshot = frontend::DashboardSnapshot {
        traders,
        payouts,
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
