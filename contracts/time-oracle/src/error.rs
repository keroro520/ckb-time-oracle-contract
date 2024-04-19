use ckb_std::error::SysError;

/// Error
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,

    UnexpectedOutputTimeOracleCells = 100,
    UnexpectOracleId = 101,
    NotFoundAnchoredHeaderDep = 102,
    NotAllowedToUpdateInATimeSpanLessThan60s = 103,
    NotAllowedToIssueMoreThan1000000Tokens = 104,
    InvalidTimeOracleData = 105,
    Unreachable = 106,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}
