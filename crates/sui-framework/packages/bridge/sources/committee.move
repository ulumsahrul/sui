// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_use)]
module bridge::committee {
    use std::vector;

    use sui::ecdsa_k1;
    use sui::event::emit;
    use sui::tx_context::{Self, TxContext};
    use sui::vec_map::{Self, VecMap};
    use sui::vec_set;
    use sui_system::sui_system;
    use sui_system::sui_system::SuiSystemState;

    use bridge::crypto;
    use bridge::message::{Self, Blocklist, BridgeMessage};
    use bridge::message_types;

    #[test_only]
    use sui::hex;
    #[test_only]
    use sui::test_scenario;
    #[test_only]
    use sui::test_utils;
    #[test_only]
    use sui::test_utils::assert_eq;
    #[test_only]
    use bridge::chain_ids;
    #[test_only]
    use sui_system::governance_test_utils::{advance_epoch_with_reward_amounts, create_sui_system_state_for_testing,
        create_validator_for_testing
    };

    friend bridge::bridge;

    const ESignatureBelowThreshold: u64 = 0;
    const EDuplicatedSignature: u64 = 1;
    const EInvalidSignature: u64 = 2;
    const ENotSystemAddress: u64 = 3;
    const EValidatorBlocklistContainsUnknownKey: u64 = 4;
    const ESenderNotActiveValidator: u64 = 5;

    const SUI_MESSAGE_PREFIX: vector<u8> = b"SUI_BRIDGE_MESSAGE";

    struct BlocklistValidatorEvent has copy, drop {
        blocklisted: bool,
        public_keys: vector<vector<u8>>,
    }

    struct BridgeCommittee has store {
        // commitee pub key and weight
        members: VecMap<vector<u8>, CommitteeMember>,
        // min stake threshold for each message type, values are voting power (percentage), 2DP
        stake_thresholds_percentage: VecMap<u8, u64>,
        // Committee member registrations for the next committee creation.
        member_registration: VecMap<address, CommitteeMemberRegistration>,
        // Epoch when the current committee was updated
        last_committee_update_epoch: u64,
    }

    struct CommitteeMember has drop, store {
        /// The Sui Address of the validator
        sui_address: address,
        /// The public key bytes of the bridge key
        bridge_pubkey_bytes: vector<u8>,
        /// Voting power percentage, 2DP
        voting_power: u64,
        /// The HTTP REST URL the member's node listens to
        /// it looks like b'https://127.0.0.1:9191'
        http_rest_url: vector<u8>,
        /// If this member is blocklisted
        blocklisted: bool,
    }

    struct CommitteeMemberRegistration has drop, store {
        /// The Sui Address of the validator
        sui_address: address,
        /// The public key bytes of the bridge key
        bridge_pubkey_bytes: vector<u8>,
        /// The HTTP REST URL the member's node listens to
        /// it looks like b'https://127.0.0.1:9191'
        http_rest_url: vector<u8>,
    }

    public(friend) fun create(ctx: &TxContext): BridgeCommittee {
        assert!(tx_context::sender(ctx) == @0x0, ENotSystemAddress);
        // Default signature threshold
        let thresholds = vec_map::empty();
        vec_map::insert(&mut thresholds, message_types::token(), 5000);
        vec_map::insert(&mut thresholds, message_types::committee_blocklist(), 5000);
        vec_map::insert(&mut thresholds, message_types::emergency_op(), 5000);
        vec_map::insert(&mut thresholds, message_types::update_asset_price(), 5000);
        vec_map::insert(&mut thresholds, message_types::update_bridge_limit(), 5000);
        BridgeCommittee {
            members: vec_map::empty(),
            stake_thresholds_percentage: thresholds,
            member_registration: vec_map::empty(),
            last_committee_update_epoch: 0,
        }
    }

