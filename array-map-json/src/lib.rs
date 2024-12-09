use fluvio_smartmodule::{smartmodule, Result, SmartModuleRecord, RecordData};

/// Expend a JSON aggregate Record containing arrays into a list of individual JSON Records.
fn json_array_to_records(record: &SmartModuleRecord) -> Result<Vec<(Option<RecordData>, RecordData)>> {
    let array = serde_json::from_slice::<Vec<serde_json::Value>>(record.value.as_ref())?;

    // Convert each JSON value from the array back into a JSON string
    let strings: Vec<String> = array
        .into_iter()
        .map(|value| serde_json::to_string(&value))
        .collect::<core::result::Result<_, _>>()?;

    // Create one record from each JSON string to send
    let kvs: Vec<(Option<RecordData>, RecordData)> = strings
        .into_iter()
        .map(|s| (None, RecordData::from(s)))
        .collect();

    Ok(kvs)
}

#[smartmodule(array_map)]
pub fn array_map(record: &SmartModuleRecord) -> Result<Vec<(Option<RecordData>, RecordData)>> {
    let result = json_array_to_records(record)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluvio_smartmodule::Record;

    fn remove_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn array_of_two_test() {
        let input = r#"[
            {"one": 1}, 
            {"two": 2}
        ]"#;
        let expected = r#"[
            (None, {"one":1}), 
            (None, {"two":2})
        ]"#;
        let record = SmartModuleRecord::new(
            Record::new(input), 0, 0
        );

        let result = array_map(&record).unwrap();
        let output = format!("{:?}", result);

        assert_eq!(result.len(), 2);
        assert_eq!(remove_whitespace(output.as_str()), remove_whitespace(expected));
    }

    #[test]
    fn empty_array_test() {
        let input = "[]";
        let expected = "[]";
        let record = SmartModuleRecord::new(
            Record::new(input), 0, 0
        );

        let result = array_map(&record).unwrap();
        let output = format!("{:?}", result);

        assert_eq!(result.len(), 0);
        assert_eq!(output.as_str(), expected);
    }

}
