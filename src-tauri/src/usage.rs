use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub(crate) enum Account {
    #[serde(rename = "apiKey")]
    ApiKey,
    #[serde(rename = "chatgpt")]
    ChatGpt {
        #[serde(default)]
        email: Option<String>,
        #[serde(rename = "planType")]
        plan_type: String,
    },
    #[serde(rename = "amazonBedrock")]
    AmazonBedrock {
        #[serde(rename = "credentialSource")]
        credential_source: String,
    },
}

impl Account {
    pub(crate) fn chatgpt_plan(&self) -> Option<&str> {
        match self {
            Self::ChatGpt { plan_type, .. } => Some(plan_type),
            _ => None,
        }
    }

    pub(crate) fn privacy_safe_kind(&self) -> &'static str {
        match self {
            Self::ApiKey => "apiKey",
            Self::ChatGpt { .. } => "chatgpt",
            Self::AmazonBedrock { .. } => "amazonBedrock",
        }
    }

    pub(crate) fn discard_private_fields(&self) {
        match self {
            Self::ChatGpt { email, .. } => {
                let _ = email.as_deref();
            }
            Self::AmazonBedrock { credential_source } => {
                let _ = credential_source.as_str();
            }
            Self::ApiKey => {}
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccountResponse {
    pub account: Option<Account>,
    pub requires_openai_auth: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RateLimitWindowRaw {
    pub used_percent: f64,
    pub window_duration_mins: Option<u64>,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreditsSnapshotRaw {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RateLimitSnapshotRaw {
    pub limit_id: Option<String>,
    pub limit_name: Option<String>,
    pub primary: Option<RateLimitWindowRaw>,
    pub secondary: Option<RateLimitWindowRaw>,
    pub credits: Option<CreditsSnapshotRaw>,
    pub plan_type: Option<String>,
    pub rate_limit_reached_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RateLimitResetCreditsRaw {
    pub available_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RateLimitsResponse {
    pub rate_limits: RateLimitSnapshotRaw,
    pub rate_limits_by_limit_id: Option<HashMap<String, RateLimitSnapshotRaw>>,
    pub rate_limit_reset_credits: Option<RateLimitResetCreditsRaw>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TokenUsageDailyBucketRaw {
    pub start_date: String,
    pub tokens: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TokenUsageSummaryRaw {
    pub lifetime_tokens: Option<u64>,
    pub peak_daily_tokens: Option<u64>,
    pub longest_running_turn_sec: Option<u64>,
    pub current_streak_days: Option<u64>,
    pub longest_streak_days: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TokenUsageResponse {
    pub summary: TokenUsageSummaryRaw,
    pub daily_usage_buckets: Option<Vec<TokenUsageDailyBucketRaw>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotStatus {
    Ready,
    SignedOut,
    Unsupported,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub status: SnapshotStatus,
    pub message: Option<String>,
    pub source: String,
    pub updated_at: String,
    pub codex_version: Option<String>,
    pub account: Option<AccountView>,
    pub rate_limits: Option<RateLimitsView>,
    pub token_usage: Option<TokenUsageView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub auth_type: String,
    pub plan_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitWindowView {
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub window_duration_mins: Option<u64>,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditsView {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitBucketView {
    pub limit_id: Option<String>,
    pub limit_name: Option<String>,
    pub primary: Option<RateLimitWindowView>,
    pub secondary: Option<RateLimitWindowView>,
    pub credits: Option<CreditsView>,
    pub plan_type: Option<String>,
    pub rate_limit_reached_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitsView {
    pub selected: RateLimitBucketView,
    pub buckets: Vec<RateLimitBucketView>,
    pub reset_credits_available: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageDailyBucketView {
    pub start_date: String,
    pub tokens: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageView {
    pub today_tokens: Option<String>,
    pub last_7_days_tokens: Option<String>,
    pub lifetime_tokens: Option<String>,
    pub peak_daily_tokens: Option<String>,
    pub longest_running_turn_sec: Option<String>,
    pub current_streak_days: Option<String>,
    pub longest_streak_days: Option<String>,
    pub daily_buckets: Option<Vec<TokenUsageDailyBucketView>>,
}

pub(crate) struct SnapshotMetadata {
    pub today: NaiveDate,
    pub source: String,
    pub updated_at: String,
    pub codex_version: Option<String>,
}

impl UsageSnapshot {
    pub(crate) fn signed_out(
        requires_openai_auth: bool,
        source: String,
        updated_at: String,
        codex_version: Option<String>,
    ) -> Self {
        let message = if requires_openai_auth {
            "Codex 尚未登录，请先在 Codex 中使用 ChatGPT 账号登录。"
        } else {
            "没有检测到可读取的 Codex 账户。"
        };
        Self {
            status: SnapshotStatus::SignedOut,
            message: Some(message.to_string()),
            source,
            updated_at,
            codex_version,
            account: None,
            rate_limits: None,
            token_usage: None,
        }
    }

    pub(crate) fn unsupported(
        account: &Account,
        source: String,
        updated_at: String,
        codex_version: Option<String>,
    ) -> Self {
        account.discard_private_fields();
        let message = match account {
            Account::ApiKey => "API Key 登录不支持读取 ChatGPT 账户额度。",
            Account::AmazonBedrock { .. } => "Amazon Bedrock 登录不支持读取 ChatGPT 账户额度。",
            Account::ChatGpt { .. } => "当前账户暂不支持读取用量。",
        };
        Self {
            status: SnapshotStatus::Unsupported,
            message: Some(message.to_string()),
            source,
            updated_at,
            codex_version,
            account: Some(AccountView {
                auth_type: account.privacy_safe_kind().to_string(),
                plan_type: account.chatgpt_plan().map(str::to_string),
            }),
            rate_limits: None,
            token_usage: None,
        }
    }
}

pub(crate) fn build_ready_snapshot(
    account: &Account,
    rate_limits: RateLimitsResponse,
    token_usage: Option<TokenUsageResponse>,
    usage_note: Option<String>,
    metadata: SnapshotMetadata,
) -> UsageSnapshot {
    let selected_raw = normalized_selected_bucket(&rate_limits);
    let mut buckets = rate_limits
        .rate_limits_by_limit_id
        .as_ref()
        .map(|items| {
            let mut entries = items
                .iter()
                .map(|(id, snapshot)| {
                    let mut snapshot = snapshot.clone();
                    if snapshot.limit_id.is_none() {
                        snapshot.limit_id = Some(id.clone());
                    }
                    to_bucket_view(snapshot)
                })
                .collect::<Vec<_>>();
            entries.sort_by(|left, right| left.limit_id.cmp(&right.limit_id));
            entries
        })
        .unwrap_or_default();

    if buckets.is_empty() {
        buckets.push(to_bucket_view(rate_limits.rate_limits.clone()));
    }

    UsageSnapshot {
        status: SnapshotStatus::Ready,
        message: usage_note,
        source: metadata.source,
        updated_at: metadata.updated_at,
        codex_version: metadata.codex_version,
        account: Some(AccountView {
            auth_type: account.privacy_safe_kind().to_string(),
            plan_type: account
                .chatgpt_plan()
                .or(selected_raw.plan_type.as_deref())
                .map(str::to_string),
        }),
        rate_limits: Some(RateLimitsView {
            selected: to_bucket_view(selected_raw),
            buckets,
            reset_credits_available: rate_limits
                .rate_limit_reset_credits
                .map(|credits| credits.available_count.to_string()),
        }),
        token_usage: token_usage.map(|usage| to_token_usage_view(usage, metadata.today)),
    }
}

fn select_bucket(response: &RateLimitsResponse) -> &RateLimitSnapshotRaw {
    let Some(buckets) = response.rate_limits_by_limit_id.as_ref() else {
        return &response.rate_limits;
    };

    buckets
        .get("codex")
        .or_else(|| {
            buckets.values().find(|item| {
                item.limit_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case("codex"))
            })
        })
        .unwrap_or(&response.rate_limits)
}

fn normalized_selected_bucket(response: &RateLimitsResponse) -> RateLimitSnapshotRaw {
    let mut selected = select_bucket(response).clone();
    let mut windows = Vec::new();
    push_unique_window(&mut windows, selected.primary.as_ref());
    push_unique_window(&mut windows, selected.secondary.as_ref());

    let historical_is_relevant = selected.limit_id == response.rate_limits.limit_id
        || selected.limit_id.as_deref() == Some("codex")
        || response.rate_limits.limit_id.is_none();
    if historical_is_relevant {
        push_unique_window(&mut windows, response.rate_limits.primary.as_ref());
        push_unique_window(&mut windows, response.rate_limits.secondary.as_ref());
    }

    windows.sort_by_key(|window| window.window_duration_mins.unwrap_or(u64::MAX));
    match windows.as_slice() {
        [] => {
            selected.primary = None;
            selected.secondary = None;
        }
        [only]
            if only
                .window_duration_mins
                .is_some_and(|minutes| minutes >= 24 * 60) =>
        {
            selected.primary = None;
            selected.secondary = Some(only.clone());
        }
        [only] => {
            selected.primary = Some(only.clone());
            selected.secondary = None;
        }
        [first, .., last] => {
            selected.primary = Some(first.clone());
            selected.secondary = Some(last.clone());
        }
    }
    selected
}

fn push_unique_window(
    windows: &mut Vec<RateLimitWindowRaw>,
    candidate: Option<&RateLimitWindowRaw>,
) {
    let Some(candidate) = candidate else {
        return;
    };
    if !windows.iter().any(|existing| existing == candidate) {
        windows.push(candidate.clone());
    }
}

fn to_bucket_view(raw: RateLimitSnapshotRaw) -> RateLimitBucketView {
    RateLimitBucketView {
        limit_id: raw.limit_id,
        limit_name: raw.limit_name,
        primary: raw.primary.map(to_window_view),
        secondary: raw.secondary.map(to_window_view),
        credits: raw.credits.map(|credits| CreditsView {
            has_credits: credits.has_credits,
            unlimited: credits.unlimited,
            balance: credits.balance,
        }),
        plan_type: raw.plan_type,
        rate_limit_reached_type: raw.rate_limit_reached_type,
    }
}

fn to_window_view(raw: RateLimitWindowRaw) -> RateLimitWindowView {
    let used_percent = raw.used_percent.clamp(0.0, 100.0);
    RateLimitWindowView {
        used_percent,
        remaining_percent: 100.0 - used_percent,
        window_duration_mins: raw.window_duration_mins,
        resets_at: raw.resets_at,
    }
}

fn to_token_usage_view(raw: TokenUsageResponse, today: NaiveDate) -> TokenUsageView {
    let cutoff = today - Duration::days(6);
    let buckets = raw.daily_usage_buckets.unwrap_or_default();
    let today_tokens = buckets
        .iter()
        .find(|bucket| parse_bucket_date(&bucket.start_date) == Some(today))
        .map(|bucket| bucket.tokens);
    let last_7_days_tokens = buckets
        .iter()
        .filter_map(|bucket| {
            let date = parse_bucket_date(&bucket.start_date)?;
            (date >= cutoff && date <= today).then_some(bucket.tokens)
        })
        .reduce(|total, tokens| total.saturating_add(tokens));
    let daily_buckets = (!buckets.is_empty()).then(|| {
        buckets
            .into_iter()
            .map(|bucket| TokenUsageDailyBucketView {
                start_date: bucket.start_date,
                tokens: bucket.tokens.to_string(),
            })
            .collect()
    });

    TokenUsageView {
        today_tokens: today_tokens.map(|value| value.to_string()),
        last_7_days_tokens: last_7_days_tokens.map(|value| value.to_string()),
        lifetime_tokens: raw.summary.lifetime_tokens.map(|value| value.to_string()),
        peak_daily_tokens: raw.summary.peak_daily_tokens.map(|value| value.to_string()),
        longest_running_turn_sec: raw
            .summary
            .longest_running_turn_sec
            .map(|value| value.to_string()),
        current_streak_days: raw
            .summary
            .current_streak_days
            .map(|value| value.to_string()),
        longest_streak_days: raw
            .summary
            .longest_streak_days
            .map(|value| value.to_string()),
        daily_buckets,
    }
}

fn parse_bucket_date(value: &str) -> Option<NaiveDate> {
    value
        .get(..10)
        .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_fixture<T: for<'de> Deserialize<'de>>(contents: &str) -> T {
        serde_json::from_str(contents).expect("fixture should match the current app-server shape")
    }

    #[test]
    fn maps_multi_bucket_rate_limits_and_usage_without_exposing_email() {
        let account: AccountResponse =
            read_fixture(include_str!("../tests/fixtures/account-chatgpt-plus.json"));
        let rate_limits: RateLimitsResponse =
            read_fixture(include_str!("../tests/fixtures/rate-limits-multi.json"));
        let token_usage: TokenUsageResponse =
            read_fixture(include_str!("../tests/fixtures/token-usage.json"));
        let account = account.account.expect("fixture account");
        let snapshot = build_ready_snapshot(
            &account,
            rate_limits,
            Some(token_usage),
            None,
            SnapshotMetadata {
                today: NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
                source: "Codex app-server".to_string(),
                updated_at: "2026-07-15T12:00:00Z".to_string(),
                codex_version: Some("codex-cli 0.144.4".to_string()),
            },
        );
        let serialized = serde_json::to_string(&snapshot).unwrap();

        assert!(!serialized.contains('@'));
        assert_eq!(snapshot.account.unwrap().plan_type.as_deref(), Some("plus"));
        let limits = snapshot.rate_limits.unwrap();
        assert_eq!(limits.selected.limit_id.as_deref(), Some("codex"));
        assert_eq!(limits.selected.primary.unwrap().remaining_percent, 72.0);
        assert_eq!(limits.buckets.len(), 2);
        let usage = snapshot.token_usage.unwrap();
        assert_eq!(usage.today_tokens.as_deref(), Some("1280000"));
        assert_eq!(usage.last_7_days_tokens.as_deref(), Some("6420000"));
    }

    #[test]
    fn rejects_api_key_as_an_account_quota_source() {
        let response: AccountResponse =
            read_fixture(include_str!("../tests/fixtures/account-api-key.json"));
        let account = response.account.expect("fixture account");
        let snapshot = UsageSnapshot::unsupported(
            &account,
            "Codex app-server".to_string(),
            "2026-07-15T12:00:00Z".to_string(),
            None,
        );

        assert!(matches!(snapshot.status, SnapshotStatus::Unsupported));
        assert!(snapshot.rate_limits.is_none());
        assert!(snapshot.message.unwrap().contains("API Key"));
    }

    #[test]
    fn classifies_a_single_weekly_window_as_long_cycle() {
        let account: AccountResponse =
            read_fixture(include_str!("../tests/fixtures/account-chatgpt-plus.json"));
        let mut rate_limits: RateLimitsResponse =
            read_fixture(include_str!("../tests/fixtures/rate-limits-multi.json"));
        let weekly = RateLimitWindowRaw {
            used_percent: 75.0,
            window_duration_mins: Some(10_080),
            resets_at: Some(1_784_690_904),
        };
        rate_limits.rate_limits.primary = Some(weekly.clone());
        rate_limits.rate_limits.secondary = None;
        if let Some(codex) = rate_limits
            .rate_limits_by_limit_id
            .as_mut()
            .and_then(|buckets| buckets.get_mut("codex"))
        {
            codex.primary = Some(weekly);
            codex.secondary = None;
        }
        let account = account.account.unwrap();
        let snapshot = build_ready_snapshot(
            &account,
            rate_limits,
            None,
            None,
            SnapshotMetadata {
                today: NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
                source: "Codex app-server".to_string(),
                updated_at: "2026-07-15T12:00:00Z".to_string(),
                codex_version: None,
            },
        );
        let selected = snapshot.rate_limits.unwrap().selected;

        assert!(selected.primary.is_none());
        assert_eq!(selected.secondary.unwrap().remaining_percent, 25.0);
    }
}