    public fun verify_signatures(
        self: &BridgeCommittee,
        message: BridgeMessage,
        signatures: vector<vector<u8>>,
    ) {
        let (i, signature_counts) = (0, vector::length(&signatures));
        let seen_pub_key = vec_set::empty<vector<u8>>();
        let required_voting_power = *vec_map::get(
            &self.stake_thresholds_percentage,
            &message::message_type(&message)
        ) ;

        // add prefix to the message bytes
        let message_bytes = SUI_MESSAGE_PREFIX;
        vector::append(&mut message_bytes, message::serialize_message(message));

        let threshold = 0;
        while (i < signature_counts) {
            let signature = vector::borrow(&signatures, i);
            let pubkey = ecdsa_k1::secp256k1_ecrecover(signature, &message_bytes, 0);
            // check duplicate
            assert!(!vec_set::contains(&seen_pub_key, &pubkey), EDuplicatedSignature);
            // make sure pub key is part of the committee
            assert!(vec_map::contains(&self.members, &pubkey), EInvalidSignature);
            // get committee signature weight and check pubkey is part of the committee
            let member = vec_map::get(&self.members, &pubkey);
            if (!member.blocklisted) {
                threshold = threshold + member.voting_power;
            };
            i = i + 1;
            vec_set::insert(&mut seen_pub_key, pubkey);
        };
        assert!(threshold >= required_voting_power, ESignatureBelowThreshold);
    }

    public(friend) fun register(
        self: &mut BridgeCommittee,
        system_state: &mut SuiSystemState,
        bridge_pubkey_bytes: vector<u8>,
        http_rest_url: vector<u8>,
        ctx: &TxContext
    ) {
        // sender must be the same sender that created the validator object
        let sender = tx_context::sender(ctx);
        let validators = sui_system::active_validator_addresses(system_state);

        assert!(vector::contains(&validators, &sender), ESenderNotActiveValidator);
        // Sender is active validator, record the registration

        // In case validator need to update the info
        if (vec_map::contains(&self.member_registration, &sender)) {
            let registration = vec_map::get_mut(&mut self.member_registration, &sender);
            registration.http_rest_url = http_rest_url;
            registration.bridge_pubkey_bytes = bridge_pubkey_bytes;
        }else {
            let registration = CommitteeMemberRegistration {
                sui_address: sender,
                bridge_pubkey_bytes,
                http_rest_url,
            };
            vec_map::insert(&mut self.member_registration, sender, registration);
        }
    }

    // This method will try to create the next committee using the registration and system state,
    // if the total stake fails to meet the minimum required percentage, it will skip the update.
    // This is to ensure we don't fail the end of epoch transaction.
    public(friend) fun try_create_next_committee(
        self: &mut BridgeCommittee,
        system_state: &mut SuiSystemState,
        min_stake_participation_percentage: u64,
        ctx: &TxContext
    ) {
        let validators = sui_system::active_validator_addresses(system_state);
        let total_member_stake = 0;
        let i = 0;

        let total_stake_amount = (sui_system::total_stake_amount(system_state) as u128);

        let new_members = vec_map::empty();

        while (i < vec_map::size(&self.member_registration)) {
            // retrieve registration
            let (_, registration) = vec_map::get_entry_by_idx(&self.member_registration, i);
            // Find validator stake amount from system state

            // Process registration if it's active validator
            if (vector::contains(&validators, &registration.sui_address)) {
                let stake_amount = sui_system::validator_stake_amount(system_state, registration.sui_address);
                let voting_power = ((stake_amount as u128) * 10000) / total_stake_amount;
                total_member_stake = total_member_stake + (stake_amount as u128);
                let member = CommitteeMember {
                    sui_address: registration.sui_address,
                    bridge_pubkey_bytes: registration.bridge_pubkey_bytes,
                    voting_power: (voting_power as u64),
                    http_rest_url: registration.http_rest_url,
                    blocklisted: false,
                };
                vec_map::insert(&mut new_members, registration.bridge_pubkey_bytes, member)
            };
            i = i + 1;
        };

        // Make sure the new committee represent enough stakes, percentage are accurate to 2DP
        let stake_participation_percentage = ((total_member_stake * 10000 / (sui_system::total_stake_amount(
            system_state
        ) as u128)) as u64);

        // Store new committee info
        if (stake_participation_percentage >= min_stake_participation_percentage) {
            // Clear registrations
            self.member_registration = vec_map::empty();
            self.members = new_members;
            self.last_committee_update_epoch = tx_context::epoch(ctx);
            // TODO: emit committee update event?
        }
    }

