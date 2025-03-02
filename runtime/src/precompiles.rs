use frame_support::dispatch::{GetDispatchInfo, Pays};
use pallet_evm::{
    IsPrecompileResult, Precompile, PrecompileHandle, PrecompileResult, PrecompileSet,
};
use sp_core::H160;
use sp_std::marker::PhantomData;

use pallet_evm::ExitError;
use pallet_evm_precompile_dispatch::{Dispatch, DispatchValidateT};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};

use pallet_evm_precompile_babe::BabePrecompile;
use pallet_evm_precompile_faucet::FaucetPrecompile;
use pallet_evm_precompile_governance::GovernancePrecompile;
use pallet_evm_precompile_nomination_pools::NominationPoolsPrecompile;
use pallet_evm_precompile_preimage::PreimagePrecompile;
use pallet_evm_precompile_staking::StakingPrecompile;
use pallet_evm_precompile_treasury::TreasuryPrecompile;

use crate::*;

pub struct FrontierPrecompiles<R>(PhantomData<R>);

impl<R> FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
{
    pub fn new() -> Self {
        Self(Default::default())
    }
    pub fn used_addresses() -> [H160; 8] {
        [hash(1), hash(2), hash(3), hash(4), hash(5), hash(1024), hash(1025), hash(1026)]
    }
}
impl<R> PrecompileSet for FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
{
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
        match handle.code_address() {
            // Ethereum precompiles :
            a if a == hash(1) => Some(ECRecover::execute(handle)),
            a if a == hash(2) => Some(Sha256::execute(handle)),
            a if a == hash(3) => Some(Ripemd160::execute(handle)),
            a if a == hash(4) => Some(Identity::execute(handle)),
            a if a == hash(5) => Some(Modexp::execute(handle)),
            // Non-Frontier specific nor Ethereum precompiles :
            a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
            a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
            a if a == hash(1026) => Some(Dispatch::<Runtime, DispatchCallFilter>::execute(handle)),
            // Atleta Precompiles
            a if a == hash(2001) => Some(GovernancePrecompile::<Runtime>::execute(handle)),
            a if a == hash(2002) => Some(TreasuryPrecompile::<Runtime>::execute(handle)),
            a if a == hash(2003) => Some(PreimagePrecompile::<Runtime>::execute(handle)),
            a if a == hash(2004) => Some(StakingPrecompile::<Runtime>::execute(handle)),
            a if a == hash(2005) => Some(FaucetPrecompile::<Runtime>::execute(handle)),
            a if a == hash(2006) => Some(NominationPoolsPrecompile::<Runtime>::execute(handle)),
            a if a == hash(2007) => Some(BabePrecompile::<Runtime>::execute(handle)),
            _ => None,
        }
    }

    fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
        IsPrecompileResult::Answer {
            is_precompile: Self::used_addresses().contains(&address),
            extra_cost: 0,
        }
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}

struct DispatchCallFilter;

impl DispatchValidateT<AccountId, RuntimeCall> for DispatchCallFilter {
    fn validate_before_dispatch(
        _origin: &AccountId,
        call: &RuntimeCall,
    ) -> Option<fp_evm::PrecompileFailure> {
        let info = call.get_dispatch_info();

        if matches!(
            call,
            // we ALLOW dispatching these calls
            RuntimeCall::Staking(..)
                | RuntimeCall::Democracy(..)
                | RuntimeCall::Elections(..)
                | RuntimeCall::Preimage(..)
                | RuntimeCall::NominationPools(..)
                | RuntimeCall::Treasury(..)
        ) {
            None
        } else if info.pays_fee == Pays::No || info.class == DispatchClass::Mandatory {
            // forbid feeless and heavy calls to prevent spaming
            Some(fp_evm::PrecompileFailure::Error {
                exit_status: ExitError::Other("Permission denied calls".into()),
            })
        } else {
            Some(fp_evm::PrecompileFailure::Error {
                exit_status: ExitError::Other(
                    "The call is not allowed to be dispatched via precompile.".into(),
                ),
            })
        }
    }
}
