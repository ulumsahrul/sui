// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module bridge::usdc {
    use std::option;
    use sui::math::pow;
    use sui::coin;
    use sui::coin::TreasuryCap;
    use sui::transfer;
    use sui::tx_context::TxContext;

    friend bridge::treasury;

    struct USDC has drop {}

    const DECIMAL: u8 = 6;

    public(friend) fun create(ctx: &mut TxContext): TreasuryCap<USDC> {
        let (treasury_cap, metadata) = coin::create_currency(
            USDC {},
            DECIMAL,
            b"USDC",
            b"USD Coin",
            b"Bridged USD Coin token",
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
