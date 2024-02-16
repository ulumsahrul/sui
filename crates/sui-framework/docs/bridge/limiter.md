
<a name="0xb_limiter"></a>

# Module `0xb::limiter`



-  [Struct `TransferLimiter`](#0xb_limiter_TransferLimiter)
-  [Struct `TransferRecord`](#0xb_limiter_TransferRecord)
-  [Constants](#@Constants_0)
-  [Function `new`](#0xb_limiter_new)
-  [Function `current_hour_since_epoch`](#0xb_limiter_current_hour_since_epoch)
-  [Function `check_and_record_transfer`](#0xb_limiter_check_and_record_transfer)
-  [Function `append_new_empty_hours`](#0xb_limiter_append_new_empty_hours)
-  [Function `remove_stale_hour_maybe`](#0xb_limiter_remove_stale_hour_maybe)
-  [Function `transfer_limits`](#0xb_limiter_transfer_limits)


<pre><code><b>use</b> <a href="dependencies/move-stdlib/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="dependencies/move-stdlib/type_name.md#0x1_type_name">0x1::type_name</a>;
<b>use</b> <a href="dependencies/move-stdlib/vector.md#0x1_vector">0x1::vector</a>;
<b>use</b> <a href="dependencies/sui-framework/clock.md#0x2_clock">0x2::clock</a>;
<b>use</b> <a href="dependencies/sui-framework/math.md#0x2_math">0x2::math</a>;
<b>use</b> <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map">0x2::vec_map</a>;
<b>use</b> <a href="chain_ids.md#0xb_chain_ids">0xb::chain_ids</a>;
<b>use</b> <a href="treasury.md#0xb_treasury">0xb::treasury</a>;
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
<code>notional_values: <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_VecMap">vec_map::VecMap</a>&lt;<a href="dependencies/move-stdlib/type_name.md#0x1_type_name_TypeName">type_name::TypeName</a>, u64&gt;</code>
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
    // hardcoded limit for <a href="bridge.md#0xb_bridge">bridge</a> genesis
    <a href="limiter.md#0xb_limiter_TransferLimiter">TransferLimiter</a> {
        transfer_limits: <a href="limiter.md#0xb_limiter_transfer_limits">transfer_limits</a>(),
        notional_values: <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_empty">vec_map::empty</a>(),
        transfer_records: <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_empty">vec_map::empty</a>()
    }
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

<a name="0xb_limiter_check_and_record_transfer"></a>

## Function `check_and_record_transfer`



<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_check_and_record_transfer">check_and_record_transfer</a>&lt;T&gt;(<a href="dependencies/sui-framework/clock.md#0x2_clock">clock</a>: &<a href="dependencies/sui-framework/clock.md#0x2_clock_Clock">clock::Clock</a>, self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferLimiter">limiter::TransferLimiter</a>, route: <a href="chain_ids.md#0xb_chain_ids_BridgeRoute">chain_ids::BridgeRoute</a>, amount: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="limiter.md#0xb_limiter_check_and_record_transfer">check_and_record_transfer</a>&lt;T&gt;(
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

    // First clean up <b>old</b> <a href="dependencies/sui-framework/transfer.md#0x2_transfer">transfer</a> histories
    <a href="limiter.md#0xb_limiter_remove_stale_hour_maybe">remove_stale_hour_maybe</a>(record, current_hour_since_epoch);
    // Backfill missing hours
    <a href="limiter.md#0xb_limiter_append_new_empty_hours">append_new_empty_hours</a>(record, current_hour_since_epoch);

    // Get limit for the route
    <b>let</b> route_limit = <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_try_get">vec_map::try_get</a>(&self.transfer_limits, &route);
    <b>assert</b>!(<a href="dependencies/move-stdlib/option.md#0x1_option_is_some">option::is_some</a>(&route_limit), <a href="limiter.md#0xb_limiter_ERR_LIMIT_NOT_FOUND_FOR_ROUTE">ERR_LIMIT_NOT_FOUND_FOR_ROUTE</a>);
    <b>let</b> route_limit = <a href="dependencies/move-stdlib/option.md#0x1_option_destroy_some">option::destroy_some</a>(route_limit);

    // Compute notional amount
    <b>let</b> coin_type = <a href="dependencies/move-stdlib/type_name.md#0x1_type_name_get">type_name::get</a>&lt;T&gt;();
    // Upcast <b>to</b> u128 <b>to</b> prevent overflow
    <b>let</b> notional_amount = (*<a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_get">vec_map::get</a>(&self.notional_values, &coin_type) <b>as</b> u128) * (amount <b>as</b> u128);
    <b>let</b> notional_amount = notional_amount / (pow(10, <a href="treasury.md#0xb_treasury_token_decimals">treasury::token_decimals</a>&lt;T&gt;()) <b>as</b> u128);
    // Should be safe <b>to</b> downcast <b>to</b> u64 after dividing by the decimals
    <b>let</b> notional_amount = (notional_amount <b>as</b> u64);

    // Check <b>if</b> <a href="dependencies/sui-framework/transfer.md#0x2_transfer">transfer</a> amount exceed limit
    <b>if</b> (record.total_amount + notional_amount &gt; route_limit) {
        <b>return</b> <b>false</b>
    };

    // Record <a href="dependencies/sui-framework/transfer.md#0x2_transfer">transfer</a> value
    <b>let</b> new_amount = <a href="dependencies/move-stdlib/vector.md#0x1_vector_pop_back">vector::pop_back</a>(&<b>mut</b> record.per_hour_amounts) + notional_amount;
    <a href="dependencies/move-stdlib/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> record.per_hour_amounts, new_amount);
    record.total_amount = record.total_amount + notional_amount;
    <b>return</b> <b>true</b>
}
</code></pre>



</details>

<a name="0xb_limiter_append_new_empty_hours"></a>

## Function `append_new_empty_hours`



<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_append_new_empty_hours">append_new_empty_hours</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferRecord">limiter::TransferRecord</a>, current_hour_since_epoch: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_append_new_empty_hours">append_new_empty_hours</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferRecord">TransferRecord</a>, current_hour_since_epoch: u64) {
    <b>if</b> (self.hour_head == current_hour_since_epoch) {
        <b>return</b> // nothing <b>to</b> backfill
    };

    // If tail is even older than 24 hours ago, advance it <b>to</b> that.
    <b>let</b> target_tail = current_hour_since_epoch - 24;
    <b>if</b> (self.hour_tail &lt; target_tail) {
        self.hour_tail = target_tail;
    };

    // If <b>old</b> head is even older than target tail, advance it <b>to</b> that.
    <b>if</b> (self.hour_head &lt; target_tail) {
        self.hour_head = target_tail;
    };

    // Backfill from head <b>to</b> current hour
    <b>while</b> (self.hour_head &lt; current_hour_since_epoch) {
        self.hour_head = self.hour_head + 1;
        <a href="dependencies/move-stdlib/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> self.per_hour_amounts, 0);
    }
}
</code></pre>



</details>

<a name="0xb_limiter_remove_stale_hour_maybe"></a>

## Function `remove_stale_hour_maybe`



<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_remove_stale_hour_maybe">remove_stale_hour_maybe</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferRecord">limiter::TransferRecord</a>, current_hour_since_epoch: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_remove_stale_hour_maybe">remove_stale_hour_maybe</a>(self: &<b>mut</b> <a href="limiter.md#0xb_limiter_TransferRecord">TransferRecord</a>, current_hour_since_epoch: u64) {
    // remove tails until it's within 24 hours range
    <b>while</b> (self.hour_tail + 24 &lt; current_hour_since_epoch && self.hour_tail &lt; self.hour_head) {
        self.total_amount = self.total_amount - <a href="dependencies/move-stdlib/vector.md#0x1_vector_remove">vector::remove</a>(&<b>mut</b> self.per_hour_amounts, 0);
        self.hour_tail = self.hour_tail + 1;
    }
}
</code></pre>



</details>

<a name="0xb_limiter_transfer_limits"></a>

## Function `transfer_limits`



<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_transfer_limits">transfer_limits</a>(): <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_VecMap">vec_map::VecMap</a>&lt;<a href="chain_ids.md#0xb_chain_ids_BridgeRoute">chain_ids::BridgeRoute</a>, u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="limiter.md#0xb_limiter_transfer_limits">transfer_limits</a>(): VecMap&lt;BridgeRoute, u64&gt; {
    <b>let</b> transfer_limits = <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_empty">vec_map::empty</a>();
    // 5M limit on Sui -&gt; Ethereum mainnet
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_eth_mainnet">chain_ids::eth_mainnet</a>(), <a href="chain_ids.md#0xb_chain_ids_sui_mainnet">chain_ids::sui_mainnet</a>()),
        5_000_000 * <a href="limiter.md#0xb_limiter_USD_VALUE_MULTIPLIER">USD_VALUE_MULTIPLIER</a>
    );

    // MAX limit for testnet and devnet
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_eth_sepolia">chain_ids::eth_sepolia</a>(), <a href="chain_ids.md#0xb_chain_ids_sui_testnet">chain_ids::sui_testnet</a>()),
        <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    );
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_eth_sepolia">chain_ids::eth_sepolia</a>(), <a href="chain_ids.md#0xb_chain_ids_sui_devnet">chain_ids::sui_devnet</a>()),
        <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    );
    <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
        &<b>mut</b> transfer_limits,
        <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_eth_sepolia">chain_ids::eth_sepolia</a>(), <a href="chain_ids.md#0xb_chain_ids_eth_sepolia">chain_ids::eth_sepolia</a>()),
        <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    );
    // <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
    //     &<b>mut</b> transfer_limits,
    //     <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_eth_local_test">chain_ids::eth_local_test</a>(), <a href="chain_ids.md#0xb_chain_ids_sui_testnet">chain_ids::sui_testnet</a>()),
    //     <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    // );
    // <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
    //     &<b>mut</b> transfer_limits,
    //     <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_eth_local_test">chain_ids::eth_local_test</a>(), <a href="chain_ids.md#0xb_chain_ids_sui_devnet">chain_ids::sui_devnet</a>()),
    //     <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    // );
    // <a href="dependencies/sui-framework/vec_map.md#0x2_vec_map_insert">vec_map::insert</a>(
    //     &<b>mut</b> transfer_limits,
    //     <a href="chain_ids.md#0xb_chain_ids_get_route">chain_ids::get_route</a>(<a href="chain_ids.md#0xb_chain_ids_eth_local_test">chain_ids::eth_local_test</a>(), <a href="chain_ids.md#0xb_chain_ids_sui_local_test">chain_ids::sui_local_test</a>()),
    //     <a href="limiter.md#0xb_limiter_MAX_TRANSFER_LIMIT">MAX_TRANSFER_LIMIT</a>
    // );
    transfer_limits
}
</code></pre>



</details>
