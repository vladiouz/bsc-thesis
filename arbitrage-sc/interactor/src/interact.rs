#![allow(non_snake_case)]

use crate::arbitrage_sc_proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use multiversx_sc_snippets::sdk::wallet::Wallet;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const STATE_FILE: &str = "state.toml";
// const OWNER_ADDRESS: &str = "erd1lxm3nexytnp5jyrctd6h0q4wvmv9pscqvg5exnwuht78yrx5j6qsekgy48";
const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const TOKEN_ID: &str = "USDC-350c4e";
const TOKEN_OUT_ID: &str = "WEGLD-a28c59";
const TOKEN_OUT_ID_2: &str = "EBUD-eb3db6";
const LP_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6";
const LP_ADDRESS_2: &str = "erd1qqqqqqqqqqqqqpgqhesqllec0vcxgyr96eu43kuv032sdsk30n4sl42tjc";

pub async fn arbitrage_sc_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "upgrade" => interact.upgrade().await,
        "isPaused" => interact.is_paused().await,
        "getStakedTokenId" => interact.staked_token_id().await,
        "getStakedAmount" => interact.staked_amount().await,
        "getUserWinnings" => interact.user_winnings().await,
        "getDevWinnings" => interact.dev_winnings().await,
        "pause" => interact.pause().await,
        "unpause" => interact.unpause().await,
        "setStakedToken" => interact.set_staked_token().await,
        "withdrawDevWinnings" => interact.withdraw_dev_winnings().await,
        // "executeTrades" => interact.execute_trades().await,
        "stake" => interact.stake().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    contract_address: Option<Bech32Address>,
}

impl State {
    // Deserializes state from file
    pub fn load_state() -> Self {
        if Path::new(STATE_FILE).exists() {
            let mut file = std::fs::File::open(STATE_FILE).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Self::default()
        }
    }

    /// Sets the contract address
    pub fn set_address(&mut self, address: Bech32Address) {
        self.contract_address = Some(address);
    }

    /// Returns the contract address
    pub fn current_address(&self) -> &Bech32Address {
        self.contract_address
            .as_ref()
            .expect("no known contract, deploy first")
    }
}

impl Drop for State {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}

pub struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    state: State,
}

impl ContractInteract {
    pub async fn new() -> Self {
        std::env::set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
        let mut interactor = Interactor::new(GATEWAY).await.use_chain_simulator(false);

        interactor.set_current_dir_from_workspace("arbitrage-sc");
        let wallet_address = interactor
            .register_wallet(Wallet::from_pem_file("wallet1.pem").expect("wallet file not found"))
            .await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_all_activations().await;

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/arbitrage-sc.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state(),
        }
    }

    pub async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(40_000_000u64)
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .init()
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = new_address.to_bech32_default();
        println!("new address: {new_address_bech32}");
        self.state.set_address(new_address_bech32);
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn is_paused(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .is_paused()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn staked_token_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .staked_token_id()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn staked_amount(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .staked_amount()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn owner_winnings_percentage(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .owner_winnings_percentage()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn user_winnings(&mut self) {
        let user = ManagedAddress::<StaticApi>::zero();

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .user_winnings(user)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn dev_winnings(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .dev_winnings()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn pause(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .pause()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn unpause(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(3_000_000u64)
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .unpause()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_staked_token(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(TOKEN_ID);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(3_000_000u64)
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .set_staked_token(token_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn withdraw_dev_winnings(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .withdraw_dev_winnings()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_owner_winnings_percentage(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .set_owner_winnings_percentage(90u8)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn execute_trades(
        &mut self,
        amount: BigUint<StaticApi>,
        swaps: MultiValueEncoded<
            StaticApi,
            MultiValue2<ManagedAddress<StaticApi>, TokenIdentifier<StaticApi>>,
        >,
    ) {
        // let token_out = TokenIdentifier::from_esdt_bytes(TOKEN_OUT_ID);
        // let sc = Bech32Address::from_bech32_string(LP_ADDRESS.to_string());
        // let token_out_2 = TokenIdentifier::from_esdt_bytes(TOKEN_OUT_ID_2);
        // let sc_2 = Bech32Address::from_bech32_string(LP_ADDRESS_2.to_string());

        // let mut swaps = MultiValueEncoded::new();
        // swaps.push(MultiValue2::from((
        //     ManagedAddress::from_address(&sc.address),
        //     token_out.clone(),
        // )));
        // swaps.push(MultiValue2::from((
        //     ManagedAddress::from_address(&sc_2.address),
        //     token_out_2.clone(),
        // )));

        // swaps.push(MultiValue2::from((
        //     ManagedAddress::from_address(&sc_2.address),
        //     token_out.clone(),
        // )));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(75_000_000u64)
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .execute_trades(amount, swaps)
            .returns(ReturnsHandledOrError::new().returns(ReturnsResultUnmanaged))
            .run()
            .await;

        match response {
            Ok(success_response) => println!("Result: {success_response:?}"),
            Err(tx_error) => eprintln!(
                "execute_trades failed: status={}, message={}",
                tx_error.status, tx_error.message
            ),
        }
    }

    pub async fn stake(&mut self) {
        let token_id = String::from(TOKEN_ID);
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(1_000_000u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(5_000_000u64)
            .typed(arbitrage_sc_proxy::ArbitrageScProxy)
            .stake()
            .payment((
                EsdtTokenIdentifier::from(token_id.as_str()),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }
}

#[tokio::test]
async fn test_deploy() {
    let mut interact = ContractInteract::new().await;
    interact.deploy().await;
}

#[tokio::test]
async fn test_unpause() {
    let mut interact = ContractInteract::new().await;
    interact.unpause().await;
}

#[tokio::test]
async fn test_set_staked_token() {
    let mut interact = ContractInteract::new().await;
    interact.set_staked_token().await;
}

#[tokio::test]
async fn test_stake() {
    let mut interact = ContractInteract::new().await;
    interact.stake().await;
}

// #[tokio::test]
// async fn test_execute_trades() {
//     let mut interact = ContractInteract::new().await;
//     // interact.deploy().await;
//     // interact.set_staked_token().await;
//     // interact.unpause().await;
//     // interact.stake().await;
//     interact.execute_trades(BigUint::from(1_000u128)).await;
// }

#[tokio::test]
async fn test_withdraw_dev_winnings() {
    let mut interact = ContractInteract::new().await;
    interact.withdraw_dev_winnings().await;
}
