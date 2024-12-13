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

#[smartmodule(array_map)]
pub fn array_map(record: &SmartModuleRecord) -> Result<Vec<(Option<RecordData>, RecordData)>> {
    let params = PARAMS.get().expect("params is not initialized");

    let processed_records = process_csv_records(record, &params)?;
    
    let result: Vec<(Option<RecordData>, RecordData)> = processed_records
        .into_iter()
        .map(|opt| match opt {
            Some(record_data) => (Some(record_data.clone()), record_data),
            None => (None, RecordData::from(vec![])),
        })
        .collect();

    Ok(result)
}


/// Process CSV record and convert each row into an individual JSON record.
fn process_csv_records(record: &SmartModuleRecord, params: &Params) -> Result<Vec<Option<RecordData>>> {
    // Initialize CSV reader with the specified delimiter and other settings
    let mut csv_reader = ReaderBuilder::new()
        .delimiter(params.delimiter)
        .has_headers(true)
        .trim(Trim::All)
        .from_reader(record.value.as_ref());

    let mut rows: Vec<Option<RecordData>> = Vec::new();

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

        // Serialize the JSON object for this row and wrap it in RecordData
        let serialized_row = serde_json::to_vec(&json_object)?;

        // Wrap in Option<RecordData> and push to rows
        rows.push(Some(RecordData::from(serialized_row)));
    }

    Ok(rows)
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

    // Utility function to convert Vec<Option<RecordData>> to Vec<u8>
    fn convert_records_to_json(result: Vec<Option<RecordData>>) -> Vec<u8> {
        let result_json: Vec<Value> = result
            .into_iter()
            .filter_map(|record| record.map(|r| {
                // Convert RecordData into JSON object
                serde_json::from_slice::<Value>(&r.to_vec()).unwrap()
            }))
            .collect();
    
        serde_json::to_vec(&result_json).unwrap()
    }

    #[test]
    fn test_default_conversion() {
        let csv_data = "name,age\n".to_string() + &vec!["Alice", "Bob", "Charlie", "David", "Eve"]
            .into_iter()
            .map(|name| format!("{},30", name))
            .collect::<Vec<String>>()
            .join("\n");

        let record = SmartModuleRecord::new(
            Record::new(csv_data), 0, 0
        );
        let params = Params::new(DEFAULT_DELIMITER, HeaderCase::None);

        // Test Conversion
        let computed = process_csv_records(&record, &params).unwrap();
        let computed_output = convert_records_to_json(computed);

        // Prepare Result
        let expected_json = vec![
            json!({"name": "Alice", "age": "30"}),
            json!({"name": "Bob", "age": "30"}),
            json!({"name": "Charlie", "age": "30"}),
            json!({"name": "David", "age": "30"}),
            json!({"name": "Eve", "age": "30"}),
        ];
        let expected_output = serde_json::to_vec(&expected_json).unwrap();
        
        assert_eq!(computed_output, expected_output);
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

        // Test Conversion
        let computed = process_csv_records(&record, &params).unwrap();
        let computed_output = convert_records_to_json(computed);
        
        // Prepare Result
        let expected_json = vec![
            json!({"firstName": "Alice", "lastName": "Smith"}),
            json!({"firstName": "Bob", "lastName": "Johnson"}),
        ];
        let expected_output = serde_json::to_vec(&expected_json).unwrap();

        assert_eq!(computed_output, expected_output);
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
        let computed = process_csv_records(&record, &params).unwrap();
        let computed_output = convert_records_to_json(computed);
        
        // Prepare Result
        let expected_json = vec![
            json!({"first_name": "Alice", "last_name": "Smith"}),
            json!({"first_name": "Bob", "last_name": "Johnson"}),
        ];
        let expected_output = serde_json::to_vec(&expected_json).unwrap();

        assert_eq!(computed_output, expected_output);
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
        let computed = process_csv_records(&record, &params).unwrap();
        let computed_output = convert_records_to_json(computed);
        
        // Prepare Result
        let expected_json: Vec<Value> = vec![];
        let expected_output = serde_json::to_vec(&expected_json).unwrap();

        assert_eq!(computed_output, expected_output);
    }    
}
