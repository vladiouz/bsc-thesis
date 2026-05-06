multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ArbitrageScStorage {
    #[view(isPaused)]
    #[storage_mapper("isPaused")]
    fn is_paused(&self) -> SingleValueMapper<bool>;

    #[view(isStakingPaused)]
    #[storage_mapper("isStakingPaused")]
    fn is_staking_paused(&self) -> SingleValueMapper<bool>;

    #[view(getStakedTokenId)]
    #[storage_mapper("stakedTokenId")]
    fn staked_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getStakedAmount)]
    #[storage_mapper("stakedAmount")]
    fn staked_amount(&self) -> MapMapper<ManagedAddress, BigUint>;

    #[view(getUserWinnings)]
    #[storage_mapper("userWinnings")]
    fn user_winnings(&self, user: ManagedAddress) -> SingleValueMapper<BigUint>;

    #[view(getDevWinnings)]
    #[storage_mapper("devWinnings")]
    fn dev_winnings(&self) -> SingleValueMapper<BigUint>;

    #[view(getOwnerWinningsPercentage)]
    #[storage_mapper("ownerWinningsPercentage")]
    fn owner_winnings_percentage(&self) -> SingleValueMapper<BigUint>;
}
