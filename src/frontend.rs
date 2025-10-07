use crate::{AutoDistributionConfig, PayoutListResponse, Trader, UnassignedPayout};
use chrono::NaiveDateTime;
use leptos::*;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub(crate) struct DashboardSnapshot {
    pub traders: Vec<Trader>,
    pub payouts: Vec<UnassignedPayout>,
    pub deals: PayoutListResponse,
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
.mono {
    font-family: 'Fira Code', 'Source Code Pro', monospace;
    font-size: 13px;
    letter-spacing: 0.03em;
}
.filters-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
}
.input-control {
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.input-control label {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
}
.input-control input,
.input-control select {
    padding: 8px 12px;
    border-radius: 8px;
    border: 1px solid var(--border-light);
    background: rgba(15, 23, 42, 0.6);
    color: var(--text-primary);
}
.deal-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
}
.deal-reason {
    font-size: 12px;
    color: var(--text-muted);
    max-width: 220px;
    word-break: break-word;
}
.deals-toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
}
.deals-pagination {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: flex-end;
    margin-top: 12px;
}
.deals-pagination button {
    background: rgba(56, 189, 248, 0.18);
    border: 1px solid rgba(56, 189, 248, 0.25);
    color: var(--text-primary);
    padding: 6px 12px;
    border-radius: 8px;
}
.deals-pagination button:disabled {
    opacity: 0.4;
}
button.danger {
    background: linear-gradient(135deg, var(--error), #dc2626);
    color: #fff;
}
#deals-sort-status.active {
    background: rgba(56, 189, 248, 0.2);
    border: 1px solid rgba(56, 189, 248, 0.35);
    color: var(--accent);
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
    const dealsControls = {
        search: document.getElementById('deals-search'),
        wallet: document.getElementById('deals-wallet'),
        amount: document.getElementById('deals-amount'),
        status: document.getElementById('deals-status'),
        perPage: document.getElementById('deals-per-page'),
        sortStatus: document.getElementById('deals-sort-status'),
        reset: document.getElementById('deals-reset'),
        prev: document.getElementById('deals-prev'),
        next: document.getElementById('deals-next'),
        pageInfo: document.getElementById('deals-page-info'),
    };

    let currentTraders = [];
    let currentPayouts = [];
    let currentDeals = [];
    let dealsPagination = {
        page: 1,
        totalPages: 0,
        total: 0,
        perPage: 25,
    };
    let dealsFilters = {
        search: '',
        wallet: '',
        amount: '',
        status: '',
        sort: 'createdAt',
        order: 'desc',
        page: 1,
        perPage: 25,
    };
    let isLoading = false;
    let isDealsLoading = false;
    let reloadScheduled = false;
    let dealsFilterTimer = null;

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

    function formatDateTime(value) {
        if (!value) {
            return '-';
        }
        const date = new Date(value);
        if (Number.isNaN(date.getTime())) {
            return value;
        }
        return date.toLocaleString('ru-RU');
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

    function renderDeals(response) {
        const tbody = document.querySelector('#deals-table tbody');
        if (!tbody) {
            return;
        }

        const items = Array.isArray(response?.items) ? response.items : [];
        currentDeals = items;

        if (response?.pagination) {
            const pagination = response.pagination;
            dealsPagination = {
                page: Number(pagination.page ?? dealsFilters.page ?? 1),
                totalPages: Number(pagination.totalPages ?? 0),
                total: Number(pagination.total ?? 0),
                perPage: Number(pagination.perPage ?? dealsFilters.perPage ?? 25),
            };
            dealsFilters.page = dealsPagination.page;
            dealsFilters.perPage = dealsPagination.perPage;
        } else {
            dealsPagination.totalPages = items.length ? 1 : 0;
            dealsPagination.total = items.length;
        }

        if (!items.length) {
            renderEmpty(tbody, 9, 'Нет выплат по заданным фильтрам');
            updateDealsPagination();
            syncDealsFiltersToControls();
            return;
        }

        tbody.innerHTML = items.map(deal => {
            const amount = formatAmount(deal.amount);
            const external = deal.externalReference ?? '-';
            const cancelReason = deal.cancelReason ?? '-';
            const createdAt = formatDateTime(deal.createdAt);
            const disableCancel = ['CANCELLED', 'COMPLETED', 'SUCCESS', 'FAILED'].includes(deal.status ?? '');
            const cancelTitle = disableCancel
                ? 'Отмена недоступна для этого статуса'
                : 'Отменить выплату';
            return `
                <tr>
                    <td>${deal.numericId}</td>
                    <td><span class="mono">${deal.id}</span></td>
                    <td>${external}</td>
                    <td>${deal.wallet}</td>
                    <td>${deal.bank}</td>
                    <td>${amount}</td>
                    <td>${deal.status}</td>
                    <td>${createdAt}</td>
                    <td>
                        <div class="deal-actions">
                            <span class="deal-reason">${cancelReason}</span>
                            <button
                                class="danger cancel-deal"
                                data-deal-id="${deal.id}"
                                title="${cancelTitle}"
                                ${disableCancel ? 'disabled' : ''}
                            >Отменить</button>
                        </div>
                    </td>
                </tr>
            `;
        }).join('');

        tbody.querySelectorAll('.cancel-deal').forEach(button => {
            button.addEventListener('click', async (event) => {
                const dealId = event.currentTarget.getAttribute('data-deal-id');
                await cancelDeal(dealId);
            });
        });

        updateDealsPagination();
        syncDealsFiltersToControls();
    }

    function updateDealsPagination() {
        const pageInfo = dealsControls.pageInfo;
        if (pageInfo) {
            if (dealsPagination.totalPages > 0) {
                pageInfo.textContent = `${dealsPagination.page} / ${dealsPagination.totalPages} (всего ${dealsPagination.total})`;
            } else {
                pageInfo.textContent = '0 / 0 (всего 0)';
            }
        }

        if (dealsControls.prev) {
            dealsControls.prev.disabled = dealsPagination.page <= 1;
        }
        if (dealsControls.next) {
            const noMorePages =
                dealsPagination.totalPages === 0 || dealsPagination.page >= dealsPagination.totalPages;
            dealsControls.next.disabled = noMorePages;
        }
    }

    function syncDealsFiltersToControls() {
        if (dealsControls.search) {
            dealsControls.search.value = dealsFilters.search ?? '';
        }
        if (dealsControls.wallet) {
            dealsControls.wallet.value = dealsFilters.wallet ?? '';
        }
        if (dealsControls.amount) {
            dealsControls.amount.value = dealsFilters.amount ?? '';
        }
        if (dealsControls.status) {
            dealsControls.status.value = dealsFilters.status ?? '';
        }
        if (dealsControls.perPage) {
            dealsControls.perPage.value = String(dealsFilters.perPage ?? 25);
        }
        syncDealsSortIndicator();
    }

    function syncDealsSortIndicator() {
        if (!dealsControls.sortStatus) {
            return;
        }
        if (dealsFilters.sort === 'status') {
            dealsControls.sortStatus.classList.add('active');
            dealsControls.sortStatus.setAttribute('data-order', dealsFilters.order);
            dealsControls.sortStatus.textContent =
                dealsFilters.order === 'asc' ? 'Статус ↑' : 'Статус ↓';
        } else {
            dealsControls.sortStatus.classList.remove('active');
            dealsControls.sortStatus.removeAttribute('data-order');
            dealsControls.sortStatus.textContent = 'Сортировка по статусу';
        }
    }

    function scheduleDealsReload() {
        if (dealsFilterTimer) {
            clearTimeout(dealsFilterTimer);
        }
        dealsFilterTimer = setTimeout(() => {
            dealsFilters.page = 1;
            loadDeals(true);
        }, 350);
    }

    async function loadDeals(showStatus = false) {
        if (isDealsLoading) {
            return;
        }
        isDealsLoading = true;
        try {
            if (showStatus) {
                setStatus('info', 'Обновляем список выплат...');
            }
            const params = new URLSearchParams();
            if (dealsFilters.search) {
                params.set('search', dealsFilters.search);
            }
            if (dealsFilters.wallet) {
                params.set('wallet', dealsFilters.wallet);
            }
            if (dealsFilters.amount) {
                const num = Number(dealsFilters.amount);
                if (!Number.isNaN(num)) {
                    params.set('amount', String(num));
                }
            }
            if (dealsFilters.status) {
                params.set('status', dealsFilters.status);
            }
            params.set('page', String(dealsFilters.page ?? 1));
            params.set('perPage', String(dealsFilters.perPage ?? 25));
            params.set('sort', dealsFilters.sort ?? 'createdAt');
            params.set('order', dealsFilters.order ?? 'desc');

            const query = params.toString();
            const response = await fetchJson(`/api/deals${query ? `?${query}` : ''}`);
            renderDeals(response);
            if (showStatus) {
                setStatus('success', 'Список выплат обновлен.');
            }
        } catch (error) {
            console.error('Ошибка загрузки выплат:', error);
            const tbody = document.querySelector('#deals-table tbody');
            renderEmpty(tbody, 9, 'Не удалось загрузить выплаты');
            if (showStatus) {
                setStatus('error', 'Не удалось загрузить выплаты: ' + error.message);
            }
        } finally {
            isDealsLoading = false;
        }
    }

    async function cancelDeal(dealId) {
        if (!dealId) {
            return;
        }
        const deal = currentDeals.find(item => item.id === dealId);
        if (deal && ['CANCELLED', 'COMPLETED', 'SUCCESS', 'FAILED'].includes(deal.status ?? '')) {
            setStatus('warning', 'Эту выплату нельзя отменить.');
            return;
        }
        const confirmed = window.confirm('Вы уверены, что хотите отменить выплату?');
        if (!confirmed) {
            return;
        }
        let reason = window.prompt('Причина отмены (необязательно):', '');
        if (reason === null) {
            reason = '';
        }
        const payload = {};
        if (reason.trim()) {
            payload.reason = reason.trim();
        }
        try {
            const result = await fetchJson(`/api/payouts/${dealId}/cancel`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });
            if (result?.callbackDispatched) {
                setStatus('success', 'Выплата отменена.');
            } else if (result?.callbackError) {
                setStatus('warning', 'Выплата отменена, но колбэк не доставлен: ' + result.callbackError);
            } else {
                setStatus('success', 'Выплата отменена.');
            }
            await loadDeals(false);
            await loadData(false);
        } catch (error) {
            console.error('Ошибка отмены выплаты:', error);
            setStatus('error', 'Не удалось отменить выплату: ' + error.message);
        }
    }

    function initDealsControls() {
        if (dealsControls.search) {
            dealsControls.search.addEventListener('input', (event) => {
                dealsFilters.search = event.target.value.trim();
                scheduleDealsReload();
            });
        }
        if (dealsControls.wallet) {
            dealsControls.wallet.addEventListener('input', (event) => {
                dealsFilters.wallet = event.target.value.trim();
                scheduleDealsReload();
            });
        }
        if (dealsControls.amount) {
            dealsControls.amount.addEventListener('input', (event) => {
                dealsFilters.amount = event.target.value.trim();
                scheduleDealsReload();
            });
        }
        if (dealsControls.status) {
            dealsControls.status.addEventListener('change', (event) => {
                dealsFilters.status = event.target.value.trim();
                dealsFilters.page = 1;
                loadDeals(true);
            });
        }
        if (dealsControls.perPage) {
            dealsControls.perPage.addEventListener('change', (event) => {
                const value = Number(event.target.value);
                dealsFilters.perPage = Number.isNaN(value) ? 25 : value;
                dealsFilters.page = 1;
                loadDeals(true);
            });
        }
        if (dealsControls.sortStatus) {
            dealsControls.sortStatus.addEventListener('click', () => {
                if (dealsFilters.sort === 'status') {
                    dealsFilters.order = dealsFilters.order === 'asc' ? 'desc' : 'asc';
                } else {
                    dealsFilters.sort = 'status';
                    dealsFilters.order = 'asc';
                }
                dealsFilters.page = 1;
                syncDealsSortIndicator();
                loadDeals(true);
            });
        }
        if (dealsControls.reset) {
            dealsControls.reset.addEventListener('click', () => {
                dealsFilters = {
                    search: '',
                    wallet: '',
                    amount: '',
                    status: '',
                    sort: 'createdAt',
                    order: 'desc',
                    page: 1,
                    perPage: 25,
                };
                syncDealsFiltersToControls();
                loadDeals(true);
            });
        }
        if (dealsControls.prev) {
            dealsControls.prev.addEventListener('click', () => {
                if (dealsFilters.page > 1) {
                    dealsFilters.page -= 1;
                    loadDeals(true);
                }
            });
        }
        if (dealsControls.next) {
            dealsControls.next.addEventListener('click', () => {
                if (dealsPagination.totalPages && dealsFilters.page < dealsPagination.totalPages) {
                    dealsFilters.page += 1;
                    loadDeals(true);
                }
            });
        }
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
            await Promise.all([loadData(false), loadDeals(false)]);
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
            await Promise.all([loadData(false), loadDeals(false)]);
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
            await Promise.all([loadData(false), loadDeals(false)]);
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
            try {
                await Promise.all([loadData(false), loadDeals(false)]);
            } finally {
                reloadScheduled = false;
            }
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
            if (initialData.deals?.pagination) {
                dealsFilters.perPage = Number(initialData.deals.pagination.perPage ?? dealsFilters.perPage);
                dealsFilters.page = Number(initialData.deals.pagination.page ?? dealsFilters.page);
            }
            renderTraders(currentTraders);
            renderPayouts(currentPayouts);
            if (initialData.deals) {
                renderDeals(initialData.deals);
            } else {
                const dealsBody = document.querySelector('#deals-table tbody');
                renderEmpty(dealsBody, 9, 'Нет данных о выплатах');
            }
            renderSettings(initialData.settings);
            updateMetrics(currentTraders, currentPayouts);
            syncDealsFiltersToControls();
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
        initDealsControls();
        if (!initialData) {
            syncDealsFiltersToControls();
        }
        initEventSource();
        await Promise.all([loadData(!initialData), loadDeals(!initialData)]);
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
    let deals = snapshot.deals.clone();
    let total_payout: f64 = payouts.iter().map(|p| p.amount.unwrap_or_default()).sum();
    let metrics_payouts = payouts.len();
    let total_payout_display = format_amount(Some(total_payout));
    let traders_for_options = traders.clone();
    let deals_items = deals.items.clone();
    let deals_pagination = deals.pagination.clone();
    let deals_page_info = if deals_pagination.total_pages == 0 {
        "0 / 0 (всего 0)".to_string()
    } else {
        format!(
            "{} / {} (всего {})",
            deals_pagination.page,
            deals_pagination.total_pages,
            deals_pagination.total
        )
    };
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

    let deals_view = if deals_items.is_empty() {
        view! { <tr><td class="empty" colspan="9">Нет данных о выплатах</td></tr> }
            .into_view()
    } else {
        view! {
            <For
                each=move || deals_items.clone()
                key=|deal| deal.id.clone()
                children=move |deal| {
                    let external_reference = deal
                        .external_reference
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    let cancel_reason = deal
                        .cancel_reason
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    let disable_cancel = matches!(
                        deal.status.as_str(),
                        "CANCELLED" | "COMPLETED" | "SUCCESS" | "FAILED"
                    );
                    let created_at = format_timestamp(&deal.created_at);
                    let amount_display = format_amount(Some(deal.amount));
                    view! {
                        <tr>
                            <td>{deal.numeric_id}</td>
                            <td><span class="mono">{deal.id.clone()}</span></td>
                            <td>{external_reference}</td>
                            <td>{deal.wallet.clone()}</td>
                            <td>{deal.bank.clone()}</td>
                            <td>{amount_display}</td>
                            <td>{deal.status.clone()}</td>
                            <td>{created_at}</td>
                            <td>
                                <div class="deal-actions">
                                    <span class="deal-reason">{cancel_reason}</span>
                                    <button
                                        class="danger cancel-deal"
                                        data-deal-id={deal.id.clone()}
                                        disabled=disable_cancel
                                        type="button"
                                    >"Отменить"</button>
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

                    <section class="panel">
                        <div class="panel-header">
                            <h2>Все выплаты</h2>
                        </div>
                        <div class="filters-grid">
                            <div class="input-control">
                                <label for="deals-search">Поиск</label>
                                <input
                                    id="deals-search"
                                    type="text"
                                    placeholder="numericId / externalRef / id"
                                    value=""
                                />
                            </div>
                            <div class="input-control">
                                <label for="deals-wallet">Кошелек</label>
                                <input
                                    id="deals-wallet"
                                    type="text"
                                    placeholder="Номер кошелька"
                                    value=""
                                />
                            </div>
                            <div class="input-control">
                                <label for="deals-amount">Сумма</label>
                                <input
                                    id="deals-amount"
                                    type="number"
                                    step="0.01"
                                    min="0"
                                    placeholder="Сумма"
                                    value=""
                                />
                            </div>
                            <div class="input-control">
                                <label for="deals-status">Статус</label>
                                <select id="deals-status">
                                    <option value="">Все</option>
                                    <option value="CREATED">CREATED</option>
                                    <option value="ACTIVE">ACTIVE</option>
                                    <option value="AVAILABLE">AVAILABLE</option>
                                    <option value="CHECKING">CHECKING</option>
                                    <option value="PROCESSING">PROCESSING</option>
                                    <option value="CANCELLED">CANCELLED</option>
                                    <option value="COMPLETED">COMPLETED</option>
                                    <option value="SUCCESS">SUCCESS</option>
                                    <option value="FAILED">FAILED</option>
                                    <option value="DISPUTED">DISPUTED</option>
                                    <option value="EXPIRED">EXPIRED</option>
                                    <option value="DISPUTE">DISPUTE</option>
                                </select>
                            </div>
                            <div class="input-control">
                                <label for="deals-per-page">На странице</label>
                                <select id="deals-per-page">
                                    <option value="25" selected={deals_pagination.per_page == 25}>25</option>
                                    <option value="50" selected={deals_pagination.per_page == 50}>50</option>
                                    <option value="100" selected={deals_pagination.per_page == 100}>100</option>
                                </select>
                            </div>
                        </div>
                        <div class="deals-toolbar">
                            <button id="deals-sort-status" type="button">Сортировка по статусу</button>
                            <button id="deals-reset" type="button">Сбросить фильтры</button>
                        </div>
                        <div class="table-wrapper">
                            <table id="deals-table">
                                <thead>
                                    <tr>
                                        <th>numericId</th>
                                        <th>ID</th>
                                        <th>External Reference</th>
                                        <th>Wallet</th>
                                        <th>Банк</th>
                                        <th>Сумма</th>
                                        <th>Статус</th>
                                        <th>Создана</th>
                                        <th>Действия</th>
                                    </tr>
                                </thead>
                                <tbody>{deals_view}</tbody>
                            </table>
                        </div>
                        <div class="deals-pagination">
                            <span id="deals-page-info">{deals_page_info.clone()}</span>
                            <button id="deals-prev" type="button" disabled={deals_pagination.page <= 1}>"Назад"</button>
                            <button
                                id="deals-next"
                                type="button"
                                disabled={deals_pagination.total_pages == 0
                                    || deals_pagination.page >= deals_pagination.total_pages}
                            >"Вперед"</button>
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

fn format_timestamp(value: &NaiveDateTime) -> String {
    value.format("%Y-%m-%d %H:%M:%S").to_string()
}
