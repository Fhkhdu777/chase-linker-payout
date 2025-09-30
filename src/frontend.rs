use crate::{AutoDistributionConfig, Trader, UnassignedPayout};
use leptos::*;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub(crate) struct DashboardSnapshot {
    pub traders: Vec<Trader>,
    pub payouts: Vec<UnassignedPayout>,
    pub settings: AutoDistributionConfig,
}

const STYLES: &str = r#"
:root {
    --bg-primary: #0f172a;
    --bg-secondary: rgba(15, 23, 42, 0.72);
    --bg-panel: rgba(15, 23, 42, 0.82);
    --accent: #38bdf8;
    --accent-strong: #0ea5e9;
    --border-light: rgba(148, 163, 184, 0.35);
    --text-primary: #f8fafc;
    --text-secondary: #cbd5f5;
    --text-muted: #94a3b8;
    --success: #4ade80;
    --error: #f87171;
    --warning: #fbbf24;
    --info: #38bdf8;
}
* {
    box-sizing: border-box;
}
body {
    margin: 0;
    min-height: 100vh;
    background: radial-gradient(circle at top, #1e293b, var(--bg-primary));
    color: var(--text-primary);
    font-family: 'Inter', 'Roboto', 'Segoe UI', sans-serif;
    display: flex;
    flex-direction: column;
}
.top-bar {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 32px 40px 16px 40px;
    gap: 24px;
}
.top-bar h1 {
    font-size: 28px;
    margin: 0 0 8px 0;
}
.top-bar p {
    margin: 0;
    color: var(--text-muted);
}
.status-block {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
}
.status-label {
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-muted);
}
.status-value {
    font-weight: 600;
    font-size: 16px;
}
main {
    flex: 1;
    padding: 0 40px 40px 40px;
    display: flex;
    flex-direction: column;
    gap: 24px;
}
.metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 20px;
}
.metric-card {
    background: var(--bg-panel);
    border: 1px solid var(--border-light);
    border-radius: 16px;
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    box-shadow: 0 18px 35px rgba(15, 23, 42, 0.32);
}
.metric-label {
    text-transform: uppercase;
    font-size: 11px;
    letter-spacing: 0.08em;
    color: var(--text-muted);
}
.metric-value {
    font-size: 30px;
    font-weight: 700;
}
.metric-sub {
    color: var(--text-secondary);
    font-size: 13px;
}
.status-banner {
    display: none;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px;
    border-radius: 12px;
    border: 1px solid transparent;
    font-size: 14px;
    font-weight: 500;
}
.status-banner.visible {
    display: flex;
}
.status-banner[data-type='info'] {
    background: rgba(56, 189, 248, 0.12);
    border-color: rgba(56, 189, 248, 0.35);
    color: var(--info);
}
.status-banner[data-type='success'] {
    background: rgba(74, 222, 128, 0.12);
    border-color: rgba(74, 222, 128, 0.35);
    color: var(--success);
}
.status-banner[data-type='warning'] {
    background: rgba(251, 191, 36, 0.12);
    border-color: rgba(251, 191, 36, 0.35);
    color: var(--warning);
}
.status-banner[data-type='error'] {
    background: rgba(248, 113, 113, 0.12);
    border-color: rgba(248, 113, 113, 0.35);
    color: var(--error);
}
.panel {
    background: var(--bg-panel);
    border: 1px solid var(--border-light);
    border-radius: 18px;
    padding: 24px 28px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-shadow: 0 18px 35px rgba(15, 23, 42, 0.35);
}
.panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
}
.panel-header h2 {
    margin: 0;
    font-size: 20px;
}
.panel-subtitle {
    color: var(--text-muted);
    font-size: 14px;
}
.badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 6px 12px;
    border-radius: 999px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    border: 1px solid var(--border-light);
    color: var(--text-secondary);
}
.badge[data-state='on'] {
    background: rgba(74, 222, 128, 0.12);
    border-color: rgba(74, 222, 128, 0.45);
    color: var(--success);
}
.badge[data-state='off'] {
    background: rgba(148, 163, 184, 0.12);
    border-color: rgba(148, 163, 184, 0.35);
    color: var(--text-muted);
}
.controls-row {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    align-items: center;
}
button {
    background: linear-gradient(135deg, var(--accent), var(--accent-strong));
    border: none;
    color: #0b1120;
    font-weight: 600;
    padding: 10px 18px;
    border-radius: 12px;
    cursor: pointer;
    transition: transform 0.1s ease, box-shadow 0.1s ease;
    box-shadow: 0 12px 22px rgba(14, 165, 233, 0.25);
}
button:hover {
    transform: translateY(-1px);
    box-shadow: 0 16px 28px rgba(14, 165, 233, 0.35);
}
button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    box-shadow: none;
}
input[type='number'],
select {
    background: rgba(15, 23, 42, 0.65);
    border: 1px solid var(--border-light);
    border-radius: 12px;
    color: var(--text-primary);
    padding: 9px 12px;
    font-size: 14px;
    min-width: 120px;
}
input[type='number']:focus,
select:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(56, 189, 248, 0.25);
}
.table-wrapper {
    overflow-x: auto;
    border-radius: 12px;
}
table {
    width: 100%;
    border-collapse: collapse;
}
thead {
    background: rgba(148, 163, 184, 0.12);
}
th, td {
    padding: 12px 14px;
    text-align: left;
    border-bottom: 1px solid rgba(148, 163, 184, 0.18);
    font-size: 14px;
    color: var(--text-secondary);
}
th {
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-size: 12px;
    color: var(--text-muted);
}
tbody tr:hover {
    background: rgba(59, 130, 246, 0.08);
}
tbody tr:last-child td {
    border-bottom: none;
}
.empty {
    text-align: center;
    color: var(--text-muted);
    font-style: italic;
}
.limit-controls {
    display: flex;
    gap: 10px;
    align-items: center;
}
.limit-controls input {
    max-width: 140px;
}
.assign-controls {
    display: flex;
    gap: 10px;
    align-items: center;
}
@media (max-width: 960px) {
    .top-bar {
        flex-direction: column;
        align-items: flex-start;
        padding: 28px 24px 12px 24px;
    }
    main {
        padding: 0 24px 24px 24px;
    }
    .panel {
        padding: 20px;
    }
    th, td {
        white-space: nowrap;
    }
}
@media (max-width: 640px) {
    .controls-row {
        flex-direction: column;
        align-items: stretch;
    }
    button {
        width: 100%;
    }
    .assign-controls {
        flex-direction: column;
        align-items: stretch;
    }
}
"#;

