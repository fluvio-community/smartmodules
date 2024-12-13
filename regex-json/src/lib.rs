use regex::Regex;
use once_cell::sync::OnceCell;
use eyre::ContextCompat;
use serde::Deserialize;
use serde_json::Value;

use fluvio_smartmodule::{
    smartmodule, Result, SmartModuleRecord, RecordData,
    dataplane::smartmodule::{
        SmartModuleExtraParams, SmartModuleInitError
    },
    eyre
};

static OPS: OnceCell<Vec<Operation>> = OnceCell::new();
const PARAM_NAME: &str = "spec";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Operation {
    Capture(Capture),
    Replace(Replace)
}

#[derive(Debug, Deserialize)]
struct Capture {
    #[serde(with = "serde_regex")]
    regex: Regex,
    target: String,
    output: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Replace {
    #[serde(with = "serde_regex")]
    regex: Regex,
    target: String,
    with: String,
}

impl Operation {
    pub fn get_target(&self) -> &String {
        match self {
            Operation::Capture(c) => &c.target,
            Operation::Replace(r) => &r.target
        }
    }

    pub fn get_output(&self) -> &String {
        match self {
            Operation::Capture(c) => &c.output,
            Operation::Replace(r) => &r.target
        }
    }

    pub fn run_regex(&self, text: &String) -> Result<String> {
        let result = match self {
            Operation::Capture(c) => {
                process_regex_capture(text, &c.regex)?
            },
            Operation::Replace(r) => {
                process_regex_replace(text, &r.regex, &r.with)?
            }
        };
        Ok(result)
    }

}

/// Parse input paramters
fn get_params(params: SmartModuleExtraParams) -> Result<Vec<Operation>> {
    if let Some(raw_spec) = params.get(PARAM_NAME) {
        match serde_json::from_str(raw_spec) {
            Ok(operations) => {
                Ok(operations)
            }
            Err(err) => {
                eprintln!("unable to parse spec from params: {err:?}");
                Err(eyre!("cannot parse `spec` param: {:#?}", err))
            }
        }
    } else {
        Err(SmartModuleInitError::MissingParam(PARAM_NAME.to_string()).into())
    }
}

/// Extract json value based on JSON pointer notations:
///     [ "/top/one", "/top/two"]
fn extract_json_field(data: &str, lookup: &String) -> Result<String> {
    let json:Value = serde_json::from_str(data)?;

    // Extract value and convert to string, return empty string if none.
    let result = if let Some(val) = json.pointer(lookup.as_str()) {
        match val.as_str() {
            Some(s) => s.to_owned(),
            None => val.to_string()
        }
    } else {
        "".to_owned()
    };

    Ok(result)
}

/// Run regex `capture` and return the result
fn process_regex_capture(text: &String, regex: &Regex) -> Result<String> {
    let capture = match regex.captures(text.as_str()) {
        Some(caps) => caps.get(1).map_or("", |m| m.as_str()),
        None => ""
    };

    Ok(capture.to_string())
}

/// Run regex `replace` and return the result
fn process_regex_replace(text: &String, regex: &Regex, with: &String) -> Result<String> {
    Ok(regex.replace_all(text, with).to_string())
}

/// Merge json trees
fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

/// Recursive function that traverses the json tree to insert the value at proper hierarchy
///     "/root/one" -> "test" - inserts {"root": {"one": "text"}}
///     "/root" -> "test" - inserts {"root": "text"}
/// Note, if the path matches an existing value exists, that value is replaced.
fn add_json_key_value_recursive(json: &mut Value, key_path: &String, new_value: Value ) {
    // Check json path
    // Found a match, merge json objects at this hiearchy
    let some_found_json = json.pointer_mut(key_path.as_str());
    if some_found_json.is_some() {
        return merge_json(some_found_json.unwrap(), &new_value);
    }
    
    // Peal off the leaf 
    // Use as key
    let mut path_array:Vec<_> = key_path.split('/').skip(1).collect();
    let some_key = path_array.pop();

    // No key (no neaf)
    // Merge with the top of the tree
    if some_key.is_none() {
        return merge_json(json, &new_value);
    }

    // Have key
    // Create new value storing previous key/val, and go again
    let mut v_map = serde_json::Map::new();
    v_map.insert(some_key.unwrap().to_owned(), new_value);
    let r_val = Value::Object(v_map);

    // No more path elements
    // Merge new value with the top of the tree
    if path_array.is_empty()  {
        return merge_json(json, &r_val);
    }

    // Build parent key-path and try again
    let new_key_path = format!("/{}", path_array.join("/"));
    add_json_key_value(json, &new_key_path, r_val);

}

/// Project badly formatted key_path (without /) from wipping out the existing json
/// Improperly formatted destinations, leave the original json untouched
fn add_json_key_value(json: &mut Value, key_path: &String, new_value: Value ) {
    if key_path.contains("/") {
        add_json_key_value_recursive(json, key_path, new_value)
    }
}

/// Traverse the regex list, extract JSON values, compute regex, and save output
fn apply_regex_ops_to_json_record(record: &SmartModuleRecord, ops: &Vec<Operation>) -> Result<Value> {
    let data: &str = std::str::from_utf8(record.value.as_ref())?;
    let mut json:Value = serde_json::from_str(data)?;

    let mut iter = ops.into_iter();
    while let Some(op) = iter.next() {
        // Skip if source doesn't exist
        let value = extract_json_field(data, &op.get_target())?;
        if value.is_empty() {
            continue;
        }

        // Skip if regex match empty string
        let result = op.run_regex(&value)?;
        if result.is_empty() {
            continue;
        }

        // update json record with the new values
        add_json_key_value(
            &mut json, 
            op.get_output(), 
            Value::from(result)
        );
    }

    Ok(json)
}    

#[smartmodule(map)]
pub fn map(record: &SmartModuleRecord) -> Result<(Option<RecordData>, RecordData)> {
    let key = record.key.clone();
    let ops = OPS.get().wrap_err("regex operations not initialized")?;

    let result = apply_regex_ops_to_json_record(record, ops)?;
    Ok((key, serde_json::to_string(&result)?.into()))
}

#[smartmodule(init)]
fn init(params: SmartModuleExtraParams) -> Result<()> {
    let ops = get_params(params)?;

    OPS.set(ops).expect("regex operations already initialized");

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use fluvio_smartmodule::Record;

    static INPUT: &str = r#"{
        "dedup_key": "6fcb9fe530c24613ed1df3e51c0e86addd794251f49ec6cd77fd4381cc0e0ac2",
        "description": "First: bk Second: 4 Third: 13 Fourth: Jack, tr Sec  [Encased string - (data)] (<a href='https://example.com/doc1/182031340621?pdf_header=&de_seq_num=44&caseid=456177'>9</a>)",
        "last_build_date": "Tue, 18 Apr 2023 15:00:01 GMT",
        "link": "https://www.example.comv/cgi-bin/DktRpt.pl?456177",
        "pub_date": "Mon, 17 Apr 2023 15:54:45 GMT",
        "title": "23-20670 Abby Lynn Hardy",
        "name": {
            "first": "Abby",
            "last": "Hardy",
            "ssn": "123-45-6789"
        }
    }"#;

