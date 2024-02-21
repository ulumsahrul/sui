
<a name="0xb_limiter"></a>

# Module `0xb::limiter`



-  [Struct `TransferLimiter`](#0xb_limiter_TransferLimiter)
-  [Struct `TransferRecord`](#0xb_limiter_TransferRecord)
-  [Struct `UpdateRouteLimitEvent`](#0xb_limiter_UpdateRouteLimitEvent)
-  [Struct `UpdateAssetPriceEvent`](#0xb_limiter_UpdateAssetPriceEvent)
-  [Constants](#@Constants_0)
-  [Function `new`](#0xb_limiter_new)
-  [Function `get_route_limit`](#0xb_limiter_get_route_limit)
-  [Function `get_asset_notional_price`](#0xb_limiter_get_asset_notional_price)
-  [Function `update_route_limit`](#0xb_limiter_update_route_limit)
-  [Function `update_asset_notional_price`](#0xb_limiter_update_asset_notional_price)
-  [Function `current_hour_since_epoch`](#0xb_limiter_current_hour_since_epoch)
-  [Function `check_and_record_sending_transfer`](#0xb_limiter_check_and_record_sending_transfer)
-  [Function `adjust_transfer_records`](#0xb_limiter_adjust_transfer_records)
-  [Function `initial_transfer_limits`](#0xb_limiter_initial_transfer_limits)
-  [Function `initial_notional_values`](#0xb_limiter_initial_notional_values)


<pre><code><b>use</b> <a href="dependencies/move-stdlib/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="dependencies/move-stdlib/vector.md#0x1_vector">0x1::vector</a>;
<b>use</b> <a href="dependencies/sui-framework/clock.md#0x2_clock">0x2::clock</a>;
<b>use</b> <a href="dependencies/sui-framework/event.md#0x2_event">0x2::event</a>;
<b>use</b> <a href="dependencies/sui-framework/math.md#0x2_math">0x2::math</a>;
<b>use</b> <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map">0x2::vec_map</a>;
<b>use</b> <a href="btc.md#0xb_btc">0xb::btc</a>;
<b>use</b> <a href="chain_ids.md#0xb_chain_ids">0xb::chain_ids</a>;
<b>use</b> <a href="eth.md#0xb_eth">0xb::eth</a>;
<b>use</b> <a href="treasury.md#0xb_treasury">0xb::treasury</a>;
<b>use</b> <a href="usdc.md#0xb_usdc">0xb::usdc</a>;
<b>use</b> <a href="usdt.md#0xb_usdt">0xb::usdt</a>;
</code></pre>



<a name="0xb_limiter_TransferLimiter"></a>

## Struct `TransferLimiter`



<pre><code><b>struct</b> <a href="limiter.md#0xb_limiter_TransferLimiter">TransferLimiter</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>transfer_limits: <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_VecMap">vec_map::VecMap</a>&lt;<a href="chain_ids.md#0xb_chain_ids_BridgeRoute">chain_ids::BridgeRoute</a>, u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>notional_values: <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_VecMap">vec_map::VecMap</a>&lt;u8, u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>transfer_records: <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_VecMap">vec_map::VecMap</a>&lt;<a href="chain_ids.md#0xb_chain_ids_BridgeRoute">chain_ids::BridgeRoute</a>, <a href="limiter.md#0xb_limiter_TransferRecord">limiter::TransferRecord</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0xb_limiter_TransferRecord"></a>

## Struct `TransferRecord`



<pre><code><b>struct</b> <a href="limiter.md#0xb_limiter_TransferRecord">TransferRecord</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>hour_head: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>hour_tail: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>per_hour_amounts: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>total_amount: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0xb_limiter_UpdateRouteLimitEvent"></a>

## Struct `UpdateRouteLimitEvent`



<pre><code><b>struct</b> <a href="limiter.md#0xb_limiter_UpdateRouteLimitEvent">UpdateRouteLimitEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>sending_chain: u8</code>
</dt>
<dd>

</dd>
<dt>
<code>receiving_chain: u8</code>
</dt>
<dd>

</dd>
<dt>
<code>new_limit: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0xb_limiter_UpdateAssetPriceEvent"></a>

## Struct `UpdateAssetPriceEvent`



<pre><code><b>struct</b> <a href="limiter.md#0xb_limiter_UpdateAssetPriceEvent">UpdateAssetPriceEvent</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>token_id: u8</code>
</dt>
<dd>

</dd>
<dt>
<code>new_price: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0xb_limiter_ERR_LIMIT_NOT_FOUND_FOR_ROUTE"></a>



<pre><code><b>const</b> <a href="limiter.md#0xb_limiter_ERR_LIMIT_NOT_FOUND_FOR_ROUTE">ERR_LIMIT_NOT_FOUND_FOR_ROUTE</a>: u64 = 0;
</code></pre>



<a name="0xb_limiter_MAX_TRANSFER_LIMIT"></a>



<pre><code><b>const</b> <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>: u64 = 18446744073709551615;
</code></pre>



<a name="0xb_limiter_USD_VALUE_MULTIPLIER"></a>



<pre><code><b>const</b> <a href="limiter.md#0xb_limiter_USD_VALUE_MULTIPLIER">USD_VALUE_MULTIPLIER</a>: u64 = 10000;
</code></pre>



<a name="0xb_limiter_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_new">new</a>(): <a href="limiter.md#0xb_limiter_TransferLimiter">limiter::TransferLimiter</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_new">new</a>(): <a href="limiter.md#0xb_limiter_TransferLimiter">TransferLimiter</a> {
    // hardcoded limit for <a href="bridge.md#0xb_bridge">bridge</a> <a href="dependencies/sui-system/genesis.md#0x3_genesis">genesis</a>
    <a href="limiter.md#0xb_limiter_TransferLimiter">TransferLimiter</a> {
        transfer_limits: <a href="limiter.md#0xb_limiter_initial_transfer_limits">initial_transfer_limits</a>(),
        notional_values: <a href="limiter.md#0xb_limiter_initial_notional_values">initial_notional_values</a>(),
        transfer_records: <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_empty">vec_map::empty</a>()
    }
}
</code></pre>



</details>

<a name="0xb_limiter_get_route_limit"></a>

## Function `get_route_limit`



<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_get_route_limit">get_route_limit</a>(self: &<a href="limiter.md#0xb_limiter_TransferLimiter">limiter::TransferLimiter</a>, route: &<a href="chain_ids.md#0xb_chain_ids_BridgeRoute">chain_ids::BridgeRoute</a>): &u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_get_route_limit">get_route_limit</a>(self: &<a href="limiter.md#0xb_limiter_TransferLimiter">TransferLimiter</a>, route: &BridgeRoute): &u64 {
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_get">vec_map::get</a>(&self.transfer_limits, route)
}
</code></pre>



</details>

<a name="0xb_limiter_get_asset_notional_price"></a>

## Function `get_asset_notional_price`



<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_get_asset_notional_price">get_asset_notional_price</a>(self: &<a href="limiter.md#0xb_limiter_TransferLimiter">limiter::TransferLimiter</a>, token_id: &u8): &u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_get_asset_notional_price">get_asset_notional_price</a>(self: &<a href="limiter.md#0xb_limiter_TransferLimiter">TransferLimiter</a>, token_id: &u8): &u64 {
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_get">vec_map::get</a>(&self.notional_values, token_id)
}
</code></pre>



</details>

<a name="0xb_limiter_update_route_limit"></a>

## Function `update_route_limit`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="limiter.md#0xb_limiter_update_route_limit">update_route_limit</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferLimiter">limiter::TransferLimiter</a>, route: &<a href="chain_ids.md#0xb_chain_ids_BridgeRoute">chain_ids::BridgeRoute</a>, new_usd_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="limiter.md#0xb_limiter_update_route_limit">update_route_limit</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferLimiter">TransferLimiter</a>, route: &BridgeRoute, new_usd_limit: u64) {
    <b>if</b> (!<a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_contains">vec_map::contains</a>(&self.transfer_limits, route)) {
        <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(&<b>mut</b> self.transfer_limits, *route, new_usd_limit);
        <b>return</b>
    };
    <b>let</b> entry = <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_get_mut">vec_map::get_mut</a>(&<b>mut</b> self.transfer_limits, route);
    *entry = new_usd_limit;
    emit(<a href="limiter.md#0xb_limiter_UpdateRouteLimitEvent">UpdateRouteLimitEvent</a> {
        sending_chain: *<a href="chain_ids.md#0xb_chain_ids_route_source">chain_ids::route_source</a>(route),
        receiving_chain: *<a href="chain_ids.md#0xb_chain_ids_route_destination">chain_ids::route_destination</a>(route),
        new_limit: new_usd_limit,
    })
}
</code></pre>



</details>

<a name="0xb_limiter_update_asset_notional_price"></a>

## Function `update_asset_notional_price`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="limiter.md#0xb_limiter_update_asset_notional_price">update_asset_notional_price</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferLimiter">limiter::TransferLimiter</a>, token_id: u8, new_usd_price: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="limiter.md#0xb_limiter_update_asset_notional_price">update_asset_notional_price</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferLimiter">TransferLimiter</a>, token_id: u8, new_usd_price: u64) {
    <b>if</b> (!<a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_contains">vec_map::contains</a>(&self.notional_values, &token_id)) {
        <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(&<b>mut</b> self.notional_values, token_id, new_usd_price);
        <b>return</b>
    };
    <b>let</b> entry = <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_get_mut">vec_map::get_mut</a>(&<b>mut</b> self.notional_values, &token_id);
    *entry = new_usd_price;
    emit(<a href="limiter.md#0xb_limiter_UpdateAssetPriceEvent">UpdateAssetPriceEvent</a> {
        token_id: token_id,
        new_price: new_usd_price,
    })
}
</code></pre>



</details>

<a name="0xb_limiter_current_hour_since_epoch"></a>

## Function `current_hour_since_epoch`



<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_current_hour_since_epoch">current_hour_since_epoch</a>(<a href="dependencies/sui-framework/clock.md#0x2_clock">clock</a>: &<a href="dependencies/sui-framework/clock.md#0x2_clock_Clock">clock::Clock</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_current_hour_since_epoch">current_hour_since_epoch</a>(<a href="dependencies/sui-framework/clock.md#0x2_clock">clock</a>: &Clock): u64 {
    <a href="dependencies/sui-framework/clock.md#0x2_clock_timestamp_ms">clock::timestamp_ms</a>(<a href="dependencies/sui-framework/clock.md#0x2_clock">clock</a>) / 3600000
}
</code></pre>



</details>

<a name="0xb_limiter_check_and_record_sending_transfer"></a>

## Function `check_and_record_sending_transfer`



<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_check_and_record_sending_transfer">check_and_record_sending_transfer</a>&lt;T&gt;(<a href="dependencies/sui-framework/clock.md#0x2_clock">clock</a>: &<a href="dependencies/sui-framework/clock.md#0x2_clock_Clock">clock::Clock</a>, self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferLimiter">limiter::TransferLimiter</a>, route: <a href="chain_ids.md#0xb_chain_ids_BridgeRoute">chain_ids::BridgeRoute</a>, amount: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_check_and_record_sending_transfer">check_and_record_sending_transfer</a>&lt;T&gt;(
    <a href="dependencies/sui-framework/clock.md#0x2_clock">clock</a>: & Clock,
    self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferLimiter">TransferLimiter</a>,
    route: BridgeRoute,
    amount: u64
): bool {
    // Create record for route <b>if</b> not exists
    <b>if</b> (!<a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_contains">vec_map::contains</a>(&self.transfer_records, &route)) {
        <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(&<b>mut</b> self.transfer_records, route, <a href="limiter.md#0xb_limiter_TransferRecord">TransferRecord</a> {
            hour_head: 0,
            hour_tail: 0,
            per_hour_amounts: <a href="dependencies/move-stdlib/vector.md#0x1_vector">vector</a>[],
            total_amount: 0
        })
    };
    <b>let</b> record = <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_get_mut">vec_map::get_mut</a>(&<b>mut</b> self.transfer_records, &route);
    <b>let</b> current_hour_since_epoch = <a href="limiter.md#0xb_limiter_current_hour_since_epoch">current_hour_since_epoch</a>(<a href="dependencies/sui-framework/clock.md#0x2_clock">clock</a>);

    <a href="limiter.md#0xb_limiter_adjust_transfer_records">adjust_transfer_records</a>(record, current_hour_since_epoch);

    // Get limit for the route
    <b>let</b> route_limit = <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_try_get">vec_map::try_get</a>(&self.transfer_limits, &route);
    <b>assert</b>!(<a href="dependencies/move-stdlib/option.md#0x1_option_is_some">option::is_some</a>(&route_limit), <a href="limiter.md#0xb_limiter_ERR_LIMIT_NOT_FOUND_FOR_ROUTE">ERR_LIMIT_NOT_FOUND_FOR_ROUTE</a>);
    <b>let</b> route_limit = <a href="dependencies/move-stdlib/option.md#0x1_option_destroy_some">option::destroy_some</a>(route_limit);
    <b>let</b> route_limit_adjsuted = (route_limit <b>as</b> u128) * (pow(10, <a href="treasury.md#0xb_treasury_token_decimals">treasury::token_decimals</a>&lt;T&gt;()) <b>as</b> u128);

    // Compute notional amount
    // Upcast <b>to</b> u128 <b>to</b> prevent overflow, <b>to</b> not miss out on small amounts.
    <b>let</b> notional_amount_with_token_multiplier = (*<a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_get">vec_map::get</a>(&self.notional_values, &<a href="treasury.md#0xb_treasury_token_id">treasury::token_id</a>&lt;T&gt;()) <b>as</b> u128) * (amount <b>as</b> u128);

    // Check <b>if</b> <a href="dependencies/sui-framework/transfer.md#0x2_transfer">transfer</a> amount exceed limit
    // Upscale them <b>to</b> the token's decimal.
    <b>if</b> ((record.total_amount * pow(10, <a href="treasury.md#0xb_treasury_token_decimals">treasury::token_decimals</a>&lt;T&gt;()) <b>as</b> u128) + notional_amount_with_token_multiplier &gt; route_limit_adjsuted) {
        <b>return</b> <b>false</b>
    };

    // TODO: add `treasury::token_multiplier`
    // Now scale down <b>to</b> notional value
    <b>let</b> notional_amount = notional_amount_with_token_multiplier / (pow(10, <a href="treasury.md#0xb_treasury_token_decimals">treasury::token_decimals</a>&lt;T&gt;()) <b>as</b> u128);
    // Should be safe <b>to</b> downcast <b>to</b> u64 after dividing by the decimals
    <b>let</b> notional_amount = (notional_amount <b>as</b> u64);

    // Record <a href="dependencies/sui-framework/transfer.md#0x2_transfer">transfer</a> value
    <b>let</b> new_amount = <a href="dependencies/move-stdlib/vector.md#0x1_vector_pop_back">vector::pop_back</a>(&<b>mut</b> record.per_hour_amounts) + notional_amount;
    <a href="dependencies/move-stdlib/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> record.per_hour_amounts, new_amount);
    record.total_amount = record.total_amount + notional_amount;
    <b>return</b> <b>true</b>
}
</code></pre>



</details>

<a name="0xb_limiter_adjust_transfer_records"></a>

## Function `adjust_transfer_records`



<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_adjust_transfer_records">adjust_transfer_records</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferRecord">limiter::TransferRecord</a>, current_hour_since_epoch: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_adjust_transfer_records">adjust_transfer_records</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferRecord">TransferRecord</a>, current_hour_since_epoch: u64) {
    <b>if</b> (self.hour_head == current_hour_since_epoch) {
        <b>return</b> // nothing <b>to</b> backfill
    };

    <b>let</b> target_tail = current_hour_since_epoch - 23;

    // If `hour_head` is even older than 24 hours ago, it means all items in
    // `per_hour_amounts` are <b>to</b> be evicted.
    <b>if</b> (self.hour_head &lt; target_tail) {
        self.per_hour_amounts = <a href="dependencies/move-stdlib/vector.md#0x1_vector_empty">vector::empty</a>();
        self.total_amount = 0;
        self.hour_tail = target_tail;
        self.hour_head = target_tail;
        // Don't forget <b>to</b> insert this hour's record
        <a href="dependencies/move-stdlib/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> self.per_hour_amounts, 0);
    } <b>else</b> {
        // self.hour_head is within 24 hour range.
        // some items in `per_hour_amounts` are still valid, we remove stale hours.
        <b>while</b> (self.hour_tail &lt; target_tail) {
            self.total_amount = self.total_amount - <a href="dependencies/move-stdlib/vector.md#0x1_vector_remove">vector::remove</a>(&<b>mut</b> self.per_hour_amounts, 0);
            self.hour_tail = self.hour_tail + 1;
        }
    };

    // Backfill from hour_head <b>to</b> current hour
    <b>while</b> (self.hour_head &lt; current_hour_since_epoch) {
        self.hour_head = self.hour_head + 1;
        <a href="dependencies/move-stdlib/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> self.per_hour_amounts, 0);
    }
}
</code></pre>



</details>

<a name="0xb_limiter_initial_transfer_limits"></a>

## Function `initial_transfer_limits`



<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_initial_transfer_limits">initial_transfer_limits</a>(): <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_VecMap">vec_map::VecMap</a>&lt;<a href="chain_ids.md#0xb_chain_ids_BridgeRoute">chain_ids::BridgeRoute</a>, u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_initial_transfer_limits">initial_transfer_limits</a>(): VecMap&lt;BridgeRoute, u64&gt; {
    <b>let</b> transfer_limits = <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_empty">vec_map::empty</a>();
    // 5M limit on Sui -&gt; Ethereum mainnet
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_sui_mainnet">chain_ids::sui_mainnet</a>(), <a href="chain_ids.md#0xb_chain_ids_eth_mainnet">chain_ids::eth_mainnet</a>()),
        5_000_000 * <a href="limiter.md#0xb_limiter_USD_VALUE_MULTIPLIER">USD_VALUE_MULTIPLIER</a>
    );

    // MAX limit for testnet and devnet
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_sui_testnet">chain_ids::sui_testnet</a>(), <a href="chain_ids.md#0xb_chain_ids_eth_sepolia">chain_ids::eth_sepolia</a>()),
        <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    );
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_sui_devnet">chain_ids::sui_devnet</a>(), <a href="chain_ids.md#0xb_chain_ids_eth_sepolia">chain_ids::eth_sepolia</a>()),
        <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    );
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_sui_local_test">chain_ids::sui_local_test</a>(), <a href="chain_ids.md#0xb_chain_ids_eth_sepolia">chain_ids::eth_sepolia</a>()),
        <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    );
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_sui_testnet">chain_ids::sui_testnet</a>(), <a href="chain_ids.md#0xb_chain_ids_eth_local_test">chain_ids::eth_local_test</a>()),
        <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    );
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_sui_devnet">chain_ids::sui_devnet</a>(), <a href="chain_ids.md#0xb_chain_ids_eth_local_test">chain_ids::eth_local_test</a>()),
        <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    );
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_sui_local_test">chain_ids::sui_local_test</a>(), <a href="chain_ids.md#0xb_chain_ids_eth_local_test">chain_ids::eth_local_test</a>()),
        <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    );
    transfer_limits
}
</code></pre>



