use std::borrow::Cow;

use once_cell::sync::OnceCell;
use eyre::ContextCompat;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use json_value_merge::Merge;
use dyn_fmt::AsStrFormatExt;
// use chrono::{TimeZone, Utc};

use fluvio_smartmodule::{
    smartmodule, Result, SmartModuleRecord, RecordData,
    dataplane::smartmodule::{
        SmartModuleExtraParams, SmartModuleInitError
    },
    eyre
};

static PARAMS: OnceCell<Params> = OnceCell::new();
const PARAM_NAME: &str = "spec";


#[derive(Debug, Serialize, Deserialize)]
struct Params {
    #[serde(rename="match")]
    _match: Option<Vec<Match>>,
    default: Default,
}

#[derive(Debug, Serialize, Deserialize)]
struct Match {
    key: String,
    value: String,
    format: Format
}

#[derive(Debug, Serialize, Deserialize)]
struct Default {
    format: Format
}

#[derive(Debug, Serialize, Deserialize)]
struct Format {
    with: String,
    using: Vec<String>,
    output: String
}

/// Look-up values in json [ "/top/one", "/top/two"], and generate formatted string
fn make_formatted_string(v: &Value, lookup: &Vec<String>, format_str: &String) -> Result<String> {
    let values = lookup
        .iter()
        .flat_map(|item| v.pointer(item.as_str()))
        .map(|v| {
            if let Some(s) = v.as_str() {
                Cow::Borrowed(s)
            } else {
                Cow::Owned(v.to_string())
            }
        })
        .collect::<Vec<Cow<str>>>();

    Ok(format_str.format(&values))
}

/// Generated formatted string and merge with existing object.
fn process_format(mut v: Value, format: &Format) -> Result<String> {
    let formatted = make_formatted_string(&v, &format.using, &format.with)?;
    v.merge_in(&format.output, &Value::String(formatted))?;
    
    Ok(serde_json::to_string(&v)?)
}

/// Process record based on the options in the input parameter:
///  - format has higher precedence than switch.
fn process_record(data: &str, params: &Params) -> Result<String> {
    let v:Value = serde_json::from_str(data)?;
    
    if let Some(_match) = &params._match {
        for m in _match.iter() {
            if let Some(val) = v.pointer(&m.key) {
                if val == m.value.as_str() {
                    return process_format(v, &m.format);
                }
            }
        }    
    }

    // Use default
    return process_format(v, &params.default.format);
}

#[smartmodule(map)]
pub fn map(record: &SmartModuleRecord) -> Result<(Option<RecordData>, RecordData)> {
    let key = record.key.clone();
    let data = std::str::from_utf8(record.value.as_ref())?;
    let params = PARAMS.get().wrap_err("params not initialized")?;

    /*
    let time_str = Utc.timestamp_millis_opt(record.timestamp() as i64).unwrap();
    println!("TS: {}", time_str);
    */

    let result = process_record(data, params)?;

    Ok((key, result.into()))
}

