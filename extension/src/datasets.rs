use pgrx::{prelude::*, JsonB};

use crate::planner::EXPLAIN_PLANNER_MAP;

#[pg_extern]
fn pgl_qdataset_collect(
    dataset_name: &str,
    offset: i64,
    limit: i64,
    method: default!(String, "'default'"),
    arm: default!(i32, -1),
) -> anyhow::Result<TableIterator<'static, (name!(id, i32), name!(plan, JsonB))>> {
    let explain_configuer = EXPLAIN_PLANNER_MAP
        .get(method.as_str())
        .ok_or(anyhow::anyhow!("no such method"))?;

    let current_pos: Option<i64> = Spi::get_one_with_args(
        "SELECT current_pos FROM pgl.pgl_qdataset_status WHERE dataset_name = $1",
        &vec![pgrx::datum::DatumWithOid::from(dataset_name)],
    )?;

    let current_pos = current_pos.ok_or_else(|| anyhow::anyhow!("Dataset not found"))?;

    let effective_offset = if offset == -1 { current_pos } else { offset };
    let table_name = format!("pgl_qdataset_{}", dataset_name);
    let fetch_sql = format!(
        "SELECT content FROM pgl.{} ORDER BY id OFFSET $1 LIMIT $2",
        table_name
    );

    let mut explain_queries = Vec::new();
    let mut processed_count = 0;

    Spi::connect(|client| {
        let args = vec![
            pgrx::datum::DatumWithOid::from(effective_offset),
            pgrx::datum::DatumWithOid::from(limit),
        ];

        // 1. Fetch queries
        let res = client.select(&fetch_sql, None, &args)?;
        for row in res {
            if let Some(content) = row.get_by_name::<String, _>("content")? {
                explain_queries.push(format!(
                    "EXPLAIN (ANALYZE, FORMAT JSON, BUFFERS, COSTS, TIMING, SUMMARY) {}",
                    content
                ));
                processed_count += 1;
            }
        }
        Ok::<(), pgrx::spi::SpiError>(())
    })?;

    let explain_configer_state = (explain_configuer.setup)()?;
    let results = Spi::connect(|client| {
        let mut json_results = Vec::new();
        // 2. Run queries and capture output
        for query in explain_queries {
            // EXPLAIN output is one row with one column (usually TEXT/JSON)
            let args: Vec<pgrx::datum::DatumWithOid> = vec![];
            let explain_iter = (explain_configuer.make_iter)(&query, arm);

            for actual_query in explain_iter {
                let res = client.select(&actual_query, None, &args)?;
                if !res.is_empty() {
                    // Get the first column as JsonString (to handle OID 114/json)
                    if let Some(json_val) = res.first().get_one::<pgrx::datum::JsonString>()? {
                        let value = JsonB(
                            serde_json::from_str(json_val.0.as_str()).expect("Invalid JSON format"),
                        );
                        json_results.push(value);
                    }
                }
            }
        }

        Ok::<Vec<_>, pgrx::spi::SpiError>(json_results)
    })?;
    (explain_configuer.cleanup)(&explain_configer_state)?;

    // Update current_pos in pgl_qdataset_status
    // If offset was -1, effective_offset is the old current_pos.
    if processed_count > 0 {
        let new_pos = effective_offset + processed_count;
        let update_sql =
            "UPDATE pgl.pgl_qdataset_status SET current_pos = $1 WHERE dataset_name = $2";
        let args = vec![
            pgrx::datum::DatumWithOid::from(new_pos),
            pgrx::datum::DatumWithOid::from(dataset_name),
        ];
        Spi::run_with_args(update_sql, &args)?;
    }

    Ok(TableIterator::new(
        results.into_iter().enumerate().map(|(i, v)| (i as i32, v)),
    ))
}

#[pg_extern]
fn pgl_qdataset_create(dataset_name: &str) -> Result<(), pgrx::spi::SpiError> {
    let table_name = format!("pgl_qdataset_{}", dataset_name);

    let query = format!(
        "CREATE TABLE pgl.{} (id BIGSERIAL PRIMARY KEY, content TEXT);",
        table_name
    );

    Spi::run(&query)?;

    let insert_status_sql =
        "INSERT INTO pgl.pgl_qdataset_status (dataset_name, current_pos) VALUES ($1, 0)";
    let args = vec![pgrx::datum::DatumWithOid::from(dataset_name)];
    Spi::run_with_args(insert_status_sql, &args)?;

    Ok(())
}

#[pg_extern]
fn pgl_qdataset_delete(dataset_name: &str) -> Result<(), pgrx::spi::SpiError> {
    let table_name = format!("pgl_qdataset_{}", dataset_name);

    let query = format!("DROP TABLE IF EXISTS pgl.{};", table_name);
    Spi::run(&query)?;

    let delete_status_sql = "DELETE FROM pgl.pgl_qdataset_status WHERE dataset_name = $1";
    let args = vec![pgrx::datum::DatumWithOid::from(dataset_name)];
    Spi::run_with_args(delete_status_sql, &args)?;

    Ok(())
}

#[pg_extern]
fn pgl_qdataset_insert(dataset_name: &str, query: &str) -> Result<(), pgrx::spi::SpiError> {
    let table_name = format!("pgl_qdataset_{}", dataset_name);

    // Use DatumWithOid::from to correctly construct arguments
    let args = vec![pgrx::datum::DatumWithOid::from(query)];
    let sql = format!("INSERT INTO pgl.{} (content) VALUES ($1)", table_name);

    Spi::run_with_args(&sql, &args)?;
    Ok(())
}

#[pg_extern]
fn pgl_qdataset_import(dataset_name: &str, filepath: &str) -> anyhow::Result<()> {
    let table_name = format!("pgl_qdataset_{}", dataset_name);

    let content = std::fs::read_to_string(filepath)?;

    let sql = format!("INSERT INTO pgl.{} (content) VALUES ($1)", table_name);

    for query in content.split(';') {
        let query = query.trim();
        if query.is_empty() {
            continue;
        }
        let args = vec![pgrx::datum::DatumWithOid::from(query)];
        Spi::run_with_args(&sql, &args)?;
    }
    Ok(())
}