const DASHBOARD_SCRIPT: &str = r#"
(() => {
    const statusBar = document.getElementById('global-status');
    const lastUpdatedEl = document.getElementById('last-updated');
    const metrics = {
        traders: document.getElementById('metric-traders'),
        payouts: document.getElementById('metric-payouts'),
        payoutSum: document.getElementById('metric-payout-sum'),
    };
    const autoBadge = document.getElementById('auto-status-badge');
    const settingsDescription = document.getElementById('settings-description');

    let currentTraders = [];
    let currentPayouts = [];
    let isLoading = false;
    let reloadScheduled = false;

    function setStatus(type, message) {
        if (!statusBar) {
            return;
        }
        statusBar.textContent = message;
        statusBar.setAttribute('data-type', type);
        statusBar.classList.add('visible');
        if (type !== 'error') {
            setTimeout(() => {
                statusBar.classList.remove('visible');
            }, 3500);
        }
    }

    function markUpdated() {
        if (!lastUpdatedEl) {
            return;
        }
        const now = new Date();
        lastUpdatedEl.textContent = now.toLocaleString('ru-RU');
    }

    function formatAmount(value) {
        if (value === null || value === undefined) {
            return '-';
        }
        const num = Number(value);
        if (Number.isNaN(num)) {
            return '-';
        }
        return num.toLocaleString('ru-RU', {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2,
        });
    }

    function updateMetrics(traders, payouts) {
        if (metrics.traders) {
            metrics.traders.textContent = traders.length.toString();
        }
        if (metrics.payouts) {
            metrics.payouts.textContent = payouts.length.toString();
        }
        if (metrics.payoutSum) {
            const total = payouts.reduce((acc, payout) => acc + Number(payout.amount ?? 0), 0);
            metrics.payoutSum.textContent = formatAmount(total);
        }
    }

    function renderEmpty(tbody, colspan, message) {
        if (!tbody) {
            return;
        }
        tbody.innerHTML = `<tr><td class="empty" colspan="${colspan}">${message}</td></tr>`;
    }

    async function fetchJson(url, options) {
        const response = await fetch(url, options);
        if (!response.ok) {
            const text = await response.text();
            throw new Error(text || response.statusText);
        }
        if (response.status === 204) {
            return null;
        }
        return response.json();
    }

    function renderTraders(traders) {
        currentTraders = Array.isArray(traders) ? traders : [];
        const tbody = document.querySelector('#traders-table tbody');
        if (!tbody) {
            return;
        }
        if (!currentTraders.length) {
            renderEmpty(tbody, 6, 'Нет подходящих трейдеров');
            return;
        }
        tbody.innerHTML = currentTraders.map(trader => {
            const balance = formatAmount(trader.balanceRub);
            const frozen = formatAmount(trader.frozenRub);
            const payoutBalance = formatAmount(trader.payoutBalance);
            const limitValue = trader.maxAmount === null || trader.maxAmount === undefined
                ? ''
                : Number(trader.maxAmount).toFixed(2);
            return `
                <tr>
                    <td>${trader.numericId}</td>
                    <td>${trader.email}</td>
                    <td>${balance}</td>
                    <td>${frozen}</td>
                    <td>${payoutBalance}</td>
                    <td>
                        <div class="limit-controls">
                            <input type="number" min="0" step="0.01" value="${limitValue}" id="limit-input-${trader.id}" placeholder="Без лимита" />
                            <button class="save-limit" data-trader-id="${trader.id}">Сохранить</button>
                        </div>
                    </td>
                </tr>
            `;
        }).join('');

        tbody.querySelectorAll('.save-limit').forEach(button => {
            button.addEventListener('click', async (event) => {
                const traderId = event.currentTarget.getAttribute('data-trader-id');
                await saveTraderLimit(traderId);
            });
        });
    }

    function renderPayouts(payouts) {
        currentPayouts = Array.isArray(payouts) ? payouts : [];
        const tbody = document.querySelector('#payouts-table tbody');
        if (!tbody) {
            return;
        }
        if (!currentPayouts.length) {
            renderEmpty(tbody, 5, 'Нет нераспределенных выплат');
            return;
        }

        const traderOptions = currentTraders.map(trader => `
            <option value="${trader.id}">
                ${trader.email} (ID: ${trader.numericId})
            </option>
        `).join('');

        tbody.innerHTML = currentPayouts.map(payout => {
            const amount = formatAmount(payout.amount);
            const bank = payout.bank ?? '-';
            const external = payout.externalReference ?? '-';
            return `
                <tr>
                    <td>${payout.numericId}</td>
                    <td>${amount}</td>
                    <td>${bank}</td>
                    <td>${external}</td>
                    <td>
                        <div class="assign-controls">
                            <select id="assign-select-${payout.id}">
                                <option value="">Выберите трейдера</option>
                                ${traderOptions}
                            </select>
                            <button class="assign-button" data-payout-id="${payout.id}">Привязать</button>
                        </div>
                    </td>
                </tr>
            `;
        }).join('');

        tbody.querySelectorAll('.assign-button').forEach(button => {
            button.addEventListener('click', async (event) => {
                const payoutId = event.currentTarget.getAttribute('data-payout-id');
                await assignPayout(payoutId);
            });
        });
    }

    function renderSettings(settings) {
        const checkbox = document.getElementById('auto-enabled');
        const intervalInput = document.getElementById('auto-interval');
        const enabled = Boolean(settings?.enabled);
        const interval = Number(settings?.intervalSeconds ?? 30) || 30;

        if (checkbox) {
            checkbox.checked = enabled;
        }
        if (intervalInput) {
            intervalInput.value = interval;
        }
        if (autoBadge) {
            autoBadge.textContent = enabled ? 'Активно' : 'Выключено';
            autoBadge.setAttribute('data-state', enabled ? 'on' : 'off');
        }
        if (settingsDescription) {
            settingsDescription.textContent = enabled
                ? `Автораспределение выполняется каждые ${interval} секунд.`
                : 'Автораспределение выключено.';
        }
    }

    async function loadData(showStatus = true) {
        if (isLoading) {
            return;
        }
        isLoading = true;
        try {
            if (showStatus) {
                setStatus('info', 'Обновляем данные...');
            }
            const [traders, payouts, settings] = await Promise.all([
                fetchJson('/api/traders'),
                fetchJson('/api/payouts'),
                fetchJson('/api/settings/auto-distribution'),
            ]);
            renderTraders(traders);
            renderPayouts(payouts);
            renderSettings(settings);
            updateMetrics(traders, payouts);
            markUpdated();
            if (showStatus) {
                setStatus('success', 'Данные обновлены');
            }
        } catch (error) {
            console.error('Ошибка при загрузке данных:', error);
            const tradersBody = document.querySelector('#traders-table tbody');
            const payoutsBody = document.querySelector('#payouts-table tbody');
            renderEmpty(tradersBody, 6, 'Ошибка загрузки трейдеров');
            renderEmpty(payoutsBody, 5, 'Ошибка загрузки выплат');
            setStatus('error', 'Не удалось загрузить данные: ' + error.message);
        } finally {
            isLoading = false;
        }
    }

    async function assignPayout(payoutId) {
        const select = document.getElementById(`assign-select-${payoutId}`);
        const traderId = select?.value;

        if (!traderId) {
            setStatus('warning', 'Выберите трейдера для привязки.');
            return;
        }

        try {
            await fetchJson(`/api/payouts/${payoutId}/assign`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ traderId }),
            });
            setStatus('success', 'Выплата успешно распределена.');
            await loadData(false);
        } catch (error) {
            console.error('Ошибка привязки выплаты:', error);
            setStatus('error', 'Не удалось привязать выплату: ' + error.message);
        }
    }

    async function saveTraderLimit(traderId) {
        if (!traderId) {
            return;
        }
        const input = document.getElementById(`limit-input-${traderId}`);
        if (!input) {
            return;
        }
        const raw = input.value.trim();
        const maxAmount = raw === '' ? null : Number(raw);

        if (maxAmount !== null && (Number.isNaN(maxAmount) || maxAmount < 0)) {
            setStatus('warning', 'Укажите неотрицательное число или оставьте поле пустым.');
            return;
        }

        try {
            await fetchJson(`/api/traders/${traderId}/limit`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ maxAmount }),
            });
            setStatus('success', 'Лимит трейдера обновлен.');
            await loadData(false);
        } catch (error) {
            console.error('Ошибка сохранения лимита:', error);
            setStatus('error', 'Не удалось сохранить лимит: ' + error.message);
        }
    }

    async function saveSettings() {
        const checkbox = document.getElementById('auto-enabled');
        const intervalInput = document.getElementById('auto-interval');
        const enabled = !!checkbox?.checked;
        const intervalSeconds = Number(intervalInput?.value) || 1;

        try {
            const result = await fetchJson('/api/settings/auto-distribution', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ enabled, intervalSeconds }),
            });
            renderSettings(result);
            setStatus('success', 'Настройки сохранены.');
            markUpdated();
            await loadData(false);
        } catch (error) {
            console.error('Ошибка сохранения настроек:', error);
            setStatus('error', 'Не удалось сохранить настройки: ' + error.message);
        }
    }

    function scheduleReload() {
        if (reloadScheduled) {
            return;
        }
        reloadScheduled = true;
        setTimeout(async () => {
            await loadData(false);
            reloadScheduled = false;
        }, 400);
    }

    function initEventSource() {
        try {
            const eventSource = new EventSource('/api/events');
            eventSource.onmessage = (event) => {
                try {
                    const payload = JSON.parse(event.data);
                    if (payload?.type) {
                        setStatus('info', 'Получено обновление: ' + payload.type);
                    } else {
                        setStatus('info', 'Получено обновление данных.');
                    }
                } catch (parseError) {
                    console.debug('Не удалось разобрать событие SSE:', parseError);
                    setStatus('info', 'Получено обновление данных.');
                }
                scheduleReload();
            };
            eventSource.onerror = () => {
                setStatus('warning', 'SSE соединение потеряно. Переподключение...');
                eventSource.close();
                setTimeout(initEventSource, 5000);
            };
        } catch (error) {
            console.error('Не удалось открыть SSE соединение:', error);
        }
    }

    const initialData = globalThis.__INITIAL_DASHBOARD__;
    if (initialData) {
        try {
            currentTraders = Array.isArray(initialData.traders) ? initialData.traders : [];
            currentPayouts = Array.isArray(initialData.payouts) ? initialData.payouts : [];
            renderTraders(currentTraders);
            renderPayouts(currentPayouts);
            renderSettings(initialData.settings);
            updateMetrics(currentTraders, currentPayouts);
            markUpdated();
            setStatus('info', 'Показаны данные на момент загрузки.');
        } catch (error) {
            console.error('Ошибка применения начальных данных:', error);
        }
    }

    async function bootstrap() {
        const saveButton = document.getElementById('save-settings');
        if (saveButton) {
            saveButton.addEventListener('click', saveSettings);
        }
        initEventSource();
        await loadData(!initialData);
    }

    function start() {
        bootstrap().catch(error => {
            console.error('Не удалось инициализировать страницу:', error);
            setStatus('error', 'Не удалось инициализировать страницу: ' + error.message);
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', start, { once: true });
    } else {
        start();
    }
})();
"#;