#[smartmodule(init)]
fn init(params: SmartModuleExtraParams) -> Result<()> {
    if let Some(raw_params) = params.get(PARAM_NAME) {
        match serde_json::from_str(raw_params) {
            Ok(p) => {
                PARAMS.set(p).expect(&format!("{} already initialized", PARAM_NAME));
                Ok(())
            }
            Err(err) => {
                eprintln!("unable to parse params: {err:?}");
                Err(eyre!("cannot parse params: {:#?}", err))
            }
        }
    } else {
        Err(SmartModuleInitError::MissingParam(PARAM_NAME.to_string()).into())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_format_tests() {
        // Test1
        let v:Value = serde_json::from_str(r#"{
            "email": "alice@acme.com",
            "name": "Alice Liddell",
            "type": "subscribe",
            "source": "front-page"
        }"#).unwrap();
        let format:Format = serde_json::from_str(r#"{
            "with": "{} ({}) subscribed on {}",
            "using": [
              "/name",
              "/email",
              "/source"
            ],
            "output": "/formatted"
          }"#).unwrap();
        let expected:Value = serde_json::from_str(r#"{
            "email": "alice@acme.com",
            "name": "Alice Liddell",
            "type": "subscribe",
            "source": "front-page",
            "formatted": "Alice Liddell (alice@acme.com) subscribed on front-page"
        }"#).unwrap();

        let result = process_format(v, &format);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::to_string(&expected).unwrap());

        // Test2
        let v:Value = serde_json::from_str(r#"{
            "email": "charlie@acme.com",
            "name": "Charlie Brimmer",
            "type": "use-case",
            "source": "clickstream",
            "description": "Track user interests"
        }"#).unwrap();
        let format:Format = serde_json::from_str(r#"{
            "with": "{} ({}) wants to solve the following '{}' use-case: {}",
            "using": [
                "/name",
                "/email",
                "/source",
                "/description"
            ],
            "output": "/formatted"
            }"#).unwrap();
        let expected:Value = serde_json::from_str(r#"{
            "email": "charlie@acme.com",
            "name": "Charlie Brimmer",
            "type": "use-case",
            "source": "clickstream",
            "description": "Track user interests",
            "formatted": "Charlie Brimmer (charlie@acme.com) wants to solve the following 'clickstream' use-case: Track user interests"
        }"#).unwrap();

        let result = process_format(v, &format);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::to_string(&expected).unwrap());      
    }

    #[test]
    fn process_record_match_tests() {
        let params:Params = serde_json::from_str(r#"{
            "match": [
              {
                "key": "/type",
                "value": "subscribe",
                "format": {
                  "with": "{} ({}) subscribed on {}",
                  "using": [
                    "/name",
                    "/email",
                    "/source"
                  ],
                  "output": "/formatted"
                }
              },
              {
                "key": "/type",
                "value": "use-case",
                "format": {
                  "with": "{} ({}) wants to solve the following '{}' use-case: {}",
                  "using": [
                    "/name",
                    "/email",
                    "/source",
                    "/description"
                  ],
                  "output": "/formatted"
                }
              }
            ],
            "default": {
                "format": {
                    "with": "{} ({}) submitted a request",
                    "using": [
                        "/name",
                        "/email"
                    ],
                    "output": "/formatted"
                }
            }
          }"#).unwrap();

        // Test1 - type: subscribe
        let data = r#"{
            "email": "alice@acme.com",
            "name": "Alice Liddell",
            "type": "subscribe",
            "source": "front-page"
        }"#;
        let expected:Value = serde_json::from_str(r#"{
            "email": "alice@acme.com",
            "name": "Alice Liddell",
            "type": "subscribe",
            "source": "front-page",
            "formatted": "Alice Liddell (alice@acme.com) subscribed on front-page"
        }"#).unwrap();

        let result = process_record(data, &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::to_string(&expected).unwrap());

        // Test2 - type: use-cases
        let data = r#"{
            "email": "charlie@acme.com",
            "name": "Charlie Brimmer",
            "type": "use-case",
            "source": "clickstream",
            "description": "Track user interests"
        }"#;
        let expected:Value = serde_json::from_str(r#"{
            "email": "charlie@acme.com",
            "name": "Charlie Brimmer",
            "type": "use-case",
            "source": "clickstream",
            "description": "Track user interests",
            "formatted": "Charlie Brimmer (charlie@acme.com) wants to solve the following 'clickstream' use-case: Track user interests"
        }"#).unwrap();

        let result = process_record(data, &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::to_string(&expected).unwrap());

        // Test3 - type: unknown
        let data = r#"{
            "email": "charlie@acme.com",
            "name": "Charlie Brimmer",
            "type": "unknown",
            "source": "new"
        }"#;
        let expected:Value = serde_json::from_str(r#"{
            "email": "charlie@acme.com",
            "name": "Charlie Brimmer",
            "type": "unknown",
            "source": "new",
            "formatted": "Charlie Brimmer (charlie@acme.com) submitted a request"
        }"#).unwrap();

        let result = process_record(data, &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::to_string(&expected).unwrap());        
    }

    #[test]
    fn process_switch_default_tests() {
        let params:Params = serde_json::from_str(r#"{
            "default": {
                "format": {
                    "with": "{} ({}) submitted a request",
                    "using": [
                        "/name",
                        "/email"
                    ],
                    "output": "/formatted"
                }
            }
          }"#).unwrap();
        let data= r#"{
            "email": "alice@acme.com",
            "name": "Alice Liddell",
            "type": "subscribe",
            "source": "front-page"
        }"#;
        let expected:Value = serde_json::from_str(r#"{
            "email": "alice@acme.com",
            "name": "Alice Liddell",
            "type": "subscribe",
            "source": "front-page",
            "formatted": "Alice Liddell (alice@acme.com) submitted a request"
        }"#).unwrap();

        let result = process_record(data, &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::to_string(&expected).unwrap());  
    }
}