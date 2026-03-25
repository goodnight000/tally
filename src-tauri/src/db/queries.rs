use rusqlite::{params, Connection};

use super::models::*;

/// SQL expression for computing estimated cost from session token columns + cost_rates join.
/// Assumes session columns are aliased as s.* and cost_rates joined as c.*
const COST_EXPR: &str =
    "((CASE WHEN s.total_input_tokens > s.total_cache_read_tokens
           THEN (s.total_input_tokens - s.total_cache_read_tokens) ELSE 0 END)
      * COALESCE(c.input_per_million, 0) / 1000000.0) +
     (s.total_cache_read_tokens * COALESCE(c.cache_read_per_million, 0) / 1000000.0) +
     (s.total_cache_creation_tokens * COALESCE(c.cache_creation_per_million, 0) / 1000000.0) +
     (s.total_output_tokens * COALESCE(c.output_per_million, 0) / 1000000.0)";

/// Get dashboard stats for the home page
pub fn get_dashboard_stats(
    conn: &Connection,
    tool: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<DashboardStats, rusqlite::Error> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Tokens and sessions today
    let (tokens_today, sessions_today) = {
        let mut sql = String::from(
            "SELECT COALESCE(SUM(total_tokens), 0), COUNT(*)
             FROM sessions WHERE date(start_time) = ?1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(today.clone())];
        if let Some(t) = tool {
            sql.push_str(" AND tool = ?2");
            param_values.push(Box::new(t.to_string()));
        }
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        conn.query_row(&sql, params_ref.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?
    };

    // Totals with cost (JOIN cost_rates)
    let mut sql = format!(
        "SELECT COUNT(*),
                COALESCE(SUM(s.total_tokens), 0),
                COALESCE(SUM(s.total_input_tokens), 0),
                COALESCE(SUM(s.total_output_tokens), 0),
                COALESCE(SUM(s.total_cache_read_tokens), 0),
                COALESCE(SUM(s.total_cache_creation_tokens), 0),
                COALESCE(SUM(s.total_reasoning_tokens), 0),
                COALESCE(SUM({}), 0.0)
         FROM sessions s
         LEFT JOIN cost_rates c ON s.model = c.model
         WHERE 1=1",
        COST_EXPR
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    let mut param_idx = 1;

    if let Some(t) = tool {
        sql.push_str(&format!(" AND s.tool = ?{}", param_idx));
        param_values.push(Box::new(t.to_string()));
        param_idx += 1;
    }
    if let Some(s) = start {
        sql.push_str(&format!(" AND s.start_time >= ?{}", param_idx));
        param_values.push(Box::new(s.to_string()));
        param_idx += 1;
    }
    if let Some(e) = end {
        sql.push_str(&format!(" AND s.start_time <= ?{}", param_idx));
        param_values.push(Box::new(e.to_string()));
    }

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let row_data = conn.query_row(&sql, params_ref.as_slice(), |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, f64>(7)?,
        ))
    })?;

    let streak = get_streak(conn, tool)?;

    Ok(DashboardStats {
        streak,
        tokens_today,
        sessions_today,
        total_tokens: row_data.1,
        total_sessions: row_data.0,
        total_input_tokens: row_data.2,
        total_output_tokens: row_data.3,
        total_cache_read_tokens: row_data.4,
        total_cache_creation_tokens: row_data.5,
        total_reasoning_tokens: row_data.6,
        estimated_cost: row_data.7,
    })
}

/// Calculate consecutive-day usage streak
pub fn get_streak(conn: &Connection, tool: Option<&str>) -> Result<i32, rusqlite::Error> {
    let mut sql = String::from(
        "SELECT DISTINCT date(start_time) as d FROM sessions WHERE 1=1",
    );
    if let Some(t) = tool {
        sql.push_str(&format!(" AND tool = '{}'", t.replace('\'', "''")));
    }
    sql.push_str(" ORDER BY d DESC");

    let mut stmt = conn.prepare(&sql)?;
    let dates: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    if dates.is_empty() {
        return Ok(0);
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut streak = 0;
    let mut expected = chrono::Utc::now().date_naive();

    if dates.first().map(|d| d.as_str()) != Some(today.as_str()) {
        expected -= chrono::Duration::days(1);
    }

    for date_str in &dates {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            if date == expected {
                streak += 1;
                expected -= chrono::Duration::days(1);
            } else {
                break;
            }
        }
    }

    Ok(streak)
}

