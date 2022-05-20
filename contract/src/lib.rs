/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
mod icon;

use near_contract_standards::non_fungible_token::events::NftMint;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{refund_deposit_to_account, NonFungibleToken};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

use crate::icon::DATA_IMAGE_WEBP_NEAR_ICON;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

const ARWEAVE_GATEWAY_BASE_URL: &str = "https://arweave.net/";
const NFT_NAME: &str = "Ukrainian Magicals";
const NFT_SYMBOL: &str = "UAMAG";
const NFT_DESCRIPTION: &str = "Ukrainian Magicals - unique NFT collection created by Ukrainian augmented reality team called Magicals within the framework of Hackathon «For Ukraine» by NEAR UA";

// TODO: add sale & royalties

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by the caller with predefined metadata
    #[init]
    pub fn new() -> Self {
        let metadata = NFTContractMetadata {
            spec: NFT_METADATA_SPEC.into(),
            name: NFT_NAME.into(),
            symbol: NFT_SYMBOL.into(),
            icon: Some(DATA_IMAGE_WEBP_NEAR_ICON.into()),
            base_uri: Some(ARWEAVE_GATEWAY_BASE_URL.into()),
            reference: None,
            reference_hash: None,
        };
        let contract = Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                env::predecessor_account_id(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        };
        contract
    }

    /// Mint 3 predefined tokens for contract owner as an initial tokens owner
    #[payable]
    pub fn nft_mint_all(&mut self) {
        let initial_storage = env::storage_usage();
        let issued_at = format!("{}", env::block_timestamp() / 1_000_000_000u64);
        let token_ids = &["0", "1", "2"];
        self.tokens.internal_mint_with_refund(
            token_ids[0].into(),
            self.tokens.owner_id.clone(),
            Some(TokenMetadata {
                title: Some("#0 Mariupol".into()),
                description: Some(NFT_DESCRIPTION.into()),
                media: Some("Cqe2tJCF-yygmxci0RsESa62zQNqPV9oZVDeallYI7o".into()),
                media_hash: None,
                copies: Some(1u64),
                issued_at: Some(issued_at.clone()),
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: Some("Akb7UGDwSbcYka0-frMk5T-YTJQurXzdD0ZBnSqyBRQ".into()),
                reference_hash: None,
            }),
            None,
        );
        self.tokens.internal_mint_with_refund(
            token_ids[1].into(),
            self.tokens.owner_id.clone(),
            Some(TokenMetadata {
                title: Some("#1 Kharkiv".into()),
                description: Some(NFT_DESCRIPTION.into()),
                media: Some("g2kMZ1OhktT0X8R1OzAbdpIk81Dr28uLdyJPlO5YvlM".into()),
                media_hash: None,
                copies: Some(1u64),
                issued_at: Some(issued_at.clone()),
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: Some("65nN_FOLcxCmm5dEPDQi_pQBTu6hxSslvFiepNE02F4".into()),
                reference_hash: None,
            }),
            None,
        );
        self.tokens.internal_mint_with_refund(
            token_ids[2].into(),
            self.tokens.owner_id.clone(),
            Some(TokenMetadata {
                title: Some("#2 Mykolaiv".into()),
                description: Some(NFT_DESCRIPTION.into()),
                media: Some("Cqe2tJCF-yygmxci0RsESa62zQNqPV9oZVDeallYI7o".into()),
                media_hash: None,
                copies: Some(1u64),
                issued_at: Some(issued_at),
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: Some("U8zVK7opopOesv9trJihrwIcZl7tAQcil0sbetfSJ4U".into()),
                reference_hash: None,
            }),
            None,
        );
        refund_deposit_to_account(
            env::storage_usage() - initial_storage,
            env::predecessor_account_id(),
        );
        NftMint {
            owner_id: &self.tokens.owner_id,
            token_ids,
            memo: None,
        }
        .emit();
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use std::collections::HashMap;

    use super::*;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;
    const MINT_ALL_STORAGE_COST: u128 = 21310000000000000000000;

    impl Contract {
        /// Mint a new token with ID=`token_id` belonging to `token_owner_id`.
        pub fn nft_mint(
            &mut self,
            token_id: TokenId,
            token_owner_id: AccountId,
            token_metadata: TokenMetadata,
        ) -> Token {
            assert_eq!(
                env::predecessor_account_id(),
                self.tokens.owner_id,
                "Unauthorized"
            );
            self.tokens
                .internal_mint(token_id, token_owner_id, Some(token_metadata))
        }
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: None,
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new();
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "0".to_string();
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id, accounts(0));
        assert_eq!(token.metadata.unwrap(), sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_mint_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_ALL_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        contract.nft_mint_all();

        // TODO: check nft_token() results
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id, accounts(1));
            assert_eq!(token.metadata.unwrap(), sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }
}
