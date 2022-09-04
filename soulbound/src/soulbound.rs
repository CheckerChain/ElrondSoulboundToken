#![no_std]

elrond_wasm::imports!();

use elrond_wasm::types::heap::String;

const HASH_LENGTH: usize = 32;

#[elrond_wasm::contract]
pub trait SoulboundToken{
     #[init]
    fn init(
        &self,
        name: String, 
        symbol: String, 
    ) {
        self.token_name().set(&name);
        self.token_symbol().set(&symbol);
    }

    /// @notice Removes the `token_id: BigUint` from an account. At any time, an
    ///  SBT receiver must be able to disassociate themselves from an SBT
    ///  publicly through calling this function. After successfully executing this
    ///  function, given the parameters for calling `function give` or
    ///  `function take` a token must be re-equipable.
    /// @dev Must emit a `event Transfer` with the `address to` field pointing to
    ///  the zero address.
    /// @param tokenId The identifier for an SBT.
    #[endpoint]
    fn uneqip(
        &self, 
        token_id: BigUint
    ) {
        let token_owner = self.token_owner(&token_id).get();

        require!(
            self.blockchain().get_caller() == token_owner,
            "unequip: sender must be owner"
        );
        require!(BigUint::from(self.next_token_id().get()) > token_id , "token not minted");

        self.used_hash(&token_id).set(false);
        self.burn(token_id);

    }

    /// @notice Creates and transfers the ownership of an SBT from an
    /// `caller` to the transaction's `to: ManagedAddress`.
    /// @param from The origin of the SBT.
    /// @param token_id A distinct token id for a given SBT.
    /// @param signature A secp256k1 signature of structured data hash (active, passive, token_id)
    /// @return A unique `token_id: BigUint` 
    #[endpoint]
    fn give(
        &self, 
        to: ManagedAddress, 
        token_id: BigUint, 
        signature: ManagedByteArray<Self::Api, HASH_LENGTH>
    ) -> BigUint{
        let from = self.blockchain().get_caller();
        require!(self.blockchain().get_caller() != to, "give: cannot give from self");
        require!(BigUint::from(self.next_token_id().get()) > token_id , "token not minted");

        
        let token_id = self.safe_check_agreement(from.clone(), to.clone(), &token_id, signature);
        self.mint(from, to, token_id.clone());
        self.used_hash(&token_id).set(true);
        token_id
    }

    /// @notice Creates and transfers the ownership of an SBT from an
    /// `from: ManagedAddress` to the transaction's `caller`.
    /// @param from The origin of the SBT.
    /// @param token_id A distinct token id for a given SBT.
    /// @param signature A secp256k1 signature of structured data hash (active, passive, token_id)
    /// @return A unique `token_id: BigUint` 
    #[endpoint]
    fn take(
        &self, 
        from: ManagedAddress, 
        token_id: BigUint, 
        signature: ManagedByteArray<Self::Api, HASH_LENGTH>
    ) -> BigUint{
        let to = self.blockchain().get_caller();
        require!(self.blockchain().get_caller() != from, "take: cannot take from self");
        require!(BigUint::from(self.next_token_id().get()) > token_id , "token not minted");

        let token_id = self.safe_check_agreement(to, from, &token_id, signature);
        self.used_hash(&token_id).set(true);
        token_id
    }


    fn safe_check_agreement(
        &self, 
        active: ManagedAddress, 
        passive: ManagedAddress, 
        token_id: &BigUint, 
        signature: ManagedByteArray<Self::Api, HASH_LENGTH>
    ) -> BigUint {
        let hash = self.get_hash(active, passive.clone(), token_id);
        require!(
            self.crypto().verify_secp256k1_legacy(&passive.to_byte_array(), &hash.to_byte_array(), &signature.to_byte_array()),
             "_safeCheckAgreement: invalid signature"
        );

        require!(
            !self.used_hash(&token_id.clone()).get(),
            "_safeCheckAgreement: already used"
        );

        token_id.clone()
    }

    fn get_hash(
        &self, 
        active: ManagedAddress, 
        passive: ManagedAddress, 
        token_id: &BigUint,
    ) -> ManagedByteArray<Self::Api, HASH_LENGTH> {
        let mut buffer_to_hash = ManagedBuffer::new();
        buffer_to_hash.append(&active.as_managed_buffer());
        buffer_to_hash.append(&passive.as_managed_buffer());
        buffer_to_hash.append(&self.get_buffer_from_biguint(&token_id));

        self.crypto().keccak256(buffer_to_hash)
    }

    fn mint(
        &self, 
        from: ManagedAddress,
        to: ManagedAddress,
        token_id: BigUint
    ) {
        self.token_owner(&token_id).set(to.clone());

        self.transfer_event(&from, &to, token_id);
    }

    fn burn(
        &self, 
        token_id: BigUint
    ) {
        let burn_wallet = ManagedAddress::zero();
        let token_owner = self.token_owner(&token_id).get();

        self.token_owner(&token_id).set(burn_wallet.clone());

        self.transfer_event(&token_owner, &burn_wallet, token_id);
    }

    fn get_buffer_from_biguint(
        &self,
        number: &BigUint
    ) -> ManagedBuffer {
        return number.to_bytes_be_buffer();
    }

    /// @notice Provides Token Name
    /// @return The SBTs name
    #[view(getTokenName)]
    #[storage_mapper("tokenName")]
    fn token_name(&self) -> SingleValueMapper<String>;

    /// @notice Provides Token Symbol
    /// @return The SBTs symbol
    #[view(getTokenSymbol)]
    #[storage_mapper("tokenSymbol")]
    fn token_symbol(&self) -> SingleValueMapper<String>;

    /// @notice Provide Next Running Token Id
    /// @return The number of running SBT
    #[view(getNextTokenId)]
    #[storage_mapper("getNextTokenId")]
    fn next_token_id(&self) -> SingleValueMapper<u64>;

    /// @notice Provides Token Owner for the `token_id: BigUint` provided
    /// @param owner An address for whom to query the balance
    /// @return The number of SBTs owned by `owner: ManagedAddress`, possibly zero
    #[view(getTokenOwner)]
    #[storage_mapper("tokenOwner")]
    fn token_owner(&self, token_id: &BigUint) -> SingleValueMapper<ManagedAddress>;

    /// @notice Count all SBTs assigned to an owner
    /// @param owner An address for whom to query the balance
    /// @return The number of SBTs owned by `owner: ManagedAddress`, possibly zero
    #[view(getUserBalance)]
    #[storage_mapper("userBalance")]
    fn balance(&self, owner:&ManagedAddress) -> SingleValueMapper<BigUint>;

    /// @notice Provides used hash status for the `token_id: BigUint` provided
    /// @param owner An address for whom to query the balance
    /// @return The number of SBTs owned by `owner: ManagedAddress`, possibly zero
    #[view(getUsedHash)]
    #[storage_mapper("usedHash")]
    fn used_hash(&self, token_id: &BigUint) -> SingleValueMapper<bool>; 

     /// @dev This emits when ownership of any SBT changes by any mechanism.
    ///  This event emits when SBTs are given or equipped and unequipped
    ///  (`to` == 0).
    #[event("transfer")]
    fn transfer_event(
        &self,
        #[indexed] from: &ManagedAddress,
        #[indexed] to: &ManagedAddress,
        #[indexed] token_id: BigUint,
    );

}