/// Get daily usage stats for chart (with cost)
pub fn get_daily_usage(
    conn: &Connection,
    tool: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<DailyStat>, rusqlite::Error> {
    let mut sql = format!(
        "SELECT date(s.start_time) as d,
                s.tool,
                COALESCE(SUM(s.total_input_tokens), 0),
                COALESCE(SUM(s.total_output_tokens), 0),
                COALESCE(SUM(s.total_cache_read_tokens), 0),
                COALESCE(SUM(s.total_cache_creation_tokens), 0),
                COALESCE(SUM(s.total_reasoning_tokens), 0),
                COALESCE(SUM(s.total_tokens), 0),
                COUNT(*),
                COALESCE(SUM({}), 0.0)
         FROM sessions s
         LEFT JOIN cost_rates c ON s.model = c.model
         WHERE 1=1",
        COST_EXPR
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    let mut param_idx = 1;

    if let Some(t) = tool {
        sql.push_str(&format!(" AND s.tool = ?{}", param_idx));
        param_values.push(Box::new(t.to_string()));
        param_idx += 1;
    }
    if let Some(s) = start {
        sql.push_str(&format!(" AND s.start_time >= ?{}", param_idx));
        param_values.push(Box::new(s.to_string()));
        param_idx += 1;
    }
    if let Some(e) = end {
        sql.push_str(&format!(" AND s.start_time <= ?{}", param_idx));
        param_values.push(Box::new(e.to_string()));
    }
    sql.push_str(" GROUP BY d, s.tool ORDER BY d ASC");

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let results = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(DailyStat {
                date: row.get(0)?,
                tool: row.get(1)?,
                input_tokens: row.get(2)?,
                output_tokens: row.get(3)?,
                cache_read_tokens: row.get(4)?,
                cache_creation_tokens: row.get(5)?,
                reasoning_tokens: row.get(6)?,
                total_tokens: row.get(7)?,
                session_count: row.get(8)?,
                estimated_cost: row.get(9)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Get token breakdown by model (with cost)
pub fn get_model_breakdown(
    conn: &Connection,
    tool: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<ModelBreakdown>, rusqlite::Error> {
    let mut sql = format!(
        "SELECT COALESCE(s.model, 'unknown') as m,
                s.tool,
                COALESCE(SUM(s.total_tokens), 0),
                COALESCE(SUM(s.total_input_tokens), 0),
                COALESCE(SUM(s.total_output_tokens), 0),
                COALESCE(SUM({}), 0.0)
         FROM sessions s
         LEFT JOIN cost_rates c ON s.model = c.model
         WHERE 1=1",
        COST_EXPR
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    let mut param_idx = 1;

    if let Some(t) = tool {
        sql.push_str(&format!(" AND s.tool = ?{}", param_idx));
        param_values.push(Box::new(t.to_string()));
        param_idx += 1;
    }
    if let Some(s) = start {
        sql.push_str(&format!(" AND s.start_time >= ?{}", param_idx));
        param_values.push(Box::new(s.to_string()));
        param_idx += 1;
    }
    if let Some(e) = end {
        sql.push_str(&format!(" AND s.start_time <= ?{}", param_idx));
        param_values.push(Box::new(e.to_string()));
    }
    sql.push_str(" GROUP BY m, s.tool ORDER BY SUM(s.total_tokens) DESC");

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let results = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(ModelBreakdown {
                model: row.get(0)?,
                tool: row.get(1)?,
                total_tokens: row.get(2)?,
                input_tokens: row.get(3)?,
                output_tokens: row.get(4)?,
                estimated_cost: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Get token usage by project (with cost)
pub fn get_project_breakdown(
    conn: &Connection,
    tool: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<ProjectSummary>, rusqlite::Error> {
    let mut sql = format!(
        "SELECT COALESCE(s.project_name, 'unknown') as p,
                COALESCE(SUM(s.total_tokens), 0),
                COUNT(*),
                COALESCE(SUM({}), 0.0)
         FROM sessions s
         LEFT JOIN cost_rates c ON s.model = c.model
         WHERE 1=1",
        COST_EXPR
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    let mut param_idx = 1;

    if let Some(t) = tool {
        sql.push_str(&format!(" AND s.tool = ?{}", param_idx));
        param_values.push(Box::new(t.to_string()));
        param_idx += 1;
    }
    if let Some(s) = start {
        sql.push_str(&format!(" AND s.start_time >= ?{}", param_idx));
        param_values.push(Box::new(s.to_string()));
        param_idx += 1;
    }
    if let Some(e) = end {
        sql.push_str(&format!(" AND s.start_time <= ?{}", param_idx));
        param_values.push(Box::new(e.to_string()));
    }
    sql.push_str(" GROUP BY p ORDER BY SUM(s.total_tokens) DESC");

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let results = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(ProjectSummary {
                project_name: row.get(0)?,
                total_tokens: row.get(1)?,
                session_count: row.get(2)?,
                estimated_cost: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Get heatmap data (day of week × hour)
pub fn get_heatmap_data(
    conn: &Connection,
    tool: Option<&str>,
) -> Result<Vec<HeatmapEntry>, rusqlite::Error> {
    let mut sql = String::from(
        "SELECT CAST(strftime('%w', start_time) AS INTEGER) as dow,
                CAST(strftime('%H', start_time) AS INTEGER) as hour,
                COUNT(*)
         FROM sessions WHERE 1=1",
    );
    if let Some(t) = tool {
        sql.push_str(&format!(" AND tool = '{}'", t.replace('\'', "''")));
    }
    sql.push_str(" GROUP BY dow, hour");

    let mut stmt = conn.prepare(&sql)?;
    let results = stmt
        .query_map([], |row| {
            Ok(HeatmapEntry {
                day_of_week: row.get(0)?,
                hour: row.get(1)?,
                count: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Get hourly usage for a specific date (for "Today" view)
pub fn get_hourly_usage(
    conn: &Connection,
    tool: Option<&str>,
    date: &str,
) -> Result<Vec<DailyStat>, rusqlite::Error> {
    let mut sql = format!(
        "SELECT printf('%02d', CAST(strftime('%H', s.start_time) AS INTEGER)) as hour_label,
                s.tool,
                COALESCE(SUM(s.total_input_tokens), 0),
                COALESCE(SUM(s.total_output_tokens), 0),
                COALESCE(SUM(s.total_cache_read_tokens), 0),
                COALESCE(SUM(s.total_cache_creation_tokens), 0),
                COALESCE(SUM(s.total_reasoning_tokens), 0),
                COALESCE(SUM(s.total_tokens), 0),
                COUNT(*),
                COALESCE(SUM({}), 0.0)
         FROM sessions s
         LEFT JOIN cost_rates c ON s.model = c.model
         WHERE date(s.start_time) = ?1",
        COST_EXPR
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(date.to_string())];
    let mut param_idx = 2;

    if let Some(t) = tool {
        sql.push_str(&format!(" AND s.tool = ?{}", param_idx));
        param_values.push(Box::new(t.to_string()));
    }
    sql.push_str(" GROUP BY hour_label, s.tool ORDER BY hour_label ASC");

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let results = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(DailyStat {
                date: row.get(0)?, // hour label like "00", "14", "23"
                tool: row.get(1)?,
                input_tokens: row.get(2)?,
                output_tokens: row.get(3)?,
                cache_read_tokens: row.get(4)?,
                cache_creation_tokens: row.get(5)?,
                reasoning_tokens: row.get(6)?,
                total_tokens: row.get(7)?,
                session_count: row.get(8)?,
                estimated_cost: row.get(9)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Get daily activity counts for the last N days (for 30-day heatmap)
pub fn get_daily_activity(
    conn: &Connection,
    tool: Option<&str>,
    days: i64,
) -> Result<Vec<DailyActivity>, rusqlite::Error> {
    let mut sql = format!(
        "SELECT date(start_time) as d, COUNT(*)
         FROM sessions
         WHERE start_time >= date('now', '-{} days')",
        days
    );
    if let Some(t) = tool {
        sql.push_str(&format!(" AND tool = '{}'", t.replace('\'', "''")));
    }
    sql.push_str(" GROUP BY d ORDER BY d ASC");

    let mut stmt = conn.prepare(&sql)?;
    let results = stmt
        .query_map([], |row| {
            Ok(DailyActivity {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Session SELECT columns (with cost via JOIN)
const SESSION_SELECT: &str =
    "s.id, s.tool, s.source, s.parent_session_id, s.model, s.title, s.start_time, s.end_time,
     s.project_path, s.project_name, s.git_branch, s.git_sha, s.git_origin_url, s.cli_version,
     s.total_input_tokens, s.total_output_tokens, s.total_cache_read_tokens,
     s.total_cache_creation_tokens, s.total_reasoning_tokens, s.total_tokens";

/// Get top N sessions by token count (with cost)
pub fn get_top_sessions(
    conn: &Connection,
    tool: Option<&str>,
    limit: i64,
) -> Result<Vec<Session>, rusqlite::Error> {
    let mut sql = format!(
        "SELECT {}, COALESCE({}, 0.0)
         FROM sessions s
         LEFT JOIN cost_rates c ON s.model = c.model
         WHERE 1=1",
        SESSION_SELECT, COST_EXPR
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    let mut param_idx = 1;

    if let Some(t) = tool {
        sql.push_str(&format!(" AND s.tool = ?{}", param_idx));
        param_values.push(Box::new(t.to_string()));
        param_idx += 1;
    }
    sql.push_str(&format!(" ORDER BY s.total_tokens DESC LIMIT ?{}", param_idx));
    param_values.push(Box::new(limit));

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let results = stmt
        .query_map(params_ref.as_slice(), |row| row_to_session_with_cost(row))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Get paginated, filtered session list (with cost)
pub fn get_sessions(
    conn: &Connection,
    filters: &SessionFilters,
) -> Result<SessionPage, rusqlite::Error> {
    let page = filters.page.unwrap_or(1).max(1);
    let page_size = filters.page_size.unwrap_or(50).max(1).min(200);
    let offset = (page - 1) * page_size;

    let (where_clause, param_values) = build_session_where(filters);

    // Count total
    let count_sql = format!("SELECT COUNT(*) FROM sessions s WHERE {}", where_clause);
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let total_count: i64 = conn.query_row(&count_sql, params_ref.as_slice(), |row| row.get(0))?;

    // Fetch page
    let sort_col = match filters.sort_by.as_deref() {
        Some("tool") => "s.tool",
        Some("model") => "s.model",
        Some("project_name") => "s.project_name",
        Some("total_tokens") => "s.total_tokens",
        Some("total_input_tokens") => "s.total_input_tokens",
        Some("total_output_tokens") => "s.total_output_tokens",
        Some("estimated_cost") => "estimated_cost",
        _ => "s.start_time",
    };
    let sort_dir = match filters.sort_dir.as_deref() {
        Some("asc") | Some("ASC") => "ASC",
        _ => "DESC",
    };

    let select_sql = format!(
        "SELECT {}, COALESCE({}, 0.0) as estimated_cost
         FROM sessions s
         LEFT JOIN cost_rates c ON s.model = c.model
         WHERE {} ORDER BY {} {} LIMIT {} OFFSET {}",
        SESSION_SELECT, COST_EXPR, where_clause, sort_col, sort_dir, page_size, offset
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&select_sql)?;
    let sessions: Vec<Session> = stmt
        .query_map(params_ref.as_slice(), |row| row_to_session_with_cost(row))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(SessionPage {
        sessions,
        total_count,
        page,
        page_size,
    })
}

/// Get session detail with requests and children
pub fn get_session_detail(
    conn: &Connection,
    session_id: &str,
) -> Result<SessionDetail, rusqlite::Error> {
    let session = conn.query_row(
        &format!(
            "SELECT {}, COALESCE({}, 0.0)
             FROM sessions s
             LEFT JOIN cost_rates c ON s.model = c.model
             WHERE s.id = ?1",
            SESSION_SELECT, COST_EXPR
        ),
        params![session_id],
        |row| row_to_session_with_cost(row),
    )?;

    let mut req_stmt = conn.prepare(
        "SELECT id, session_id, timestamp, model, input_tokens, output_tokens,
                cache_read_tokens, cache_creation_tokens, reasoning_tokens, total_tokens, duration_ms
         FROM requests WHERE session_id = ?1 ORDER BY timestamp ASC",
    )?;
    let requests: Vec<Request> = req_stmt
        .query_map(params![session_id], |row| {
            Ok(Request {
                id: row.get(0)?,
                session_id: row.get(1)?,
                timestamp: row.get(2)?,
                model: row.get(3)?,
                input_tokens: row.get(4)?,
                output_tokens: row.get(5)?,
                cache_read_tokens: row.get(6)?,
                cache_creation_tokens: row.get(7)?,
                reasoning_tokens: row.get(8)?,
                total_tokens: row.get(9)?,
                duration_ms: row.get(10)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut child_stmt = conn.prepare(
        &format!(
            "SELECT {}, COALESCE({}, 0.0)
             FROM sessions s
             LEFT JOIN cost_rates c ON s.model = c.model
             WHERE s.parent_session_id = ?1 ORDER BY s.start_time ASC",
            SESSION_SELECT, COST_EXPR
        ),
    )?;
    let children: Vec<Session> = child_stmt
        .query_map(params![session_id], |row| row_to_session_with_cost(row))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(SessionDetail {
        session,
        requests,
        children,
    })
}

/// Get cost rates
pub fn get_cost_rates(conn: &Connection) -> Result<Vec<CostRate>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT model, input_per_million, output_per_million,
                cache_read_per_million, cache_creation_per_million, effective_from
         FROM cost_rates ORDER BY model",
    )?;
    let results = stmt
        .query_map([], |row| {
            Ok(CostRate {
                model: row.get(0)?,
                input_per_million: row.get(1)?,
                output_per_million: row.get(2)?,
                cache_read_per_million: row.get(3)?,
                cache_creation_per_million: row.get(4)?,
                effective_from: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(results)
}

/// Insert or update a cost rate
pub fn upsert_cost_rate(conn: &Connection, rate: &CostRate) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO cost_rates (model, input_per_million, output_per_million,
                cache_read_per_million, cache_creation_per_million, effective_from)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(model) DO UPDATE SET
                input_per_million = excluded.input_per_million,
                output_per_million = excluded.output_per_million,
                cache_read_per_million = excluded.cache_read_per_million,
                cache_creation_per_million = excluded.cache_creation_per_million,
                effective_from = excluded.effective_from",
        params![
            rate.model,
            rate.input_per_million,
            rate.output_per_million,
            rate.cache_read_per_million,
            rate.cache_creation_per_million,
            rate.effective_from,
        ],
    )?;
    Ok(())
}

// ---- Helper functions ----

fn row_to_session_with_cost(row: &rusqlite::Row) -> Result<Session, rusqlite::Error> {
    Ok(Session {
        id: row.get(0)?,
        tool: row.get(1)?,
        source: row.get(2)?,
        parent_session_id: row.get(3)?,
        model: row.get(4)?,
        title: row.get(5)?,
        start_time: row.get(6)?,
        end_time: row.get(7)?,
        project_path: row.get(8)?,
        project_name: row.get(9)?,
        git_branch: row.get(10)?,
        git_sha: row.get(11)?,
        git_origin_url: row.get(12)?,
        cli_version: row.get(13)?,
        total_input_tokens: row.get(14)?,
        total_output_tokens: row.get(15)?,
        total_cache_read_tokens: row.get(16)?,
        total_cache_creation_tokens: row.get(17)?,
        total_reasoning_tokens: row.get(18)?,
        total_tokens: row.get(19)?,
        estimated_cost: row.get(20)?,
    })
}

fn build_session_where(
    filters: &SessionFilters,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut clauses = vec!["1=1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    let mut idx = 1;

    if let Some(ref t) = filters.tool {
        clauses.push(format!("s.tool = ?{}", idx));
        params.push(Box::new(t.clone()));
        idx += 1;
    }
    if let Some(ref s) = filters.source {
        clauses.push(format!("s.source = ?{}", idx));
        params.push(Box::new(s.clone()));
        idx += 1;
    }
    if let Some(ref sd) = filters.start_date {
        clauses.push(format!("s.start_time >= ?{}", idx));
        params.push(Box::new(sd.clone()));
        idx += 1;
    }
    if let Some(ref ed) = filters.end_date {
        clauses.push(format!("s.start_time <= ?{}", idx));
        params.push(Box::new(ed.clone()));
        idx += 1;
    }
    if let Some(min) = filters.token_min {
        clauses.push(format!("s.total_tokens >= ?{}", idx));
        params.push(Box::new(min));
        idx += 1;
    }
    if let Some(max) = filters.token_max {
        clauses.push(format!("s.total_tokens <= ?{}", idx));
        params.push(Box::new(max));
        idx += 1;
    }
    if let Some(ref models) = filters.model {
        if !models.is_empty() {
            let placeholders: Vec<String> = models
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", idx + i))
                .collect();
            clauses.push(format!("s.model IN ({})", placeholders.join(",")));
            for m in models {
                params.push(Box::new(m.clone()));
            }
            idx += models.len();
        }
    }
    if let Some(ref projects) = filters.project {
        if !projects.is_empty() {
            let placeholders: Vec<String> = projects
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", idx + i))
                .collect();
            clauses.push(format!("s.project_name IN ({})", placeholders.join(",")));
            for p in projects {
                params.push(Box::new(p.clone()));
            }
            idx += projects.len();
        }
    }
    if let Some(ref search) = filters.search {
        if !search.is_empty() {
            let pattern = format!("%{}%", search);
            clauses.push(format!(
                "(s.title LIKE ?{idx} OR s.project_name LIKE ?{idx} OR s.git_branch LIKE ?{idx})"
            ));
            params.push(Box::new(pattern));
        }
    }

    (clauses.join(" AND "), params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_test_db;

    fn insert_test_session(conn: &Connection, id: &str, tool: &str, date: &str, tokens: i64) {
        conn.execute(
            "INSERT INTO sessions (id, tool, model, start_time, total_tokens, total_input_tokens, total_output_tokens)
             VALUES (?1, ?2, 'test-model', ?3, ?4, ?5, ?6)",
            params![id, tool, format!("{}T12:00:00Z", date), tokens, tokens / 2, tokens / 2],
        )
        .unwrap();
    }

    #[test]
    fn test_dashboard_stats_empty() {
        let conn = init_test_db().unwrap();
        let stats = get_dashboard_stats(&conn, None, None, None).unwrap();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.total_tokens, 0);
        assert_eq!(stats.streak, 0);
        assert_eq!(stats.estimated_cost, 0.0);
    }

    #[test]
    fn test_dashboard_stats_with_data() {
        let conn = init_test_db().unwrap();
        insert_test_session(&conn, "claude:1", "claude", "2025-01-15", 1000);
        insert_test_session(&conn, "codex:1", "codex", "2025-01-15", 2000);

        let stats = get_dashboard_stats(&conn, None, None, None).unwrap();
        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.total_tokens, 3000);

        let claude_stats = get_dashboard_stats(&conn, Some("claude"), None, None).unwrap();
        assert_eq!(claude_stats.total_sessions, 1);
        assert_eq!(claude_stats.total_tokens, 1000);
    }

    #[test]
    fn test_cost_calculation() {
        let conn = init_test_db().unwrap();
        // Insert a session with known token values
        conn.execute(
            "INSERT INTO sessions (id, tool, model, start_time, total_tokens, total_input_tokens, total_output_tokens, total_cache_read_tokens)
             VALUES ('c:1', 'claude', 'claude-sonnet', '2025-01-15T00:00:00Z', 2000000, 1000000, 1000000, 500000)",
            [],
        ).unwrap();
        // Insert cost rate: $3/M input, $15/M output, $0.30/M cache read
        conn.execute(
            "INSERT INTO cost_rates (model, input_per_million, output_per_million, cache_read_per_million, cache_creation_per_million)
             VALUES ('claude-sonnet', 3.0, 15.0, 0.30, 3.75)",
            [],
        ).unwrap();

        let stats = get_dashboard_stats(&conn, None, None, None).unwrap();
        // non_cached_input = 1M - 500K = 500K, cost = (500K * 3 / 1M) + (500K * 0.30 / 1M) + (1M * 15 / 1M) = 1.5 + 0.15 + 15 = 16.65
        assert!((stats.estimated_cost - 16.65).abs() < 0.01);
    }

    #[test]
    fn test_daily_usage() {
        let conn = init_test_db().unwrap();
        insert_test_session(&conn, "c:1", "claude", "2025-01-15", 500);
        insert_test_session(&conn, "c:2", "claude", "2025-01-15", 300);
        insert_test_session(&conn, "x:1", "codex", "2025-01-16", 700);

        let daily = get_daily_usage(&conn, None, None, None).unwrap();
        assert_eq!(daily.len(), 2);
        assert_eq!(daily[0].date, "2025-01-15");
        assert_eq!(daily[0].total_tokens, 800);
    }

    #[test]
    fn test_model_breakdown() {
        let conn = init_test_db().unwrap();
        conn.execute(
            "INSERT INTO sessions (id, tool, model, start_time, total_tokens, total_input_tokens, total_output_tokens)
             VALUES ('c:1', 'claude', 'sonnet', '2025-01-15T00:00:00Z', 1000, 500, 500)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, tool, model, start_time, total_tokens, total_input_tokens, total_output_tokens)
             VALUES ('c:2', 'claude', 'opus', '2025-01-15T00:00:00Z', 2000, 1000, 1000)",
            [],
        ).unwrap();

        let breakdown = get_model_breakdown(&conn, None, None, None).unwrap();
        assert_eq!(breakdown.len(), 2);
        assert_eq!(breakdown[0].model, "opus");
        assert_eq!(breakdown[0].total_tokens, 2000);
    }

    #[test]
    fn test_session_pagination() {
        let conn = init_test_db().unwrap();
        for i in 0..10 {
            insert_test_session(
                &conn,
                &format!("s:{}", i),
                "claude",
                "2025-01-15",
                (10 - i) * 100,
            );
        }

        let filters = SessionFilters {
            tool: None, model: None, project: None, source: None,
            start_date: None, end_date: None, token_min: None, token_max: None,
            search: None, sort_by: Some("total_tokens".to_string()),
            sort_dir: Some("desc".to_string()), page: Some(1), page_size: Some(3),
        };
        let page = get_sessions(&conn, &filters).unwrap();
        assert_eq!(page.total_count, 10);
        assert_eq!(page.sessions.len(), 3);
        assert_eq!(page.sessions[0].total_tokens, 1000);
    }

    #[test]
    fn test_cost_rate_upsert() {
        let conn = init_test_db().unwrap();
        let rate = CostRate {
            model: "opus".to_string(),
            input_per_million: Some(15.0),
            output_per_million: Some(75.0),
            cache_read_per_million: Some(1.5),
            cache_creation_per_million: Some(18.75),
            effective_from: Some("2025-01-01".to_string()),
        };
        upsert_cost_rate(&conn, &rate).unwrap();

        let rates = get_cost_rates(&conn).unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].model, "opus");
        assert_eq!(rates[0].input_per_million, Some(15.0));

        let updated = CostRate {
            input_per_million: Some(20.0),
            ..rate
        };
        upsert_cost_rate(&conn, &updated).unwrap();
        let rates = get_cost_rates(&conn).unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].input_per_million, Some(20.0));
    }

    #[test]
    fn test_session_detail() {
        let conn = init_test_db().unwrap();
        insert_test_session(&conn, "c:parent", "claude", "2025-01-15", 1000);
        conn.execute(
            "INSERT INTO sessions (id, tool, model, parent_session_id, start_time, total_tokens, total_input_tokens, total_output_tokens)
             VALUES ('c:child', 'claude', 'test-model', 'c:parent', '2025-01-15T12:30:00Z', 200, 100, 100)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO requests (id, session_id, timestamp, input_tokens, output_tokens, total_tokens)
             VALUES ('r:1', 'c:parent', '2025-01-15T12:00:00Z', 300, 200, 500)",
            [],
        ).unwrap();

        let detail = get_session_detail(&conn, "c:parent").unwrap();
        assert_eq!(detail.session.id, "c:parent");
        assert_eq!(detail.requests.len(), 1);
        assert_eq!(detail.children.len(), 1);
        assert_eq!(detail.children[0].id, "c:child");
    }
}