</details>

<a name="0xb_limiter_initial_notional_values"></a>

## Function `initial_notional_values`



<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_initial_notional_values">initial_notional_values</a>(): <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_VecMap">vec_map::VecMap</a>&lt;u8, u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_initial_notional_values">initial_notional_values</a>(): VecMap&lt;u8, u64&gt; {
    <b>let</b> notional_values = <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_empty">vec_map::empty</a>();
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(&<b>mut</b> notional_values, <a href="treasury.md#0xb_treasury_token_id">treasury::token_id</a>&lt;BTC&gt;(), 50_000 * <a href="limiter.md#0xb_limiter_USD_VALUE_MULTIPLIER">USD_VALUE_MULTIPLIER</a>);
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(&<b>mut</b> notional_values, <a href="treasury.md#0xb_treasury_token_id">treasury::token_id</a>&lt;ETH&gt;(), 3_000 * <a href="limiter.md#0xb_limiter_USD_VALUE_MULTIPLIER">USD_VALUE_MULTIPLIER</a>);
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(&<b>mut</b> notional_values, <a href="treasury.md#0xb_treasury_token_id">treasury::token_id</a>&lt;USDC&gt;(), 1 * <a href="limiter.md#0xb_limiter_USD_VALUE_MULTIPLIER">USD_VALUE_MULTIPLIER</a>);
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(&<b>mut</b> notional_values, <a href="treasury.md#0xb_treasury_token_id">treasury::token_id</a>&lt;USDT&gt;(), 1 * <a href="limiter.md#0xb_limiter_USD_VALUE_MULTIPLIER">USD_VALUE_MULTIPLIER</a>);
    notional_values
}
</code></pre>



</details>
