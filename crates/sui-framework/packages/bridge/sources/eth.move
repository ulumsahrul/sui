// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module bridge::eth {
    use std::option;
    use sui::math::pow;
    use sui::coin;
    use sui::coin::TreasuryCap;
    use sui::transfer;
    use sui::tx_context::TxContext;

    friend bridge::treasury;

    struct ETH has drop {}

    const DECIMAL: u8 = 8;

    public(friend) fun create(ctx: &mut TxContext): TreasuryCap<ETH> {
        let (treasury_cap, metadata) = coin::create_currency(
            ETH {},
            DECIMAL,
            b"ETH",
            b"Ethereum",
            b"Bridged Ethereum token",
            option::none(),
            ctx
        );
        transfer::public_freeze_object(metadata);
        treasury_cap
    }

    public fun decimal(): u8 {
        DECIMAL
    }

    public fun multiplier(): u64 {
        pow(10, DECIMAL)
    }
}