    // This function applys the blocklist to the committee members, we won't need to run this very often so this is not gas optimised.
    // TODO: add tests for this function
    public(friend) fun execute_blocklist(self: &mut BridgeCommittee, blocklist: Blocklist) {
        let blocklisted = message::blocklist_type(&blocklist) != 1;
        let eth_addresses = message::blocklist_validator_addresses(&blocklist);
        let list_len = vector::length(eth_addresses);
        let list_idx = 0;
        let member_idx = 0;
        let pub_keys = vector::empty<vector<u8>>();
        while (list_idx < list_len) {
            let target_address = vector::borrow(eth_addresses, list_idx);
            let found = false;
            while (member_idx < vec_map::size(&self.members)) {
                let (pub_key, member) = vec_map::get_entry_by_idx_mut(&mut self.members, member_idx);
                let eth_address = crypto::ecdsa_pub_key_to_eth_address(*pub_key);
                if (*target_address == eth_address) {
                    member.blocklisted = blocklisted;
                    vector::push_back(&mut pub_keys, *pub_key);
                    found = true;
                    break
                };
                member_idx = member_idx + 1;
            };
            assert!(found, EValidatorBlocklistContainsUnknownKey);
            list_idx = list_idx + 1;
        };
        emit(BlocklistValidatorEvent {
            blocklisted,
            public_keys: pub_keys,
        })
    }

    public(friend) fun committee_members(self: &BridgeCommittee): &VecMap<vector<u8>, CommitteeMember> {
        &self.members
    }

    #[test_only]
    // This is a token transfer message for testing
    const TEST_MSG: vector<u8> =
        b"00010a0000000000000000200000000000000000000000000000000000000000000000000000000000000064012000000000000000000000000000000000000000000000000000000000000000c8033930000000000000";

    #[test_only]
    const VALIDATOR1_PUBKEY: vector<u8> = b"029bef8d556d80e43ae7e0becb3a7e6838b95defe45896ed6075bb9035d06c9964";
    #[test_only]
    const VALIDATOR2_PUBKEY: vector<u8> = b"033e99a541db69bd32040dfe5037fbf5210dafa8151a71e21c5204b05d95ce0a62";


    #[test]
    fun test_verify_signatures_good_path() {
        let committee = setup_test();
        let msg = message::deserialize_message_test_only(hex::decode(TEST_MSG));
        // good path
        verify_signatures(
            &committee,
            msg,
            vector[hex::decode(
                b"8ba030a450cb1e36f61e572645fc9da1dea5f79b6db663a21ab63286d7fc29af447433abdd0c0b35ab751154ac5b612ae64d3be810f0d9e10ff68e764514ced300"
            ), hex::decode(
                b"439379cc7b3ee3ebe1ff59d011dafc1caac47da6919b089c90f6a24e8c284b963b20f1f5421385456e57ac6b69c4b5f0d345aa09b8bc96d88d87051c7349e83801"
            )],
        );

        // Clean up
        test_utils::destroy(committee)
    }

    #[test]
    #[expected_failure(abort_code = EDuplicatedSignature)]
    fun test_verify_signatures_duplicated_sig() {
        let committee = setup_test();
        let msg = message::deserialize_message_test_only(hex::decode(TEST_MSG));
        // good path
        verify_signatures(
            &committee,
            msg,
            vector[hex::decode(
                b"439379cc7b3ee3ebe1ff59d011dafc1caac47da6919b089c90f6a24e8c284b963b20f1f5421385456e57ac6b69c4b5f0d345aa09b8bc96d88d87051c7349e83801"
            ), hex::decode(
                b"439379cc7b3ee3ebe1ff59d011dafc1caac47da6919b089c90f6a24e8c284b963b20f1f5421385456e57ac6b69c4b5f0d345aa09b8bc96d88d87051c7349e83801"
            )],
        );
        abort 0
    }

