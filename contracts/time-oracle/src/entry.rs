use alloc::{vec::Vec};
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level::{load_script};
use ckb_std::{
    ckb_types::prelude::*,
    ckb_types::packed::*,
    high_level::{load_script, load_cell_type, QueryIter},
};
use crate::error::{Error};
use crate::types::{TimeOracle};

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

pub fn main() -> Result<(), Error> {
    let time_oracle_script = load_script()?;
    let time_oracle_script_hash = load_script_hash()?;

    let time_oracle_cell_inputs: Vec<_> = QueryIter::new(load_cell_type, Source::GroupInput)
        .map(|script| script.unwrap_or_default())
        .collect();
    let time_oracle_cell_outputs: Vec<_> = QueryIter::new(load_cell_type, Source::GroupOutput)
        .map(|script| script.unwrap_or_default())
        .collect();
    CHECK!(time_oracle_cell_inputs  <= 1, Error::Unreachable);
    CHECK!(time_oracle_cell_outputs == 1, Error::UnexpectedOutputTimeOracleCells);

    // Ensure the ORACLE_ID is equal to script.args when initializing Time Oracle Cell
    if time_oracle_cell_inputs.len() == 0 {
        let time_oracle_cell_output_index =
            find_position_by_type(&time_oracle_cell_outputs[0], Source::Output).ok_or(Error::IndexOutOfBound)?;
        let oracle_id = unique_id(time_oracle_cell_output_index);
        CHECK!(oracle_id == time_oracle_script.args.as_ref()[..32], Error::UnexpectOracleId);

        return Ok(());
    }


    // Ensure that the updating timestamp is greater than or equal to `last_updated_timestamp + 60s`.
    let time_oracle_cell_input  = time_oracle_cell_inputs[0];
    let time_oracle_cell_output = time_oracle_cell_outputs[0];

    let raw_cluster_data = load_cell_data(cell_dep_index, CellDep)?;
    let prev_oracle = TimeOracle::from_compatible_slice(time_oracle_cell_inputs[0].

    let cluster_data = spore::ClusterData::from_compatible_slice(raw_cluster_data)
        .map_err(|_| Error::InvalidClusterData)?;

    
// let Input_Time_Oracle_Cell  = <load the input Time Oracle Cell>
// let Output_Time_Oracle_Cell = <load the output Time Oracle Cell>
// let diff_timestamp = Output_Time_Oracle_Cell.output_data.last_updated_timestamp - Output_Time_Oracle_Cell.output_data.last_updated_timestamp
// assert!(Output_Time_Oracle_Cell.output_data.last_updated_timestamp > Output_Time_Oracle_Cell.output_data.last_updated_timestamp, "Not allowed to update to a lesser timestamp")
// assert!(diff_timestamp > 60s, "Not allowed to update in a time span less than 60s")

    return Ok(());
}
