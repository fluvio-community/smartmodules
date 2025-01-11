

use fluvio_smartmodule::{smartmodule, SmartModuleRecord, RecordData, Result, eyre};
use solana_transaction_status_client_types::{
    EncodedTransactionWithStatusMeta, EncodedTransaction, UiMessage, ParsedAccount};
use serde::{Serialize, Deserialize};

#[smartmodule(filter_map)]
pub fn filter_map(record: &SmartModuleRecord) -> Result<Option<(Option<RecordData>, RecordData)>> {
    // Compute balance differences
    let diff = compute_balance_diff(record.value.as_ref())?;

    // Return None if no differences are found
    if diff.is_empty() {
        return Ok(None);
    }

    // Serialize to RecordData
    let json_string = serde_json::to_string(&diff).map_err(|e| eyre!("Failed to serialize JSON: {}", e))?;
    let result = Some((None, RecordData::from(json_string)));

    Ok(result)
}

// Compute Balance Differences
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct BalanceDifference {
    account: String,
    #[serde(rename = "preBalance")]
    pre_balance: u64,
    #[serde(rename = "postBalance")]
    post_balance: u64,
    difference: i64,
}

// Parse record to EndocodedTransactionWithStatusMeta, retrieve balances, and compute differences
fn compute_balance_diff(data: &[u8]) -> Result<Vec<BalanceDifference>> {
    let record: EncodedTransactionWithStatusMeta = serde_json::from_slice(data)
        .map_err(|e| eyre!("Failed to parse EncodedTransactionWithStatusMeta {}", e))?;

    // Parse balances from meta
    let meta = record.meta.as_ref()
        .ok_or_else(|| eyre!("Meta field is missing"))?;
    let pre_balances:&Vec<u64> = meta.pre_balances.as_ref();
    let post_balances:&Vec<u64> = meta.post_balances.as_ref();

    // Parse accounts from transaction
    let accounts = match &record.transaction {
        EncodedTransaction::Json(transaction) => match &transaction.message {
            UiMessage::Parsed(parsed_message) => &parsed_message.account_keys,
            UiMessage::Raw(_) => return Err(eyre!("Transaction message is raw, not parsed")),
        },
        EncodedTransaction::Accounts(accounts) => &accounts.account_keys,
        EncodedTransaction::Binary(_, _) | EncodedTransaction::LegacyBinary(_) => {
            return Err(eyre!("Transaction is not in JSON format"))
        }
    };

    // Compute balance differences
    let balance_diff = find_balance_differences(accounts, pre_balances, post_balances)?;    
    Ok(balance_diff)
}

// Find balance differences between pre_balances and post_balances and return as a Vec<BalanceDifference>
fn find_balance_differences(
    accounts: &[ParsedAccount],
    pre_balances: &[u64],
    post_balances: &[u64],
) -> Result<Vec<BalanceDifference>> {
    if accounts.len() != pre_balances.len() || accounts.len() != post_balances.len() {
        return Err(eyre!(
            "Accounts, pre_balances, and post_balances arrays must have the same length"
        ));
    }

    let mut diffs = Vec::new();
    for (i, account) in accounts.iter().enumerate() {
        let pre_balance = pre_balances[i];
        let post_balance = post_balances[i];

        if pre_balance != post_balance {
            diffs.push(BalanceDifference {
                account: account.pubkey.clone(),
                pre_balance: pre_balance,
                post_balance: post_balance,
                difference: (post_balance as i64) - (pre_balance as i64),
            });
        }
    }

    Ok(diffs)
}

/// Test cases
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn process_record_test() {
        // Expected result
        let expected_result:Vec<BalanceDifference> = serde_json::from_str(r#"
        [
            {
                "account": "2ATdozUDANVdw1um7Lf82bZ4hKPtMGMT4pH42CrLKfn6",
                "preBalance": 7600182,
                "postBalance": 1703981890,
                "difference": 1696381708
            },
            {
                "account": "47kUcJY97j4argbJveNAwFGt3mK8vvSDm4e5vcawFk3B",
                "preBalance": 619005615169,
                "postBalance": 617308697333,
                "difference": -1696917836
            }
        ]"#).unwrap();

        // Read the test data from the file
        let test_file = "test-data/input-record.json";
        let mut input_file = File::open(test_file).expect("Failed to open file");
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer).expect("Failed to read file");

        // Process the record and handle the result
        let result = compute_balance_diff(&buffer);
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_result);
    }
}