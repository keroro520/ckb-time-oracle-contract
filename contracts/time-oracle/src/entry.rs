use alloc::{vec::Vec};
use core::result::Result;

use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    ckb_types::packed::*,
    high_level::{load_cell_type_hash, load_cell_data, load_cell, load_header, load_input, load_script, load_cell_type, QueryIter},
};
use ckb_hash::Blake2bBuilder;
use crate::error::{Error};
use crate::types::{TimeOracle, SUDTData};

include!(concat!(env!("OUT_DIR"), "/code_hashes.rs"));

macro_rules! CHECK {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}

pub fn find_position_by_type(type_script: &Script, source: Source) -> Option<usize> {
    QueryIter::new(load_cell_type, source).position(|script| match script {
        Some(script) => script.as_bytes() == type_script.as_bytes(),
        _ => false,
    })
}

pub fn sum_udt_by_type(source: Source, TIME_sudt_script_hash: &[u8; 32]) -> u128 {
    let mut sum = 0;
    let mut i = 0;
    loop {
        match load_cell_type_hash(i, source) {
            Ok(Some(type_hash)) => if type_hash.as_slice() == *TIME_sudt_script_hash {
                let output_data = load_cell_data(i, source).unwrap();
                let sudt_data = SUDTData::from_compatible_slice(output_data.as_slice()).unwrap();
                let amount = u128::from_be_bytes(sudt_data.amount().into());
                sum += amount;
            }
            Ok(None) => {
                return sum;
            }
            Err(_) => {
                unreachable!();
            }
        }

        i += 1;
    }
}

/// Unique ID = hash(tx.inputs[0]) | output_index
pub fn unique_id(output_index: usize) -> [u8; 32] {
    let first_input = load_input(0, Source::Input).unwrap();
    let mut blake2b = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    blake2b.update(first_input.as_slice());
    blake2b.update(&(output_index as u64).to_le_bytes());
    let mut unique_id = [0; 32];
    blake2b.finalize(&mut unique_id);
    unique_id
}

/// Returns if there is any header_dep with timestamp == `block_timestamp`.
// TODO: change to QueryIter
pub fn is_anchored_header_dep_exist(block_timestamp: u64) -> bool {
    let mut i = 0;
    loop {
        if let Ok(header_dep) = load_header(i, Source::HeaderDep) {
            if header_dep.timestamp() == block_timestamp {
                return true;
            }
        } else {
            return false;
        }

        i += 1;
    }
}

pub fn main() -> Result<(), Error> {
    let time_oracle_script = load_script()?;
    let time_oracle_script_hash = time_oracle_script.calc_script_hash();
    let time_oracle_cell_inputs: Vec<_> = QueryIter::new(load_cell_type, Source::GroupInput)
        .map(|script| script.unwrap_or_default())
        .collect();
    let time_oracle_cell_outputs: Vec<_> = QueryIter::new(load_cell_type, Source::GroupOutput)
        .map(|script| script.unwrap_or_default())
        .collect();


    let is_initializing_mode = time_oracle_cell_inputs.len() == 0;
    if is_initializing_mode {
        // At the initializing mode,
        //
        // ensure the ORACLE_ID is equal to script.args and
        // ensure the output Time Oracle Cell is unique.
        CHECK!(time_oracle_cell_outputs.len() == 1, Error::UnexpectedOutputTimeOracleCells);
        let time_oracle_cell_output_index =
            find_position_by_type(&time_oracle_cell_outputs[0], Source::Output).ok_or(Error::IndexOutOfBound)?;
        let oracle_id = unique_id(time_oracle_cell_output_index);
        CHECK!(oracle_id == time_oracle_script.args().as_ref()[..32], Error::UnexpectOracleId);

        // Exit optimistically.
        return Ok(());
    } else {
        // When updating,
        // ensure the input Time Oracle Cell is unique and
        // ensure the output Time Oracle Cell is unique.
        CHECK!(time_oracle_cell_inputs.len()  == 1, Error::Unreachable);
        CHECK!(time_oracle_cell_outputs.len() == 1, Error::UnexpectedOutputTimeOracleCells);
    }


    // Now we are updating the Time Oracle.

    let time_oracle_cell_input  = time_oracle_cell_inputs[0];
    let time_oracle_cell_output = time_oracle_cell_outputs[0];
    let prev_index = find_position_by_type(&time_oracle_cell_input, Source::Input).ok_or(Error::IndexOutOfBound)?;
    let post_index = find_position_by_type(&time_oracle_cell_output, Source::Output).ok_or(Error::IndexOutOfBound)?;
    let prev_output_data = load_cell_data(prev_index, Sourece::Input)?;
    let post_output_data = load_cell_data(post_index, Sourece::Output)?;
    let prev = TimeOracle::from_compatible_slice(prev_output_data.as_slice()).map_err(|_| Error::InvalidTimeOracleData)?;
    let post = TimeOracle::from_compatible_slice(post_output_data.as_slice()).map_err(|_| Error::InvalidTimeOracleData)?;
    let prev_timestamp = prev.last_updated_timestmap;
    let post_timestamp = post.last_updated_timestmap;

    // Ensure that the updating timestamp is greater than or equal to `last_updated_timestamp + 60s`.
    CHECK!(
        post_timestamp > prev_timestamp && post_timestamp - prev_timestamp >= 60_000,
        Error::NotAllowedToUpdateInATimeSpanLessThan60s
    );


    // Ensure that the anchored block header exists in the `tx.cell_deps`
    CHECK!(is_anchored_header_dep_exist(post_timestamp), Error::NotFoundAnchoredHeaderDep);


    // Ensure the additional issued "TIME" token less than or equal to 1000
    let TIME_sUDT_Script = Script::new_builder()
        .hash_type(ScriptHashType::Type.into())
        .code_hash(SUDT_CODE_TYPE_HASH)
        .args(time_oracle_script_hash);
    let TIME_sUDT_Script_Hash = TIME_sUDT_Script.calc_script_hash();
    let inputs_token_sum  = sum_udt_by_type(Source::Input, TIME_sUDT_Script_Hash.as_slice());
    let outputs_token_sum = sum_udt_by_type(Source::Output, TIME_sUDT_Script_Hash.as_slice());
    CHECK!(
        outputs_token_sum > inputs_token_sum && outputs_token_sum - inputs_token_sum < 1_000_000,
        Error::NotAllowedToIssueMoreThan1000000Tokens
    );

    return Ok(());
}