    #[test]
    #[expected_failure(abort_code = EInvalidSignature)]
    fun test_verify_signatures_invalid_signature() {
        let committee = setup_test();
        let msg = message::deserialize_message_test_only(hex::decode(TEST_MSG));
        // good path
        verify_signatures(
            &committee,
            msg,
            vector[hex::decode(
                b"6ffb3e5ce04dd138611c49520fddfbd6778879c2db4696139f53a487043409536c369c6ffaca165ce3886723cfa8b74f3e043e226e206ea25e313ea2215e6caf01"
            )],
        );
        abort 0
    }

    #[test]
    #[expected_failure(abort_code = ESignatureBelowThreshold)]
    fun test_verify_signatures_below_threshold() {
        let committee = setup_test();
        let msg = message::deserialize_message_test_only(hex::decode(TEST_MSG));
        // good path
        verify_signatures(
            &committee,
            msg,
            vector[hex::decode(
                b"439379cc7b3ee3ebe1ff59d011dafc1caac47da6919b089c90f6a24e8c284b963b20f1f5421385456e57ac6b69c4b5f0d345aa09b8bc96d88d87051c7349e83801"
            )],
        );
        abort 0
    }

    #[test]
    fun test_init_committee() {
        let scenario = test_scenario::begin(@0x0);
        let ctx = test_scenario::ctx(&mut scenario);
        let committee = create(ctx);

        let validators = vector[
            create_validator_for_testing(@validator1, 100, ctx),
            create_validator_for_testing(@validator2, 100, ctx)
        ];
        create_sui_system_state_for_testing(validators, 0, 0, ctx);
        advance_epoch_with_reward_amounts(0, 0, &mut scenario);
        test_scenario::next_tx(&mut scenario, @0x0);

        let system_state = test_scenario::take_shared<SuiSystemState>(&scenario);

        // validator registration
        register(&mut committee, &mut system_state, hex::decode(VALIDATOR1_PUBKEY), b"", &tx(@validator1, 0));
        register(&mut committee, &mut system_state, hex::decode(VALIDATOR2_PUBKEY), b"", &tx(@validator2, 0));

        // Check committee before creation
        assert!(vec_map::is_empty(&committee.members), 0);

        let ctx = test_scenario::ctx(&mut scenario);
        try_create_next_committee(&mut committee, &mut system_state, 60, ctx);

        assert_eq(2, vec_map::size(&committee.members));

        test_utils::destroy(committee);
        test_scenario::return_shared(system_state);
        test_scenario::end(scenario);
    }

    #[test]
    #[expected_failure(abort_code = ESenderNotActiveValidator)]
    fun test_init_committee_not_validator() {
        let scenario = test_scenario::begin(@0x0);
        let ctx = test_scenario::ctx(&mut scenario);
        let committee = create(ctx);

        let validators = vector[
            create_validator_for_testing(@validator1, 100, ctx),
            create_validator_for_testing(@validator2, 100, ctx)
        ];
        create_sui_system_state_for_testing(validators, 0, 0, ctx);
        advance_epoch_with_reward_amounts(0, 0, &mut scenario);
        test_scenario::next_tx(&mut scenario, @0x0);

        let system_state = test_scenario::take_shared<SuiSystemState>(&scenario);

        // validator registration
        register(&mut committee, &mut system_state, hex::decode(VALIDATOR1_PUBKEY), b"", &tx(@validator3, 0));

        test_utils::destroy(committee);
        test_scenario::return_shared(system_state);
        test_scenario::end(scenario);
    }