#[component]
fn App(snapshot: DashboardSnapshot) -> impl IntoView {
    let initial_json = serde_json::to_string(&snapshot).unwrap_or_else(|_| "{}".to_string());
    let traders = snapshot.traders.clone();
    let payouts = snapshot.payouts.clone();
    let settings = snapshot.settings.clone();

    let metrics_traders = traders.len();
    let total_payout: f64 = payouts.iter().map(|p| p.amount.unwrap_or_default()).sum();
    let metrics_payouts = payouts.len();
    let total_payout_display = format_amount(Some(total_payout));
    let traders_for_options = traders.clone();
    let settings_description = if settings.enabled {
        format!(
            "Автораспределение выполняется каждые {} секунд.",
            settings.interval_seconds.max(1)
        )
    } else {
        "Автораспределение выключено.".to_string()
    };

    let traders_view = if traders.is_empty() {
        view! { <tr><td class="empty" colspan="6">Нет подходящих трейдеров</td></tr> }.into_view()
    } else {
        view! {
            <For
                each=move || traders.clone()
                key=|trader| trader.id.clone()
                children=move |trader| {
                    let limit_value = trader
                        .max_amount
                        .map(|v| format!("{:.2}", v))
                        .unwrap_or_default();
                    view! {
                        <tr>
                            <td>{trader.numeric_id}</td>
                            <td>{trader.email.clone()}</td>
                            <td>{format_amount(trader.balance_rub)}</td>
                            <td>{format_amount(trader.frozen_rub)}</td>
                            <td>{format_amount(trader.payout_balance)}</td>
                            <td>
                                <div class="limit-controls">
                                    <input
                                        type="number"
                                        min="0"
                                        step="0.01"
                                        value=limit_value
                                        id={format!("limit-input-{}", trader.id)}
                                        placeholder="Без лимита"
                                    />
                                    <button class="save-limit" data-trader-id={trader.id.clone()}>"Сохранить"</button>
                                </div>
                            </td>
                        </tr>
                    }
                }
            />
        }
        .into_view()
    };

    let payouts_view = if payouts.is_empty() {
        view! { <tr><td class="empty" colspan="5">Нет нераспределенных выплат</td></tr> }
            .into_view()
    } else {
        view! {
            <For
                each=move || payouts.clone()
                key=|payout| payout.id.clone()
                children=move |payout| {
                    let options: Vec<_> = traders_for_options
                        .iter()
                        .map(|trader| {
                            view! {
                                <option value={trader.id.clone()}>
                                    {format!("{} (ID: {})", trader.email, trader.numeric_id)}
                                </option>
                            }
                        })
                        .collect();
                    view! {
                        <tr>
                            <td>{payout.numeric_id}</td>
                            <td>{format_amount(payout.amount)}</td>
                            <td>{payout.bank.clone().unwrap_or_else(|| "-".to_string())}</td>
                            <td>{payout.external_reference.clone().unwrap_or_else(|| "-".to_string())}</td>
                            <td>
                                <div class="assign-controls">
                                    <select id={format!("assign-select-{}", payout.id)}>
                                        <option value="">"Выберите трейдера"</option>
                                        {options.into_view()}
                                    </select>
                                    <button class="assign-button" data-payout-id={payout.id.clone()}>"Привязать"</button>
                                </div>
                            </td>
                        </tr>
                    }
                }
            />
        }
        .into_view()
    };

    let initial_data_script = format!("window.__INITIAL_DASHBOARD__ = {};", initial_json);
    let dashboard_script = DASHBOARD_SCRIPT.to_string();

    let badge_state = if settings.enabled { "on" } else { "off" };
    let badge_text = if settings.enabled {
        "Активно"
    } else {
        "Выключено"
    };

    view! {
        <html lang="ru">
            <head>
                <meta charset="UTF-8" />
                <title>Chase Linker Dashboard</title>
                <style>{STYLES}</style>
            </head>
            <body>
                <header class="top-bar">
                    <div>
                        <h1>Распределение выплат</h1>
                        <p>Управляйте автораспределением и следите за очередью выплат в реальном времени.</p>
                    </div>
                    <div class="status-block">
                        <span class="status-label">Обновлено</span>
                        <span class="status-value" id="last-updated">-</span>
                    </div>
                </header>
                <main>
                    <div id="global-status" class="status-banner" role="status"></div>
                    <section class="metrics-grid">
                        <article class="metric-card">
                            <span class="metric-label">Активные трейдеры</span>
                            <span class="metric-value" id="metric-traders">{metrics_traders}</span>
                            <span class="metric-sub">Количество трейдеров, готовых принять выплаты</span>
                        </article>
                        <article class="metric-card">
                            <span class="metric-label">Нераспределенных выплат</span>
                            <span class="metric-value" id="metric-payouts">{metrics_payouts}</span>
                            <span class="metric-sub">Текущая очередь выплат без исполнителя</span>
                        </article>
                        <article class="metric-card">
                            <span class="metric-label">Сумма к распределению</span>
                            <span class="metric-value" id="metric-payout-sum">{total_payout_display}</span>
                            <span class="metric-sub">Совокупный объем ожидающих выплат</span>
                        </article>
                    </section>

                    <section class="panel">
                        <div class="panel-header">
                            <div>
                                <h2>Настройки автоматического распределения</h2>
                                <p id="settings-description" class="panel-subtitle">{settings_description}</p>
                            </div>
                            <span id="auto-status-badge" class="badge" data-state=badge_state>{badge_text}</span>
                        </div>
                        <div class="controls-row">
                            <label>
                                <input type="checkbox" id="auto-enabled" checked=settings.enabled />
                                " Включить распределение"
                            </label>
                            <label>
                                "Интервал (сек):"
                                <input
                                    type="number"
                                    id="auto-interval"
                                    min="1"
                                    value={settings.interval_seconds.max(1).to_string()}
                                />
                            </label>
                            <button id="save-settings">Сохранить</button>
                        </div>
                    </section>

                    <section class="panel">
                        <div class="panel-header">
                            <h2>Доступные трейдеры</h2>
                        </div>
                        <div class="table-wrapper">
                            <table id="traders-table">
                                <thead>
                                    <tr>
                                        <th>numericId</th>
                                        <th>Email</th>
                                        <th>Рублевый баланс</th>
                                        <th>Заморожено RUB</th>
                                        <th>Payout баланс</th>
                                        <th>Макс сумма</th>
                                    </tr>
                                </thead>
                                <tbody>{traders_view}</tbody>
                            </table>
                        </div>
                    </section>

                    <section class="panel">
                        <div class="panel-header">
                            <h2>Нераспределенные выплаты</h2>
                        </div>
                        <div class="table-wrapper">
                            <table id="payouts-table">
                                <thead>
                                    <tr>
                                        <th>numericId</th>
                                        <th>Сумма</th>
                                        <th>Банк</th>
                                        <th>External Reference</th>
                                        <th>Действия</th>
                                    </tr>
                                </thead>
                                <tbody>{payouts_view}</tbody>
                            </table>
                        </div>
                    </section>
                </main>
                <script inner_html=initial_data_script></script>
                <script inner_html=dashboard_script></script>
            </body>
        </html>
    }
}

pub(crate) fn render_dashboard_page(snapshot: DashboardSnapshot) -> String {
    let html = leptos::ssr::render_to_string(move || view! { <App snapshot=snapshot.clone() /> });
    format!("<!DOCTYPE html>{html}")
}

fn format_amount(value: Option<f64>) -> String {
    match value {
        Some(v) => format!("{:.2}", v),
        None => "-".to_string(),
    }
}