    #[test]
    fn extract_json_field_tests() {
        // string
        let lookup = "/description".to_owned();
        let result: &str = r#"First: bk Second: 4 Third: 13 Fourth: Jack, tr Sec  [Encased string - (data)] (<a href='https://example.com/doc1/182031340621?pdf_header=&de_seq_num=44&caseid=456177'>9</a>)"#;
        assert_eq!(result.to_owned(), extract_json_field(INPUT, &lookup).unwrap());

        // nested node
        let lookup = "/name/last".to_owned();
        let result = "Hardy";
        assert_eq!(result.to_owned(), extract_json_field(INPUT, &lookup).unwrap());

        // nested tree
        let lookup = "/name".to_owned();
        let result = r#"{"first": "Abby", "last": "Hardy", "ssn":"123-45-6789"}"#;
        let expected: Value = serde_json::from_str(result).unwrap();
        assert_eq!(expected.to_string(), extract_json_field(INPUT, &lookup).unwrap());

        // invalid 
        let lookup = "/invalid".to_owned();
        let result = "";
        assert_eq!(result.to_owned(), extract_json_field(INPUT, &lookup).unwrap());
    }

    #[test]
    fn process_regex_capture_test() {
        let input: &str = r#"First: bk Second: 4 Third: 13 Fourth: Jack, tr Sec  [Encased string - (data)] (<a href='https://example.com/doc1/182031340621?pdf_header=&de_seq_num=44&caseid=456177'>9</a>)"#;

        // First
        let re = Regex::new(r"(?i)First:\s+(\w+)\b").unwrap();
        let expected = "bk".to_owned();

        let result = process_regex_capture(&input.to_owned(), &re);
        assert_eq!(result.unwrap(), expected);

        // Second
        let re = Regex::new(r"(?i)Second:\s+(\w+)\b").unwrap();
        let expected = "4".to_owned();

        let result = process_regex_capture(&input.to_owned(), &re);
        assert_eq!(result.unwrap(), expected);

        // Third
        let re = Regex::new(r"(?i)Third:\s+(\w+)\b").unwrap();
        let expected = "13".to_owned();

        let result = process_regex_capture(&input.to_owned(), &re);
        assert_eq!(result.unwrap(), expected);

        // Fourth
        let re = Regex::new(r"(?i)Fourth:\s+([\w,\s\.\']*\S)\s*\[").unwrap();
        let expected = "Jack, tr Sec".to_owned();

        let result = process_regex_capture(&input.to_owned(), &re);
        assert_eq!(result.unwrap(), expected);

        // doc-link
        let re = Regex::new(r"href='([^']+)'").unwrap();
        let expected = "https://example.com/doc1/182031340621?pdf_header=&de_seq_num=44&caseid=456177".to_owned();

        let result = process_regex_capture(&input.to_owned(), &re);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn process_regex_replace_test() {

        // Replace all
        let input = r"123-45-6789".to_owned();
        let re = Regex::new(r"\d{3}-\d{2}-\d{4}").unwrap();
        let with = "***-**-****".to_owned();
        let expected = "***-**-****".to_owned();

        let result = process_regex_replace(&input, &re, &with);
        assert_eq!(result.unwrap(), expected);

        // Replace subset
        let input = r"Alice Jackson, ssn 123-45-6789, location: NY".to_owned();
        let re = Regex::new(r"\d{3}-\d{2}-\d{4}").unwrap();
        let with ="***-**-****".to_owned();
        let expected = "Alice Jackson, ssn ***-**-****, location: NY".to_owned();

        let result = process_regex_replace(&input, &re, &with);
        assert_eq!(result.unwrap(), expected);

        // Replace none
        let input = r"not a match".to_owned();
        let re = Regex::new(r"\d{3}-\d{2}-\d{4}").unwrap();
        let with ="***-**-****".to_owned();
        let expected = "not a match".to_owned();

        let result = process_regex_replace(&input, &re, &with);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn add_json_key_value_test() {

        // Test: Empty tree
        let mut json:Value = serde_json::from_str(r#"{}"#).unwrap();
        let key_path = "/root".to_owned();
        let new_v:Value = serde_json::json!("xyz");
        let expected:Value = serde_json::from_str(r#"{"root": "xyz"}"#).unwrap();

        add_json_key_value(&mut json, &key_path, new_v);
        assert_eq!(json, expected);

        // Test: Empty tree
        let mut json:Value = serde_json::json!("");
        let key_path = "/root".to_owned();
        let new_v:Value = serde_json::json!("xyz");
        let expected:Value = serde_json::from_str(r#"{"root": "xyz"}"#).unwrap();

        add_json_key_value(&mut json, &key_path, new_v);
        assert_eq!(json, expected);

        // Test: Invalid Node
        let mut json:Value = serde_json::json!("");
        let key_path = "root".to_owned();
        let new_v:Value = serde_json::json!("xyz");
        let expected:Value = serde_json::json!("");

        add_json_key_value(&mut json, &key_path, new_v);
        assert_eq!(json, expected);

        // Test: Add peer leaf
        let mut json:Value = serde_json::from_str(r#"{"root": {"aaa" : 1 , "bbb": 2}}"#).unwrap();
        let key_path = "/root/ccc".to_owned();
        let new_v:Value = serde_json::json!(3);
        let expected :Value = serde_json::from_str(r#"{"root": {"aaa" : 1 , "bbb": 2, "ccc": 3}}"#).unwrap();

        add_json_key_value(&mut json, &key_path, new_v);
        assert_eq!(json, expected);

        // Test: Add peer middle leave
        let mut json:Value = serde_json::from_str(r#"{"root": {"aaa" : {"bbb": 2}}}"#).unwrap();
        let key_path = "/root/ccc".to_owned();
        let new_v:Value = serde_json::json!(3);
        let expected :Value = serde_json::from_str(r#"{"root": {"aaa" : {"bbb": 2}, "ccc": 3}}"#).unwrap();

        add_json_key_value(&mut json, &key_path, new_v);
        assert_eq!(json, expected);

        // Test: Add deep nested leave
        let mut json:Value = serde_json::from_str(r#"{"root": {"aaa" : {"bbb": 2}}}"#).unwrap();
        let key_path = "/root/aaa/ccc".to_owned();
        let new_v:Value = serde_json::json!(3);
        let expected :Value = serde_json::from_str(r#"{"root": {"aaa" : {"bbb": 2, "ccc": 3}}}"#).unwrap();

        add_json_key_value(&mut json, &key_path, new_v);
        assert_eq!(json, expected);

        // Test: Swap content
        let mut json:Value = serde_json::from_str(r#"{"root": [{"aaa" : 1} , {"bbb": 2}]}"#).unwrap();
        let key_path = "/root/ccc".to_owned();
        let new_v:Value = serde_json::json!(3);
        let expected :Value = serde_json::from_str(r#"{"root": {"ccc": 3}}"#).unwrap();

        add_json_key_value(&mut json, &key_path, new_v);
        assert_eq!(json, expected);

    }

    #[test]
    fn apply_regex_ops_to_json_record_tests() {
        static EXPECTED: &str = r#"{
            "dedup_key": "6fcb9fe530c24613ed1df3e51c0e86addd794251f49ec6cd77fd4381cc0e0ac2",
            "description": "First: bk Second: 4 Third: 13 Fourth: Jack, tr Sec  [Encased string - (data)] (<a href='https://example.com/doc1/182031340621?pdf_header=&de_seq_num=44&caseid=456177'>9</a>)",
            "last_build_date": "Tue, 18 Apr 2023 15:00:01 GMT",
            "link": "https://www.example.comv/cgi-bin/DktRpt.pl?456177",
            "pub_date": "Mon, 17 Apr 2023 15:54:45 GMT",
            "title": "23-20670 Abby Lynn Hardy",
            "name": {
                "first": "Abby", 
                "last": "Hardy",
                "ssn": "***-**-****"
            },
            "parsed": {
                "first": "bk",
                "second": "4",
                "third": "13",
                "fourth": "Jack, tr Sec",
                "doc-link": "https://example.com/doc1/182031340621?pdf_header=&de_seq_num=44&caseid=456177"
            }
        }"#;
        let ops: Vec<Operation> = vec![
            Operation::Capture(Capture {
                regex: Regex::new(r"(?i)First:\s+(\w+)\b").unwrap(), 
                target: "/description".to_owned(), 
                output: "/parsed/first".to_owned()
            }),
            Operation::Capture(Capture {
                regex: Regex::new(r"(?i)Second:\s+(\w+)\b").unwrap(), 
                target: "/description".to_owned(), 
                output: "/parsed/second".to_owned()
            }),
            Operation::Capture(Capture {
                regex: Regex::new(r"(?i)Third:\s+(\w+)\b").unwrap(), 
                target: "/description".to_owned(), 
                output: "/parsed/third".to_owned()
            }),
            Operation::Capture(Capture {
                regex: Regex::new(r"(?i)Fourth:\s+([\w,\s\.\']*\S)\s*\[").unwrap(), 
                target: "/description".to_owned(), 
                output: "/parsed/fourth".to_owned()
            }),
            Operation::Capture(Capture {
                regex: Regex::new(r"href='([^']+)'").unwrap(), 
                target: "/description".to_owned(), 
                output: "/parsed/doc-link".to_owned()
            }),
            Operation::Replace(Replace {
                regex: Regex::new( r"\d{3}-\d{2}-\d{4}").unwrap(), 
                target: "/name/ssn".to_owned(),
                with: "***-**-****".to_owned()
            })
        ];

        let record = SmartModuleRecord::new(Record::new(INPUT), 0, 0);
        let result = apply_regex_ops_to_json_record(&record, &ops).unwrap();
        let expected_value:Value = serde_json::from_str(EXPECTED).unwrap();
        assert_eq!(result, expected_value);
    }

}