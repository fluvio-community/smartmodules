use regex::Regex;
use once_cell::sync::OnceCell;
use eyre::ContextCompat;
use serde::Deserialize;

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
    Replace(Replace)
}

#[derive(Debug, Deserialize)]
struct Replace {
    #[serde(with = "serde_regex")]
    regex: Regex,
    with: String,
}

impl Operation {
    pub fn run_regex(&self, text: &String) -> String {
        match self {
            Operation::Replace(r) => {
                r.regex.replace_all(text,  &r.with).to_string()
            }
        }
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

/// Traverse the regex list, compute regex, and collect output
fn apply_regex_ops_to_json_record(record: &SmartModuleRecord, ops: &Vec<Operation>) -> Result<String> {
    let data_str: &str = std::str::from_utf8(record.value.as_ref())?;
    let mut data = data_str.to_string();

    let mut iter = ops.into_iter();
    while let Some(op) = iter.next() {
        data = op.run_regex(&data);
    }

    Ok(data)
}    

#[smartmodule(map)]
pub fn map(record: &SmartModuleRecord) -> Result<(Option<RecordData>, RecordData)> {
    let key = record.key.clone();
    let ops = OPS.get().wrap_err("regex operations not initialized")?;

    let result = apply_regex_ops_to_json_record(record, ops)?;
    Ok((key, result.into()))
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
    use serde_json::Value;
    use fluvio_smartmodule::Record;
    
    static INPUT: &str = r#"{
        "description": "Independence High School",
        "class": "2025-A",
        "students": [
          {
              "first": "Abby",
              "last": "Hardy",
              "address": "285 LA PALA DR APT 2343, SAN JOSE CA 95127",
              "ssn": "123-45-6789"
          },
          {
              "first": "Bob",
              "last": "Newmal",
              "address": "21 E TRIMBLE RD, Santa Clara CA 95347",
              "ssn": "987-65-4321"
          },
          {
              "first": "Cindy",
              "last": "Hall",
              "address": "1601 PRIME PL, Milpitas CA 95344",
              "ssn": "999-88-7777"
          }
        ]
    }"#;

    #[test]
    fn run_regex_test() {
        // Replace exact
        let input = r"123-45-6789".to_owned();
        let regex = r"\d{3}-\d{2}-\d{4}";
        let op = Operation::Replace(Replace {
            regex: Regex::new(regex).unwrap(),
            with: "***-**-****".to_owned()
        });
        let expected = "***-**-****".to_owned();

        let result = op.run_regex(&input);
        assert_eq!(result, expected);

        // Replace subset
        let input = r"Alice Jackson, ssn 123-45-6789, location: NY".to_owned();
        let regex = r"\d{3}-\d{2}-\d{4}";
        let op = Operation::Replace(Replace {
            regex: Regex::new(regex).unwrap(),
            with: "***-**-****".to_owned()
        });
        let expected = "Alice Jackson, ssn ***-**-****, location: NY".to_owned();

        let result = op.run_regex(&input);
        assert_eq!(result, expected);

        // Replace multiple
        let input = r"Alice, ssn 123-45-6789, Jack, ssn 987-65-4321".to_owned();
        let regex = r"\d{3}-\d{2}-\d{4}";
        let op = Operation::Replace(Replace {
            regex: Regex::new(regex).unwrap(),
            with: "***-**-****".to_owned()
        });
        let expected = "Alice, ssn ***-**-****, Jack, ssn ***-**-****".to_owned();

        let result = op.run_regex(&input);
        assert_eq!(result, expected);

        // Replace address
        let input = r#""address": "285 LA PALA DR APT 2343, SAN JOSE CA 95127""#.to_owned();
        let regex = r#"(?P<first>"address":\s+\")([\w\d\s]+),"#;
        let op = Operation::Replace(Replace {
            regex: Regex::new(regex).unwrap(),
            with: "${first}...".to_owned()
        });
        let expected = r#""address": "... SAN JOSE CA 95127""#.to_owned();

        let result = op.run_regex(&input);
        assert_eq!(result, expected);

        // Replace none
        let input = r"not a match".to_owned();
        let regex = r"\d{3}-\d{2}-\d{4}";
        let op = Operation::Replace(Replace {
            regex: Regex::new(regex).unwrap(),
            with: "***-**-****".to_owned()
        });
        let expected = r"not a match".to_owned();

        let result = op.run_regex(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn apply_regex_ops_to_json_record_tests() {
        static EXPECTED: &str = r#"{
            "description": "Independence High School",
            "class": "2025-A",
            "students": [
              {
                  "first": "Abby",
                  "last": "Hardy",
                  "ssn": "***-**-****",
                  "address": "... SAN JOSE CA 95127"
              },
              {
                  "first": "Bob",
                  "last": "Newmal",
                  "ssn": "***-**-****",
                  "address": "... Santa Clara CA 95347"
              },
              {
                  "first": "Cindy",
                  "last": "Hall",
                  "ssn": "***-**-****",
                  "address": "... Milpitas CA 95344"
              }
            ]
        }"#;
        let ops = vec![
            Operation::Replace(Replace {
                regex: Regex::new(r"\d{3}-\d{2}-\d{4}").unwrap(),
                with: "***-**-****".to_owned()
            }),
            Operation::Replace(Replace {
                regex: Regex::new(r#"(?P<first>"address":\s+\")([\w\d\s]+),"#).unwrap(),
                with: "${first}...".to_owned()
            })
        ];

        let record = SmartModuleRecord::new(Record::new(INPUT), 0, 0);
        let result = apply_regex_ops_to_json_record(&record, &ops).unwrap();
        let result_value: Value = serde_json::from_str(result.as_str()).unwrap();

        let expected_value: Value = serde_json::from_str(EXPECTED).unwrap();
        assert_eq!(result_value, expected_value);
    }

}