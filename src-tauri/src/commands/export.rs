use tauri::State;

use crate::commands::usage::DbState;

/// Quote a CSV field if it contains special characters (comma, quote, newline)
fn csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[tauri::command]
pub fn export_usage_csv(db: State<'_, DbState>, period: String) -> Result<String, String> {
    let records = db.export_usage_records(&period).map_err(|e| e.to_string())?;
    // UTF-8 BOM for Excel compatibility
    let mut csv = String::from("\u{FEFF}");
    csv.push_str("id,source,session_id,timestamp,model,input_tokens,output_tokens,cache_creation_tokens,cache_read_tokens,total_tokens,cost_usd,project_path\n");
    for r in &records {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            csv_field(&r.id),
            csv_field(r.source.as_str()),
            csv_field(&r.session_id),
            csv_field(&r.timestamp.to_rfc3339()),
            csv_field(&r.model),
            r.tokens.input_tokens,
            r.tokens.output_tokens,
            r.tokens.cache_creation_tokens,
            r.tokens.cache_read_tokens,
            r.tokens.total_tokens,
            r.cost_usd.unwrap_or(0.0),
            csv_field(r.project_path.as_deref().unwrap_or("")),
        ));
    }
    Ok(csv)
}

#[tauri::command]
pub fn export_usage_json(db: State<'_, DbState>, period: String) -> Result<String, String> {
    let records = db.export_usage_records(&period).map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&records).map_err(|e| e.to_string())
}
