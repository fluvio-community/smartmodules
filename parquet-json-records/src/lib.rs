use fluvio_smartmodule::{smartmodule, Result, eyre, SmartModuleRecord, RecordData};
use serde_json::Value;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::{Field, List, Map, Row};
use parquet::data_type::Decimal;
use bytes::Bytes;
use chrono::{NaiveDate, DateTime};

#[smartmodule(array_map)]
pub fn array_map(record: &SmartModuleRecord) -> Result<Vec<(Option<RecordData>, RecordData)>> {
    let result = process_parquet_data(record.value.as_ref())?;
    Ok(result)
}

/// Process Parquet data from a byte slice and produce JSON records
fn process_parquet_data(data: &[u8]) -> Result<Vec<(Option<RecordData>, RecordData)>> {
    // Convert the data slice to Bytes
    let bytes = Bytes::copy_from_slice(data);

    // Initialize a Parquet reader from Bytes
    let reader = SerializedFileReader::new(bytes)
        .map_err(|e| eyre!("Failed to read Parquet data: {}", e))?;

    let mut records = Vec::new();

    // Iterate over rows and convert each to JSON
    for row_result in reader.get_row_iter(None)
        .map_err(|e| eyre!("Failed to iterate over rows: {}", e))? 
    {
        let row = row_result.map_err(|e| eyre!("Error reading row: {}", e))?;
        let json_value = row_to_json(&row);
        let json_string = serde_json::to_string(&json_value)
            .map_err(|e| eyre!("Failed to serialize JSON: {}", e))?;
        records.push((None, RecordData::from(json_string)));
    }

    Ok(records)
}

/// Convert a Parquet row to JSON
fn row_to_json(row: &Row) -> Value {
    let mut json_object = serde_json::Map::new();

    for (key, field) in row.get_column_iter() {
        let json_value = field_to_json(&field);
        json_object.insert(key.to_string(), json_value);
    }

    Value::Object(json_object)
}

/// Convert individual Field to JSON
fn field_to_json(field: &Field) -> Value {
    match field {
        Field::Null => Value::Null,
        Field::Bool(v) => Value::Bool(*v),
        Field::Byte(v) => Value::Number((*v as i64).into()),
        Field::Short(v) => Value::Number((*v as i64).into()),
        Field::Int(v) => Value::Number((*v as i64).into()),
        Field::Long(v) => Value::Number((*v).into()),
        Field::UByte(v) => Value::Number((*v as u64).into()),
        Field::UShort(v) => Value::Number((*v as u64).into()),
        Field::UInt(v) => Value::Number((*v as u64).into()),
        Field::ULong(v) => Value::Number((*v).into()),
        Field::Float(v) => serde_json::Number::from_f64(*v as f64)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        Field::Float16(v) => serde_json::Number::from_f64(v.to_f64())
            .map(Value::Number)            
            .unwrap_or(Value::Null),
        Field::Double(v) => serde_json::Number::from_f64(*v)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        Field::Str(v) => Value::String(v.clone()),
        Field::Bytes(v) => Value::String(hex::encode(v)),
        Field::Decimal(decimal) => decimal_to_json(decimal),
        Field::Date(days) => date_to_json(*days),
        Field::TimestampMillis(ts) => timestamp_to_json(*ts, 1_000),
        Field::TimestampMicros(ts) => timestamp_to_json(*ts, 1_000_000),
        Field::MapInternal(map) => map_to_json(map),
        Field::Group(group) => group_to_json(group),
        Field::ListInternal(list) => list_to_json(list),
    }
}

/// Convert Decimal to JSON
fn decimal_to_json(decimal: &Decimal) -> Value {
    match decimal {
        Decimal::Int32 { value, .. } => {
            let num = i32::from_be_bytes(*value);
            Value::String(num.to_string())
        }
        Decimal::Int64 { value, .. } => {
            let num = i64::from_be_bytes(*value);
            Value::String(num.to_string())
        }
        Decimal::Bytes { value, .. } => {
            Value::String(format!("0x{}", hex::encode(value)))
        }
    }
}

/// Convert Date (days since epoch) to JSON
fn date_to_json(days: i32) -> Value {
    match NaiveDate::from_num_days_from_ce_opt(days + 719_163) {
        Some(naive_date) => Value::String(naive_date.to_string()),
        None => Value::Null,
    }
}

/// Convert Timestamp to JSON
fn timestamp_to_json(timestamp: i64, divisor: i64) -> Value {
    let naive_datetime = DateTime::from_timestamp(
        timestamp / divisor,
        ((timestamp % divisor) * 1_000_000) as u32, // Nanoseconds as i32
    );
    match naive_datetime {
        Some(datetime) => Value::String(datetime.to_string()),
        None => Value::Null,
    }
}

/// Convert Map to JSON
fn map_to_json(map: &Map) -> Value {
    let mut json_object = serde_json::Map::new();
    for (key, value) in map.entries() {
        json_object.insert(key.to_string(), field_to_json(value));
    }
    Value::Object(json_object)
}

/// Convert Group to JSON
fn group_to_json(group: &Row) -> Value {
    row_to_json(group) // Recursively process nested rows
}

/// Convert List to JSON
fn list_to_json(list: &List) -> Value {
    let json_array: Vec<Value> = list.elements().iter().map(field_to_json).collect();
    Value::Array(json_array)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_process_parquet_data_with_mtcars() -> Result<()> {
        let file_path = "test-data/mtcars.parquet";
        let mut file = File::open(file_path).expect("Failed to open file");
        let mut parquet_data = Vec::new();
        file.read_to_end(&mut parquet_data).expect("Failed to read file");

        // Call the function under test
        let records = process_parquet_data(&parquet_data)?;

        // Validate the output
        assert!(!records.is_empty(), "The dataset should not be empty");

        // Print and check the first record for debugging purposes
        if let Some((_, record_data)) = records.first() {
            println!("First record: {:?}", record_data);
        }

        // Further validations (example: check specific values if known)
        assert_eq!(records.len(), 32, "The dataset should have 32 rows");

        Ok(())
    }
}