    #[test]
    fun test_init_committee_not_enough_stake() {
        let scenario = test_scenario::begin(@0x0);
        let ctx = test_scenario::ctx(&mut scenario);
        let committee = create(ctx);

        let validators = vector[
            create_validator_for_testing(@validator1, 100, ctx),
            create_validator_for_testing(@validator2, 100, ctx)
        ];
        create_sui_system_state_for_testing(validators, 0, 0, ctx);
        advance_epoch_with_reward_amounts(0, 0, &mut scenario);
        test_scenario::next_tx(&mut scenario, @0x0);

        let system_state = test_scenario::take_shared<SuiSystemState>(&scenario);

        // validator registration
        register(&mut committee, &mut system_state, hex::decode(VALIDATOR1_PUBKEY), b"", &tx(@validator1, 0));

        // Check committee before creation
        assert!(vec_map::is_empty(&committee.members), 0);

        let ctx = test_scenario::ctx(&mut scenario);
        try_create_next_committee(&mut committee, &mut system_state, 60, ctx);

        // committee should be empty because registration did not reach min stake threshold.
        assert!(vec_map::is_empty(&committee.members), 0);

        test_utils::destroy(committee);
        test_scenario::return_shared(system_state);
        test_scenario::end(scenario);
    }

    #[test_only]
    fun tx(sender: address, hint: u64): TxContext {
        tx_context::new_from_hint(sender, hint, 1, 0, 0)
    }

    #[test]
    #[expected_failure(abort_code = ESignatureBelowThreshold)]
    fun test_verify_signatures_with_blocked_committee_member() {
        let committee = setup_test();
        let msg = message::deserialize_message_test_only(hex::decode(TEST_MSG));
        // good path, this test should have passed in previous test
        verify_signatures(
            &committee,
            msg,
            vector[hex::decode(
                b"8ba030a450cb1e36f61e572645fc9da1dea5f79b6db663a21ab63286d7fc29af447433abdd0c0b35ab751154ac5b612ae64d3be810f0d9e10ff68e764514ced300"
            ), hex::decode(
                b"439379cc7b3ee3ebe1ff59d011dafc1caac47da6919b089c90f6a24e8c284b963b20f1f5421385456e57ac6b69c4b5f0d345aa09b8bc96d88d87051c7349e83801"
            )],
        );

        let (validator1, member) = vec_map::get_entry_by_idx(&committee.members, 0);
        assert!(!member.blocklisted, 0);

        // Block a member
        let blocklist = message::create_blocklist_message(
            chain_ids::sui_testnet(),
            0,
            0, // type 0 is block
            vector[crypto::ecdsa_pub_key_to_eth_address(*validator1)]
        );
        let blocklist = message::extract_blocklist_payload(&blocklist);
        execute_blocklist(&mut committee, blocklist);

        let (_, blocked_member) = vec_map::get_entry_by_idx(&committee.members, 0);
        assert!(blocked_member.blocklisted, 0);

        // Verify signature should fail now
        verify_signatures(
            &committee,
            msg,
            vector[hex::decode(
                b"8ba030a450cb1e36f61e572645fc9da1dea5f79b6db663a21ab63286d7fc29af447433abdd0c0b35ab751154ac5b612ae64d3be810f0d9e10ff68e764514ced300"
            ), hex::decode(
                b"439379cc7b3ee3ebe1ff59d011dafc1caac47da6919b089c90f6a24e8c284b963b20f1f5421385456e57ac6b69c4b5f0d345aa09b8bc96d88d87051c7349e83801"
            )],
        );

        // Clean up
        test_utils::destroy(committee);
    }

