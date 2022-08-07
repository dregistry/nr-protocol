use near_sdk::{Balance, Gas};

pub const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);

pub const NO_DEPOSIT: Balance = 0;

/// Account ID used for $NEAR in near-sdk v3.
/// Need to keep it around for backward compatibility.
pub const OLD_BASE_TOKEN: &str = "some";

/// 1 yN to prevent access key fraud.
#[allow(dead_code)]
pub const ONE_YOCTO_NEAR: Balance = 1;

/// Gas for single ft_transfer call.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(10_000_000_000_000);

// pub const VOTING_COUNT: u64 = 24 * 60 * 60;
pub const ONE_NEAR: Balance = 100_000_000_000_000_000_000_000;
