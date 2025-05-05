use std::sync::OnceLock;

use convert_case::{Case, Casing};
use fluvio_smartmodule::{
    dataplane::smartmodule::SmartModuleExtraParams, eyre, smartmodule, RecordData, Result,
    SmartModuleRecord,
};
use serde::{Deserialize, Serialize};

static SPEC: OnceLock<Spec> = OnceLock::new();
const PARAM_NAME: &str = "spec";
const DEFAULT_CASE: &str = "snake";

#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    #[serde(default = "Spec::default_case")]
    pub casing: String,
    #[serde(default = "Spec::default_depth")]
    pub depth: u8,
}

impl Spec {
    pub fn default_case() -> String {
        DEFAULT_CASE.to_string()
    }

    pub fn default_depth() -> u8 {
        u8::MAX
    }

    pub fn get_case(&self) -> Case<'_> {
        match self.casing.to_lowercase().as_str() {
            DEFAULT_CASE => Case::Snake,
            "camel" => Case::Camel,
            "pascal" => Case::Pascal,
            "kebab" => Case::Kebab,
            "constant" => Case::Constant,
            "cobol" => Case::Cobol,
            _ => Case::Snake,
        }
    }
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            casing: DEFAULT_CASE.to_string(),
            depth: u8::MAX,
        }
    }
}

#[smartmodule(init)]
fn init(params: SmartModuleExtraParams) -> Result<()> {
    if let Some(raw_spec) = params.get(PARAM_NAME) {
        let spec: Spec = serde_json::from_str(raw_spec)?;
        SPEC.set(spec)
            .map_err(|err| eyre!("cannot set spec: {:#?}", err))
    } else {
        SPEC.set(Spec::default())
            .map_err(|err| eyre!("cannot set spec: {:#?}", err))
    }
}

#[smartmodule(map)]
pub fn map(record: &SmartModuleRecord) -> Result<(Option<RecordData>, RecordData)> {
    let def_spec = Spec::default();
    let spec = SPEC.get().unwrap_or(&def_spec);
    let case = spec.get_case();
    let key = record.key.clone();
    let mut src_data: serde_json::Value = serde_json::from_slice(record.value.as_ref())?;
    struct Args<'a> {
        case: Case<'a>,
        depth: u8,
        current_depth: u16,
    }
    fn map_fn(args: Args<'_>, v: &mut serde_json::Value) {
        let case = args.case;
        let depth = args.depth;
        let mut current_depth = args.current_depth;
        if v.is_object() {
            let dst = v.as_object_mut().unwrap();
            let keys: Vec<String> = dst.keys().cloned().collect();
            current_depth = current_depth + 1;
            for k in keys {
                let new_key = k.to_case(case);
                let mut v = dst.remove(&k).unwrap();
                if current_depth <= depth as u16 {
                    map_fn(
                        Args {
                            case,
                            depth,
                            current_depth,
                        },
                        &mut v,
                    );
                }
                dst.insert(new_key, v.clone());
            }
        } else if v.is_array() {
            let items = v.as_array_mut().unwrap();
            for item in items {
                map_fn(
                    Args {
                        case,
                        depth,
                        current_depth,
                    },
                    item,
                );
            }
        }
    }
    let args = Args {
        case,
        depth: spec.depth,
        current_depth: 1,
    };
    map_fn(args, &mut src_data);
    Ok((key, serde_json::to_string(&src_data)?.as_bytes().into()))
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use fluvio_smartmodule::Record;

    use super::*;

    #[test]
    fn map_works() {
        let mut params = BTreeMap::new();
        let spec = Spec {
            casing: "camel".to_string(),
            depth: 2,
        };
        let spec_json = serde_json::to_string(&spec).unwrap();
        params.insert("spec".to_string(), spec_json);
        init(params.into()).expect("cannot init params");

        let record = SmartModuleRecord::new(
            Record::new_key_value(
                "noop".to_string(),
                r#"{"foo_bar":"foo","ipsum":{"foo_bar":{"more_foo":"bar"}}}"#.to_string(),
            ),
            0,
            0,
        );
        let (_, v) = map(&record).unwrap();
        let data = std::str::from_utf8(v.as_ref()).unwrap();
        assert_eq!(
            data,
            r#"{"fooBar":"foo","ipsum":{"fooBar":{"more_foo":"bar"}}}"#
        );
    }
}