    #[test]
    #[expected_failure(abort_code = EValidatorBlocklistContainsUnknownKey)]
    fun test_execute_blocklist_abort_upon_unknown_validator() {
        let committee = setup_test();

        // // val0 and val1 are not blocked yet
        let (validator0, _) = vec_map::get_entry_by_idx(&committee.members, 0);
        // assert!(!member0.blocklisted, 0);
        // let (validator1, member1) = vec_map::get_entry_by_idx(&committee.members, 1);
        // assert!(!member1.blocklisted, 0);

        let eth_address0 = crypto::ecdsa_pub_key_to_eth_address(*validator0);
        let invalid_eth_address1 = x"0000000000000000000000000000000000000000";

        // Blocklist both
        let blocklist = message::create_blocklist_message(
            chain_ids::sui_testnet(),
            0, // seq
            0, // type 0 is blocklist
            vector[eth_address0, invalid_eth_address1]
        );
        let blocklist = message::extract_blocklist_payload(&blocklist);
        execute_blocklist(&mut committee, blocklist);

        // Clean up
        test_utils::destroy(committee);
    }

    #[test]
    fun test_execute_blocklist() {
        let committee = setup_test();

        // val0 and val1 are not blocked yet
        let (validator0, member0) = vec_map::get_entry_by_idx(&committee.members, 0);
        assert!(!member0.blocklisted, 0);
        let (validator1, member1) = vec_map::get_entry_by_idx(&committee.members, 1);
        assert!(!member1.blocklisted, 0);

        let eth_address0 = crypto::ecdsa_pub_key_to_eth_address(*validator0);
        let eth_address1 = crypto::ecdsa_pub_key_to_eth_address(*validator1);

        // Blocklist both
        let blocklist = message::create_blocklist_message(
            chain_ids::sui_testnet(),
            0, // seq
            0, // type 0 is blocklist
            vector[eth_address0, eth_address1]
        );
        let blocklist = message::extract_blocklist_payload(&blocklist);
        execute_blocklist(&mut committee, blocklist);

        // val 0 is blocklisted
        let (_, blocked_member) = vec_map::get_entry_by_idx(&committee.members, 0);
        assert!(blocked_member.blocklisted, 0);
        // val 1 is too
        let (_, blocked_member) = vec_map::get_entry_by_idx(&committee.members, 1);
        assert!(blocked_member.blocklisted, 0);

        // unblocklist val1
        let blocklist = message::create_blocklist_message(
            chain_ids::sui_testnet(),
            1, // seq, this is supposed to increment, but we don't test it here
            1, // type 1 is unblocklist
            vector[eth_address1],
        );
        let blocklist = message::extract_blocklist_payload(&blocklist);
        execute_blocklist(&mut committee, blocklist);

        // val 0 is still blocklisted
        let (_, blocked_member) = vec_map::get_entry_by_idx(&committee.members, 0);
        assert!(blocked_member.blocklisted, 0);
        // val 1 is not
        let (_, blocked_member) = vec_map::get_entry_by_idx(&committee.members, 1);
        assert!(!blocked_member.blocklisted, 0);

        // Clean up
        test_utils::destroy(committee);
    }

    #[test_only]
    fun setup_test(): BridgeCommittee {
        let members = vec_map::empty<vector<u8>, CommitteeMember>();

        let bridge_pubkey_bytes = hex::decode(VALIDATOR1_PUBKEY);
        vec_map::insert(&mut members, bridge_pubkey_bytes, CommitteeMember {
            sui_address: @validator1,
            bridge_pubkey_bytes,
            voting_power: 5000,
            http_rest_url: b"https://127.0.0.1:9191",
            blocklisted: false
        });

        let bridge_pubkey_bytes = hex::decode(VALIDATOR2_PUBKEY);
        vec_map::insert(&mut members, bridge_pubkey_bytes, CommitteeMember {
            sui_address: @validator2,
            bridge_pubkey_bytes,
            voting_power: 5000,
            http_rest_url: b"https://127.0.0.1:9192",
            blocklisted: false
        });

        let thresholds = vec_map::empty();
        vec_map::insert(&mut thresholds, message_types::token(), 6000);

        let committee = BridgeCommittee {
            members,
            stake_thresholds_percentage: thresholds,
            member_registration: vec_map::empty(),
            last_committee_update_epoch: 1,
        };
        committee
    }
}
