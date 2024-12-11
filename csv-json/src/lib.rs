use csv::{ReaderBuilder, Trim};
use fluvio_smartmodule::{
    dataplane::smartmodule::SmartModuleExtraParams, smartmodule, RecordData, Result,
    SmartModuleRecord,
};
use heck::{ToLowerCamelCase, ToSnakeCase};
use serde_json::{json, Value};
use std::sync::OnceLock;

static PARAMS: OnceLock<Params> = OnceLock::new();
const DELIMITER_PARAM_NAME: &str = "delimiter";
const HEADER_CASE_PARAM_NAME: &str = "header_case";
const DEFAULT_DELIMITER: u8 = b',';

#[smartmodule(map)]
pub fn map(record: &SmartModuleRecord) -> Result<(Option<RecordData>, RecordData)> {
    let params = PARAMS.get().expect("params is not initialized");

    let key = record.key.clone();
    let value = process_csv_record(record, &params)?;
    
    Ok((key, RecordData::from(value)))
}

fn process_csv_record(record: &SmartModuleRecord, params: &Params) -> Result<Vec<u8>> {
    // Initialize CSV reader with the specified delimiter and other settings
    let mut csv_reader = ReaderBuilder::new()
        .delimiter(params.delimiter)
        .has_headers(true)
        .trim(Trim::All)
        .from_reader(record.value.as_ref());

    let mut rows: Vec<Value> = Vec::new();

    // Collect headers and apply the header case transformation
    let headers: Vec<String> = csv_reader
        .headers()?
        .iter()
        .map(|h| match params.header_case {
            HeaderCase::Camel => h.to_lower_camel_case(),
            HeaderCase::Snake => h.to_snake_case(),
            HeaderCase::None => h.to_string(),
        })
        .collect();

    // Collect all records as Vec<Vec<String>>
    let records: Vec<Vec<String>> = csv_reader.records()
        .map(|record| record.unwrap().iter().map(|s| s.to_string()).collect())
        .collect();

    // Iterate over all records and convert each to a JSON object
    for record in records.iter() {
        let json_object: Value = headers
            .iter()
            .zip(record.iter())
            .map(|(key, value)| (key.clone(), json!(value)))
            .collect();
        rows.push(json_object);
    }

    // Serialize the rows into a JSON byte vector
    let serialized_output = serde_json::to_vec(&rows)?;

    Ok(serialized_output)
}

#[smartmodule(init)]
fn init(params: SmartModuleExtraParams) -> Result<()> {
    let delimiter_param = params
        .get(DELIMITER_PARAM_NAME)
        .map_or(DEFAULT_DELIMITER, |v| {
            v.chars().next().expect("delimiter is empty") as u8
        });

    let case_param = params
        .get(HEADER_CASE_PARAM_NAME)
        .map_or(HeaderCase::None, |v| {
            v.to_string().try_into().unwrap_or_else(|e| {
                panic!("failed to parse header case: {}", e);
            })
        });

    PARAMS
        .set(Params::new(delimiter_param, case_param))
        .expect("params is already initialized");

    Ok(())
}


#[derive(Debug)]
struct Params {
    delimiter: u8,
    header_case: HeaderCase,
}

#[derive(Debug)]
enum HeaderCase {
    Camel,
    Snake,
    None,
}

impl TryFrom<String> for HeaderCase {
    type Error = &'static str;
    fn try_from(s: String) -> core::result::Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "camel" => Ok(HeaderCase::Camel),
            "snake" => Ok(HeaderCase::Snake),
            "" | "none" => Ok(HeaderCase::None),
            _ => Err("Invalid header case"),
        }
    }
}

impl Params {
    fn new(delimiter: u8, header_case: HeaderCase) -> Self {
        Self {
            delimiter,
            header_case,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluvio_smartmodule::{Record, RecordData, SmartModuleRecord};
    use fluvio_smartmodule::dataplane::record::RecordHeader;
    use serde_json::json;

    #[test]
    fn test_default_conversion() {
        let csv_data = "name,age\n".to_string() + &vec!["Alice", "Bob", "Charlie", "David", "Eve"]
            .into_iter()
            .map(|name| format!("{},30", name))
            .collect::<Vec<String>>()
            .join("\n");

        let data_record = Record {
            preamble: RecordHeader::default(),
            key: None,
            value: RecordData::from(csv_data.as_bytes().to_vec()),
            headers: 0,
        };
        let record = SmartModuleRecord::new(data_record, 0, 0);
        let params = Params::new(DEFAULT_DELIMITER, HeaderCase::None);

        // Test Conversion
        let result =process_csv_record(&record, &params).unwrap();
        
        // Prepare Result
        let expected_json = vec![
            json!({"name": "Alice", "age": "30"}),
            json!({"name": "Bob", "age": "30"}),
            json!({"name": "Charlie", "age": "30"}),
            json!({"name": "David", "age": "30"}),
            json!({"name": "Eve", "age": "30"}),
        ];
        let expected_output = serde_json::to_vec(&expected_json).unwrap();
        
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_camel_case_conversion() {
        let csv_data = "first_name,last_name\nAlice,Smith\nBob,Johnson\n";
        let data_record = Record {
            preamble: RecordHeader::default(),
            key: None,
            value: RecordData::from(csv_data.as_bytes().to_vec()),
            headers: 0,
        };
        let record = SmartModuleRecord::new(data_record, 0, 0);
        let params = Params::new(DEFAULT_DELIMITER, HeaderCase::Camel);

        // Compute CSV
        let result =process_csv_record(&record, &params).unwrap();
        
        // Prepare Result
        let expected_json = vec![
            json!({"firstName": "Alice", "lastName": "Smith"}),
            json!({"firstName": "Bob", "lastName": "Johnson"}),
        ];
        let expected_output = serde_json::to_vec(&expected_json).unwrap();

        assert_eq!(result, expected_output);
    }  

    #[test]
    fn test_snake_case_conversion() {
        let csv_data = "firstName,lastName\nAlice,Smith\nBob,Johnson\n";
        let data_record = Record {
            preamble: RecordHeader::default(),
            key: None,
            value: RecordData::from(csv_data.as_bytes().to_vec()),
            headers: 0,
        };
        let record = SmartModuleRecord::new(data_record, 0, 0);
        let params = Params::new(DEFAULT_DELIMITER, HeaderCase::Snake);

        // Test Conversion
        let result = process_csv_record(&record, &params).unwrap();
        
        // Prepare Result
        let expected_json = vec![
            json!({"first_name": "Alice", "last_name": "Smith"}),
            json!({"first_name": "Bob", "last_name": "Johnson"}),
        ];
        let expected_output = serde_json::to_vec(&expected_json).unwrap();

        assert_eq!(result, expected_output);
    }  

    #[test]
    fn test_empty_csv() {
        let csv_data = "";
        let data_record = Record {
            preamble: RecordHeader::default(),
            key: None,
            value: RecordData::from(csv_data.as_bytes().to_vec()),
            headers: 0,
        };
        let record = SmartModuleRecord::new(data_record, 0, 0);
        let params = Params::new(DEFAULT_DELIMITER, HeaderCase::None);

        // Test Conversion
        let result = process_csv_record(&record, &params).unwrap();
        
        // Prepare Result
        let expected_json: Vec<Value> = vec![];
        let expected_output = serde_json::to_vec(&expected_json).unwrap();

        assert_eq!(result, expected_output);
    }
}
