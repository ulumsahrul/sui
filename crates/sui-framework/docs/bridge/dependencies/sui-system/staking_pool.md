
<a name="0x3_staking_pool"></a>

# Module `0x3::staking_pool`



-  [Resource `StakingPool`](#0x3_staking_pool_StakingPool)
-  [Struct `PoolTokenExchangeRate`](#0x3_staking_pool_PoolTokenExchangeRate)
-  [Resource `StakedSui`](#0x3_staking_pool_StakedSui)
-  [Constants](#@Constants_0)
-  [Function `new`](#0x3_staking_pool_new)
-  [Function `request_add_stake`](#0x3_staking_pool_request_add_stake)
-  [Function `request_withdraw_stake`](#0x3_staking_pool_request_withdraw_stake)
-  [Function `withdraw_from_principal`](#0x3_staking_pool_withdraw_from_principal)
-  [Function `unwrap_staked_sui`](#0x3_staking_pool_unwrap_staked_sui)
-  [Function `deposit_rewards`](#0x3_staking_pool_deposit_rewards)
-  [Function `process_pending_stakes_and_withdraws`](#0x3_staking_pool_process_pending_stakes_and_withdraws)
-  [Function `process_pending_stake_withdraw`](#0x3_staking_pool_process_pending_stake_withdraw)
-  [Function `process_pending_stake`](#0x3_staking_pool_process_pending_stake)
-  [Function `withdraw_rewards`](#0x3_staking_pool_withdraw_rewards)
-  [Function `activate_staking_pool`](#0x3_staking_pool_activate_staking_pool)
-  [Function `deactivate_staking_pool`](#0x3_staking_pool_deactivate_staking_pool)
-  [Function `sui_balance`](#0x3_staking_pool_sui_balance)
-  [Function `pool_id`](#0x3_staking_pool_pool_id)
-  [Function `staked_sui_amount`](#0x3_staking_pool_staked_sui_amount)
-  [Function `stake_activation_epoch`](#0x3_staking_pool_stake_activation_epoch)
-  [Function `is_preactive`](#0x3_staking_pool_is_preactive)
-  [Function `is_inactive`](#0x3_staking_pool_is_inactive)
-  [Function `split`](#0x3_staking_pool_split)
-  [Function `split_staked_sui`](#0x3_staking_pool_split_staked_sui)
-  [Function `join_staked_sui`](#0x3_staking_pool_join_staked_sui)
-  [Function `is_equal_staking_metadata`](#0x3_staking_pool_is_equal_staking_metadata)
-  [Function `pool_token_exchange_rate_at_epoch`](#0x3_staking_pool_pool_token_exchange_rate_at_epoch)
-  [Function `pending_stake_amount`](#0x3_staking_pool_pending_stake_amount)
-  [Function `pending_stake_withdraw_amount`](#0x3_staking_pool_pending_stake_withdraw_amount)
-  [Function `exchange_rates`](#0x3_staking_pool_exchange_rates)
-  [Function `sui_amount`](#0x3_staking_pool_sui_amount)
-  [Function `pool_token_amount`](#0x3_staking_pool_pool_token_amount)
-  [Function `is_preactive_at_epoch`](#0x3_staking_pool_is_preactive_at_epoch)
-  [Function `get_sui_amount`](#0x3_staking_pool_get_sui_amount)
-  [Function `get_token_amount`](#0x3_staking_pool_get_token_amount)
-  [Function `initial_exchange_rate`](#0x3_staking_pool_initial_exchange_rate)
-  [Function `check_balance_invariants`](#0x3_staking_pool_check_balance_invariants)


<pre><code><b>use</b> <a href="../../dependencies/move-stdlib/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="../../dependencies/sui-framework/bag.md#0x2_bag">0x2::bag</a>;
<b>use</b> <a href="../../dependencies/sui-framework/balance.md#0x2_balance">0x2::balance</a>;
<b>use</b> <a href="../../dependencies/sui-framework/math.md#0x2_math">0x2::math</a>;
<b>use</b> <a href="../../dependencies/sui-framework/object.md#0x2_object">0x2::object</a>;
<b>use</b> <a href="../../dependencies/sui-framework/sui.md#0x2_sui">0x2::sui</a>;
<b>use</b> <a href="../../dependencies/sui-framework/table.md#0x2_table">0x2::table</a>;
<b>use</b> <a href="../../dependencies/sui-framework/transfer.md#0x2_transfer">0x2::transfer</a>;
<b>use</b> <a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context">0x2::tx_context</a>;
</code></pre>



<a name="0x3_staking_pool_StakingPool"></a>

## Resource `StakingPool`



<pre><code><b>struct</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../../dependencies/sui-framework/object.md#0x2_object_UID">object::UID</a></code>
</dt>
<dd>

</dd>
<dt>
<code>activation_epoch: <a href="../../dependencies/move-stdlib/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>deactivation_epoch: <a href="../../dependencies/move-stdlib/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>sui_balance: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>rewards_pool: <a href="../../dependencies/sui-framework/balance.md#0x2_balance_Balance">balance::Balance</a>&lt;<a href="../../dependencies/sui-framework/sui.md#0x2_sui_SUI">sui::SUI</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>pool_token_balance: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>exchange_rates: <a href="../../dependencies/sui-framework/table.md#0x2_table_Table">table::Table</a>&lt;u64, <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">staking_pool::PoolTokenExchangeRate</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>pending_stake: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>pending_total_sui_withdraw: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>pending_pool_token_withdraw: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>extra_fields: <a href="../../dependencies/sui-framework/bag.md#0x2_bag_Bag">bag::Bag</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_staking_pool_PoolTokenExchangeRate"></a>

## Struct `PoolTokenExchangeRate`



<pre><code><b>struct</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>sui_amount: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>pool_token_amount: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x3_staking_pool_StakedSui"></a>

## Resource `StakedSui`



<pre><code><b>struct</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../../dependencies/sui-framework/object.md#0x2_object_UID">object::UID</a></code>
</dt>
<dd>

</dd>
<dt>
<code>pool_id: <a href="../../dependencies/sui-framework/object.md#0x2_object_ID">object::ID</a></code>
</dt>
<dd>

</dd>
<dt>
<code>stake_activation_epoch: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>principal: <a href="../../dependencies/sui-framework/balance.md#0x2_balance_Balance">balance::Balance</a>&lt;<a href="../../dependencies/sui-framework/sui.md#0x2_sui_SUI">sui::SUI</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x3_staking_pool_EActivationOfInactivePool"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EActivationOfInactivePool">EActivationOfInactivePool</a>: u64 = 16;
</code></pre>



<a name="0x3_staking_pool_EDeactivationOfInactivePool"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EDeactivationOfInactivePool">EDeactivationOfInactivePool</a>: u64 = 11;
</code></pre>



<a name="0x3_staking_pool_EDelegationOfZeroSui"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EDelegationOfZeroSui">EDelegationOfZeroSui</a>: u64 = 17;
</code></pre>



<a name="0x3_staking_pool_EDelegationToInactivePool"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EDelegationToInactivePool">EDelegationToInactivePool</a>: u64 = 10;
</code></pre>



<a name="0x3_staking_pool_EDestroyNonzeroBalance"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EDestroyNonzeroBalance">EDestroyNonzeroBalance</a>: u64 = 5;
</code></pre>



<a name="0x3_staking_pool_EIncompatibleStakedSui"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EIncompatibleStakedSui">EIncompatibleStakedSui</a>: u64 = 12;
</code></pre>



<a name="0x3_staking_pool_EInsufficientPoolTokenBalance"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EInsufficientPoolTokenBalance">EInsufficientPoolTokenBalance</a>: u64 = 0;
</code></pre>



<a name="0x3_staking_pool_EInsufficientRewardsPoolBalance"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EInsufficientRewardsPoolBalance">EInsufficientRewardsPoolBalance</a>: u64 = 4;
</code></pre>



<a name="0x3_staking_pool_EInsufficientSuiTokenBalance"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EInsufficientSuiTokenBalance">EInsufficientSuiTokenBalance</a>: u64 = 3;
</code></pre>



<a name="0x3_staking_pool_EPendingDelegationDoesNotExist"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EPendingDelegationDoesNotExist">EPendingDelegationDoesNotExist</a>: u64 = 8;
</code></pre>



<a name="0x3_staking_pool_EPoolAlreadyActive"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EPoolAlreadyActive">EPoolAlreadyActive</a>: u64 = 14;
</code></pre>



<a name="0x3_staking_pool_EPoolNotPreactive"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EPoolNotPreactive">EPoolNotPreactive</a>: u64 = 15;
</code></pre>



<a name="0x3_staking_pool_EStakedSuiBelowThreshold"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EStakedSuiBelowThreshold">EStakedSuiBelowThreshold</a>: u64 = 18;
</code></pre>



<a name="0x3_staking_pool_ETokenBalancesDoNotMatchExchangeRate"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_ETokenBalancesDoNotMatchExchangeRate">ETokenBalancesDoNotMatchExchangeRate</a>: u64 = 9;
</code></pre>



<a name="0x3_staking_pool_ETokenTimeLockIsSome"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_ETokenTimeLockIsSome">ETokenTimeLockIsSome</a>: u64 = 6;
</code></pre>



<a name="0x3_staking_pool_EWithdrawAmountCannotBeZero"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EWithdrawAmountCannotBeZero">EWithdrawAmountCannotBeZero</a>: u64 = 2;
</code></pre>



<a name="0x3_staking_pool_EWithdrawalInSameEpoch"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EWithdrawalInSameEpoch">EWithdrawalInSameEpoch</a>: u64 = 13;
</code></pre>



<a name="0x3_staking_pool_EWrongDelegation"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EWrongDelegation">EWrongDelegation</a>: u64 = 7;
</code></pre>



<a name="0x3_staking_pool_EWrongPool"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EWrongPool">EWrongPool</a>: u64 = 1;
</code></pre>



<a name="0x3_staking_pool_MIN_STAKING_THRESHOLD"></a>



<pre><code><b>const</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_MIN_STAKING_THRESHOLD">MIN_STAKING_THRESHOLD</a>: u64 = 1000000000;
</code></pre>



<a name="0x3_staking_pool_new"></a>

## Function `new`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_new">new</a>(ctx: &<b>mut</b> <a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_new">new</a>(ctx: &<b>mut</b> TxContext) : <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a> {
    <b>let</b> exchange_rates = <a href="../../dependencies/sui-framework/table.md#0x2_table_new">table::new</a>(ctx);
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a> {
        id: <a href="../../dependencies/sui-framework/object.md#0x2_object_new">object::new</a>(ctx),
        activation_epoch: <a href="../../dependencies/move-stdlib/option.md#0x1_option_none">option::none</a>(),
        deactivation_epoch: <a href="../../dependencies/move-stdlib/option.md#0x1_option_none">option::none</a>(),
        sui_balance: 0,
        rewards_pool: <a href="../../dependencies/sui-framework/balance.md#0x2_balance_zero">balance::zero</a>(),
        pool_token_balance: 0,
        exchange_rates,
        pending_stake: 0,
        pending_total_sui_withdraw: 0,
        pending_pool_token_withdraw: 0,
        extra_fields: <a href="../../dependencies/sui-framework/bag.md#0x2_bag_new">bag::new</a>(ctx),
    }
}
</code></pre>



</details>

<a name="0x3_staking_pool_request_add_stake"></a>

## Function `request_add_stake`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_request_add_stake">request_add_stake</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, stake: <a href="../../dependencies/sui-framework/balance.md#0x2_balance_Balance">balance::Balance</a>&lt;<a href="../../dependencies/sui-framework/sui.md#0x2_sui_SUI">sui::SUI</a>&gt;, stake_activation_epoch: u64, ctx: &<b>mut</b> <a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_request_add_stake">request_add_stake</a>(
    pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>,
    stake: Balance&lt;SUI&gt;,
    stake_activation_epoch: u64,
    ctx: &<b>mut</b> TxContext
) : <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a> {
    <b>let</b> sui_amount = <a href="../../dependencies/sui-framework/balance.md#0x2_balance_value">balance::value</a>(&stake);
    <b>assert</b>!(!<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_inactive">is_inactive</a>(pool), <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EDelegationToInactivePool">EDelegationToInactivePool</a>);
    <b>assert</b>!(sui_amount &gt; 0, <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EDelegationOfZeroSui">EDelegationOfZeroSui</a>);
    <b>let</b> staked_sui = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a> {
        id: <a href="../../dependencies/sui-framework/object.md#0x2_object_new">object::new</a>(ctx),
        pool_id: <a href="../../dependencies/sui-framework/object.md#0x2_object_id">object::id</a>(pool),
        stake_activation_epoch,
        principal: stake,
    };
    pool.pending_stake = pool.pending_stake + sui_amount;
    staked_sui
}
</code></pre>



</details>

<a name="0x3_staking_pool_request_withdraw_stake"></a>

## Function `request_withdraw_stake`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_request_withdraw_stake">request_withdraw_stake</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, staked_sui: <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>, ctx: &<a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="../../dependencies/sui-framework/balance.md#0x2_balance_Balance">balance::Balance</a>&lt;<a href="../../dependencies/sui-framework/sui.md#0x2_sui_SUI">sui::SUI</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_request_withdraw_stake">request_withdraw_stake</a>(
    pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>,
    staked_sui: <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>,
    ctx: &TxContext
) : Balance&lt;SUI&gt; {
    <b>let</b> (pool_token_withdraw_amount, principal_withdraw) =
        <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_withdraw_from_principal">withdraw_from_principal</a>(pool, staked_sui);
    <b>let</b> principal_withdraw_amount = <a href="../../dependencies/sui-framework/balance.md#0x2_balance_value">balance::value</a>(&principal_withdraw);

    <b>let</b> rewards_withdraw = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_withdraw_rewards">withdraw_rewards</a>(
        pool, principal_withdraw_amount, pool_token_withdraw_amount, <a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context_epoch">tx_context::epoch</a>(ctx)
    );
    <b>let</b> total_sui_withdraw_amount = principal_withdraw_amount + <a href="../../dependencies/sui-framework/balance.md#0x2_balance_value">balance::value</a>(&rewards_withdraw);

    pool.pending_total_sui_withdraw = pool.pending_total_sui_withdraw + total_sui_withdraw_amount;
    pool.pending_pool_token_withdraw = pool.pending_pool_token_withdraw + pool_token_withdraw_amount;

    // If the pool is inactive, we immediately process the withdrawal.
    <b>if</b> (<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_inactive">is_inactive</a>(pool)) <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_process_pending_stake_withdraw">process_pending_stake_withdraw</a>(pool);

    // TODO: implement withdraw bonding period here.
    <a href="../../dependencies/sui-framework/balance.md#0x2_balance_join">balance::join</a>(&<b>mut</b> principal_withdraw, rewards_withdraw);
    principal_withdraw
}
</code></pre>



</details>

<a name="0x3_staking_pool_withdraw_from_principal"></a>

## Function `withdraw_from_principal`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_withdraw_from_principal">withdraw_from_principal</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, staked_sui: <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>): (u64, <a href="../../dependencies/sui-framework/balance.md#0x2_balance_Balance">balance::Balance</a>&lt;<a href="../../dependencies/sui-framework/sui.md#0x2_sui_SUI">sui::SUI</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_withdraw_from_principal">withdraw_from_principal</a>(
    pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>,
    staked_sui: <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>,
) : (u64, Balance&lt;SUI&gt;) {

    // Check that the stake information matches the pool.
    <b>assert</b>!(staked_sui.pool_id == <a href="../../dependencies/sui-framework/object.md#0x2_object_id">object::id</a>(pool), <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EWrongPool">EWrongPool</a>);

    <b>let</b> exchange_rate_at_staking_epoch = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pool_token_exchange_rate_at_epoch">pool_token_exchange_rate_at_epoch</a>(pool, staked_sui.stake_activation_epoch);
    <b>let</b> principal_withdraw = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_unwrap_staked_sui">unwrap_staked_sui</a>(staked_sui);
    <b>let</b> pool_token_withdraw_amount = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_get_token_amount">get_token_amount</a>(&exchange_rate_at_staking_epoch, <a href="../../dependencies/sui-framework/balance.md#0x2_balance_value">balance::value</a>(&principal_withdraw));

    (
        pool_token_withdraw_amount,
        principal_withdraw,
    )
}
</code></pre>



</details>

<a name="0x3_staking_pool_unwrap_staked_sui"></a>

## Function `unwrap_staked_sui`



<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_unwrap_staked_sui">unwrap_staked_sui</a>(staked_sui: <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>): <a href="../../dependencies/sui-framework/balance.md#0x2_balance_Balance">balance::Balance</a>&lt;<a href="../../dependencies/sui-framework/sui.md#0x2_sui_SUI">sui::SUI</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_unwrap_staked_sui">unwrap_staked_sui</a>(staked_sui: <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>): Balance&lt;SUI&gt; {
    <b>let</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a> {
        id,
        pool_id: _,
        stake_activation_epoch: _,
        principal,
    } = staked_sui;
    <a href="../../dependencies/sui-framework/object.md#0x2_object_delete">object::delete</a>(id);
    principal
}
</code></pre>



</details>

<a name="0x3_staking_pool_deposit_rewards"></a>

## Function `deposit_rewards`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_deposit_rewards">deposit_rewards</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, rewards: <a href="../../dependencies/sui-framework/balance.md#0x2_balance_Balance">balance::Balance</a>&lt;<a href="../../dependencies/sui-framework/sui.md#0x2_sui_SUI">sui::SUI</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_deposit_rewards">deposit_rewards</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>, rewards: Balance&lt;SUI&gt;) {
    pool.sui_balance = pool.sui_balance + <a href="../../dependencies/sui-framework/balance.md#0x2_balance_value">balance::value</a>(&rewards);
    <a href="../../dependencies/sui-framework/balance.md#0x2_balance_join">balance::join</a>(&<b>mut</b> pool.rewards_pool, rewards);
}
</code></pre>



</details>

<a name="0x3_staking_pool_process_pending_stakes_and_withdraws"></a>

## Function `process_pending_stakes_and_withdraws`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_process_pending_stakes_and_withdraws">process_pending_stakes_and_withdraws</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, ctx: &<a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_process_pending_stakes_and_withdraws">process_pending_stakes_and_withdraws</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>, ctx: &TxContext) {
    <b>let</b> new_epoch = <a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context_epoch">tx_context::epoch</a>(ctx) + 1;
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_process_pending_stake_withdraw">process_pending_stake_withdraw</a>(pool);
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_process_pending_stake">process_pending_stake</a>(pool);
    <a href="../../dependencies/sui-framework/table.md#0x2_table_add">table::add</a>(
        &<b>mut</b> pool.exchange_rates,
        new_epoch,
        <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a> { sui_amount: pool.sui_balance, pool_token_amount: pool.pool_token_balance },
    );
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_check_balance_invariants">check_balance_invariants</a>(pool, new_epoch);
}
</code></pre>



</details>

<a name="0x3_staking_pool_process_pending_stake_withdraw"></a>

## Function `process_pending_stake_withdraw`



<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_process_pending_stake_withdraw">process_pending_stake_withdraw</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_process_pending_stake_withdraw">process_pending_stake_withdraw</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>) {
    pool.sui_balance = pool.sui_balance - pool.pending_total_sui_withdraw;
    pool.pool_token_balance = pool.pool_token_balance - pool.pending_pool_token_withdraw;
    pool.pending_total_sui_withdraw = 0;
    pool.pending_pool_token_withdraw = 0;
}
</code></pre>



</details>

<a name="0x3_staking_pool_process_pending_stake"></a>

## Function `process_pending_stake`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_process_pending_stake">process_pending_stake</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_process_pending_stake">process_pending_stake</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>) {
    // Use the most up <b>to</b> date exchange rate <b>with</b> the rewards deposited and withdraws effectuated.
    <b>let</b> latest_exchange_rate =
        <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a> { sui_amount: pool.sui_balance, pool_token_amount: pool.pool_token_balance };
    pool.sui_balance = pool.sui_balance + pool.pending_stake;
    pool.pool_token_balance = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_get_token_amount">get_token_amount</a>(&latest_exchange_rate, pool.sui_balance);
    pool.pending_stake = 0;
}
</code></pre>



</details>

<a name="0x3_staking_pool_withdraw_rewards"></a>

## Function `withdraw_rewards`



<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_withdraw_rewards">withdraw_rewards</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, principal_withdraw_amount: u64, pool_token_withdraw_amount: u64, epoch: u64): <a href="../../dependencies/sui-framework/balance.md#0x2_balance_Balance">balance::Balance</a>&lt;<a href="../../dependencies/sui-framework/sui.md#0x2_sui_SUI">sui::SUI</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_withdraw_rewards">withdraw_rewards</a>(
    pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>,
    principal_withdraw_amount: u64,
    pool_token_withdraw_amount: u64,
    epoch: u64,
) : Balance&lt;SUI&gt; {
    <b>let</b> exchange_rate = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pool_token_exchange_rate_at_epoch">pool_token_exchange_rate_at_epoch</a>(pool, epoch);
    <b>let</b> total_sui_withdraw_amount = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_get_sui_amount">get_sui_amount</a>(&exchange_rate, pool_token_withdraw_amount);
    <b>let</b> reward_withdraw_amount =
        <b>if</b> (total_sui_withdraw_amount &gt;= principal_withdraw_amount)
            total_sui_withdraw_amount - principal_withdraw_amount
        <b>else</b> 0;
    // This may happen when we are withdrawing everything from the pool and
    // the rewards pool <a href="../../dependencies/sui-framework/balance.md#0x2_balance">balance</a> may be less than reward_withdraw_amount.
    // TODO: FIGURE OUT EXACTLY WHY THIS CAN HAPPEN.
    reward_withdraw_amount = <a href="../../dependencies/sui-framework/math.md#0x2_math_min">math::min</a>(reward_withdraw_amount, <a href="../../dependencies/sui-framework/balance.md#0x2_balance_value">balance::value</a>(&pool.rewards_pool));
    <a href="../../dependencies/sui-framework/balance.md#0x2_balance_split">balance::split</a>(&<b>mut</b> pool.rewards_pool, reward_withdraw_amount)
}
</code></pre>



</details>

<a name="0x3_staking_pool_activate_staking_pool"></a>

## Function `activate_staking_pool`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_activate_staking_pool">activate_staking_pool</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, activation_epoch: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_activate_staking_pool">activate_staking_pool</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>, activation_epoch: u64) {
    // Add the initial exchange rate <b>to</b> the <a href="../../dependencies/sui-framework/table.md#0x2_table">table</a>.
    <a href="../../dependencies/sui-framework/table.md#0x2_table_add">table::add</a>(
        &<b>mut</b> pool.exchange_rates,
        activation_epoch,
        <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_initial_exchange_rate">initial_exchange_rate</a>()
    );
    // Check that the pool is preactive and not inactive.
    <b>assert</b>!(<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_preactive">is_preactive</a>(pool), <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EPoolAlreadyActive">EPoolAlreadyActive</a>);
    <b>assert</b>!(!<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_inactive">is_inactive</a>(pool), <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EActivationOfInactivePool">EActivationOfInactivePool</a>);
    // Fill in the active epoch.
    <a href="../../dependencies/move-stdlib/option.md#0x1_option_fill">option::fill</a>(&<b>mut</b> pool.activation_epoch, activation_epoch);
}
</code></pre>



</details>

<a name="0x3_staking_pool_deactivate_staking_pool"></a>

## Function `deactivate_staking_pool`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_deactivate_staking_pool">deactivate_staking_pool</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, deactivation_epoch: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_deactivate_staking_pool">deactivate_staking_pool</a>(pool: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>, deactivation_epoch: u64) {
    // We can't deactivate an already deactivated pool.
    <b>assert</b>!(!<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_inactive">is_inactive</a>(pool), <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EDeactivationOfInactivePool">EDeactivationOfInactivePool</a>);
    pool.deactivation_epoch = <a href="../../dependencies/move-stdlib/option.md#0x1_option_some">option::some</a>(deactivation_epoch);
}
</code></pre>



</details>

<a name="0x3_staking_pool_sui_balance"></a>

## Function `sui_balance`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_sui_balance">sui_balance</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_sui_balance">sui_balance</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>): u64 { pool.sui_balance }
</code></pre>



</details>

<a name="0x3_staking_pool_pool_id"></a>

## Function `pool_id`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pool_id">pool_id</a>(staked_sui: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>): <a href="../../dependencies/sui-framework/object.md#0x2_object_ID">object::ID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pool_id">pool_id</a>(staked_sui: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>): ID { staked_sui.pool_id }
</code></pre>



</details>

<a name="0x3_staking_pool_staked_sui_amount"></a>

## Function `staked_sui_amount`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_staked_sui_amount">staked_sui_amount</a>(staked_sui: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_staked_sui_amount">staked_sui_amount</a>(staked_sui: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>): u64 { <a href="../../dependencies/sui-framework/balance.md#0x2_balance_value">balance::value</a>(&staked_sui.principal) }
</code></pre>



</details>

<a name="0x3_staking_pool_stake_activation_epoch"></a>

## Function `stake_activation_epoch`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_stake_activation_epoch">stake_activation_epoch</a>(staked_sui: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_stake_activation_epoch">stake_activation_epoch</a>(staked_sui: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>): u64 {
    staked_sui.stake_activation_epoch
}
</code></pre>



</details>

<a name="0x3_staking_pool_is_preactive"></a>

## Function `is_preactive`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_preactive">is_preactive</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_preactive">is_preactive</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>): bool{
    <a href="../../dependencies/move-stdlib/option.md#0x1_option_is_none">option::is_none</a>(&pool.activation_epoch)
}
</code></pre>



</details>

<a name="0x3_staking_pool_is_inactive"></a>

## Function `is_inactive`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_inactive">is_inactive</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_inactive">is_inactive</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>): bool {
    <a href="../../dependencies/move-stdlib/option.md#0x1_option_is_some">option::is_some</a>(&pool.deactivation_epoch)
}
</code></pre>



</details>

<a name="0x3_staking_pool_split"></a>

## Function `split`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_split">split</a>(self: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>, split_amount: u64, ctx: &<b>mut</b> <a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_split">split</a>(self: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>, split_amount: u64, ctx: &<b>mut</b> TxContext): <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a> {
    <b>let</b> original_amount = <a href="../../dependencies/sui-framework/balance.md#0x2_balance_value">balance::value</a>(&self.principal);
    <b>assert</b>!(split_amount &lt;= original_amount, <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EInsufficientSuiTokenBalance">EInsufficientSuiTokenBalance</a>);
    <b>let</b> remaining_amount = original_amount - split_amount;
    // Both resulting parts should have at least <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_MIN_STAKING_THRESHOLD">MIN_STAKING_THRESHOLD</a>.
    <b>assert</b>!(remaining_amount &gt;= <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_MIN_STAKING_THRESHOLD">MIN_STAKING_THRESHOLD</a>, <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EStakedSuiBelowThreshold">EStakedSuiBelowThreshold</a>);
    <b>assert</b>!(split_amount &gt;= <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_MIN_STAKING_THRESHOLD">MIN_STAKING_THRESHOLD</a>, <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EStakedSuiBelowThreshold">EStakedSuiBelowThreshold</a>);
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a> {
        id: <a href="../../dependencies/sui-framework/object.md#0x2_object_new">object::new</a>(ctx),
        pool_id: self.pool_id,
        stake_activation_epoch: self.stake_activation_epoch,
        principal: <a href="../../dependencies/sui-framework/balance.md#0x2_balance_split">balance::split</a>(&<b>mut</b> self.principal, split_amount),
    }
}
</code></pre>



</details>

<a name="0x3_staking_pool_split_staked_sui"></a>

## Function `split_staked_sui`



<pre><code><b>public</b> entry <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_split_staked_sui">split_staked_sui</a>(stake: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>, split_amount: u64, ctx: &<b>mut</b> <a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_split_staked_sui">split_staked_sui</a>(stake: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>, split_amount: u64, ctx: &<b>mut</b> TxContext) {
    <a href="../../dependencies/sui-framework/transfer.md#0x2_transfer_transfer">transfer::transfer</a>(<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_split">split</a>(stake, split_amount, ctx), <a href="../../dependencies/sui-framework/tx_context.md#0x2_tx_context_sender">tx_context::sender</a>(ctx));
}
</code></pre>



</details>

<a name="0x3_staking_pool_join_staked_sui"></a>

## Function `join_staked_sui`



<pre><code><b>public</b> entry <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_join_staked_sui">join_staked_sui</a>(self: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>, other: <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_join_staked_sui">join_staked_sui</a>(self: &<b>mut</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>, other: <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>) {
    <b>assert</b>!(<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_equal_staking_metadata">is_equal_staking_metadata</a>(self, &other), <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_EIncompatibleStakedSui">EIncompatibleStakedSui</a>);
    <b>let</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a> {
        id,
        pool_id: _,
        stake_activation_epoch: _,
        principal,
    } = other;

    <a href="../../dependencies/sui-framework/object.md#0x2_object_delete">object::delete</a>(id);
    <a href="../../dependencies/sui-framework/balance.md#0x2_balance_join">balance::join</a>(&<b>mut</b> self.principal, principal);
}
</code></pre>



</details>

<a name="0x3_staking_pool_is_equal_staking_metadata"></a>

## Function `is_equal_staking_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_equal_staking_metadata">is_equal_staking_metadata</a>(self: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>, other: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">staking_pool::StakedSui</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_equal_staking_metadata">is_equal_staking_metadata</a>(self: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>, other: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakedSui">StakedSui</a>): bool {
    (self.pool_id == other.pool_id) &&
    (self.stake_activation_epoch == other.stake_activation_epoch)
}
</code></pre>



</details>

<a name="0x3_staking_pool_pool_token_exchange_rate_at_epoch"></a>

## Function `pool_token_exchange_rate_at_epoch`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pool_token_exchange_rate_at_epoch">pool_token_exchange_rate_at_epoch</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, epoch: u64): <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">staking_pool::PoolTokenExchangeRate</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pool_token_exchange_rate_at_epoch">pool_token_exchange_rate_at_epoch</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>, epoch: u64): <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a> {
    // If the pool is preactive then the exchange rate is always 1:1.
    <b>if</b> (<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_preactive_at_epoch">is_preactive_at_epoch</a>(pool, epoch)) {
        <b>return</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_initial_exchange_rate">initial_exchange_rate</a>()
    };
    <b>let</b> clamped_epoch = <a href="../../dependencies/move-stdlib/option.md#0x1_option_get_with_default">option::get_with_default</a>(&pool.deactivation_epoch, epoch);
    <b>let</b> epoch = <a href="../../dependencies/sui-framework/math.md#0x2_math_min">math::min</a>(clamped_epoch, epoch);
    <b>let</b> activation_epoch = *<a href="../../dependencies/move-stdlib/option.md#0x1_option_borrow">option::borrow</a>(&pool.activation_epoch);

    // Find the latest epoch that's earlier than the given epoch <b>with</b> an entry in the <a href="../../dependencies/sui-framework/table.md#0x2_table">table</a>
    <b>while</b> (epoch &gt;= activation_epoch) {
        <b>if</b> (<a href="../../dependencies/sui-framework/table.md#0x2_table_contains">table::contains</a>(&pool.exchange_rates, epoch)) {
            <b>return</b> *<a href="../../dependencies/sui-framework/table.md#0x2_table_borrow">table::borrow</a>(&pool.exchange_rates, epoch)
        };
        epoch = epoch - 1;
    };
    // This line really should be unreachable. Do we want an <b>assert</b> <b>false</b> here?
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_initial_exchange_rate">initial_exchange_rate</a>()
}
</code></pre>



</details>

<a name="0x3_staking_pool_pending_stake_amount"></a>

## Function `pending_stake_amount`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pending_stake_amount">pending_stake_amount</a>(<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool">staking_pool</a>: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pending_stake_amount">pending_stake_amount</a>(<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool">staking_pool</a>: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>): u64 {
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool">staking_pool</a>.pending_stake
}
</code></pre>



</details>

<a name="0x3_staking_pool_pending_stake_withdraw_amount"></a>

## Function `pending_stake_withdraw_amount`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pending_stake_withdraw_amount">pending_stake_withdraw_amount</a>(<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool">staking_pool</a>: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pending_stake_withdraw_amount">pending_stake_withdraw_amount</a>(<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool">staking_pool</a>: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>): u64 {
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool">staking_pool</a>.pending_total_sui_withdraw
}
</code></pre>



</details>

<a name="0x3_staking_pool_exchange_rates"></a>

## Function `exchange_rates`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_exchange_rates">exchange_rates</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>): &<a href="../../dependencies/sui-framework/table.md#0x2_table_Table">table::Table</a>&lt;u64, <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">staking_pool::PoolTokenExchangeRate</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_exchange_rates">exchange_rates</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>): &Table&lt;u64, <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a>&gt; {
    &pool.exchange_rates
}
</code></pre>



</details>

<a name="0x3_staking_pool_sui_amount"></a>

## Function `sui_amount`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_sui_amount">sui_amount</a>(exchange_rate: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">staking_pool::PoolTokenExchangeRate</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_sui_amount">sui_amount</a>(exchange_rate: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a>): u64 {
    exchange_rate.sui_amount
}
</code></pre>



</details>

<a name="0x3_staking_pool_pool_token_amount"></a>

## Function `pool_token_amount`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pool_token_amount">pool_token_amount</a>(exchange_rate: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">staking_pool::PoolTokenExchangeRate</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pool_token_amount">pool_token_amount</a>(exchange_rate: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a>): u64 {
    exchange_rate.pool_token_amount
}
</code></pre>



</details>

<a name="0x3_staking_pool_is_preactive_at_epoch"></a>

## Function `is_preactive_at_epoch`



<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_preactive_at_epoch">is_preactive_at_epoch</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, epoch: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_preactive_at_epoch">is_preactive_at_epoch</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>, epoch: u64): bool{
    // Either the pool is currently preactive or the pool's starting epoch is later than the provided epoch.
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_is_preactive">is_preactive</a>(pool) || (*<a href="../../dependencies/move-stdlib/option.md#0x1_option_borrow">option::borrow</a>(&pool.activation_epoch) &gt; epoch)
}
</code></pre>



</details>

<a name="0x3_staking_pool_get_sui_amount"></a>

## Function `get_sui_amount`



<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_get_sui_amount">get_sui_amount</a>(exchange_rate: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">staking_pool::PoolTokenExchangeRate</a>, token_amount: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_get_sui_amount">get_sui_amount</a>(exchange_rate: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a>, token_amount: u64): u64 {
    // When either amount is 0, that means we have no stakes <b>with</b> this pool.
    // The other amount might be non-zero when there's dust left in the pool.
    <b>if</b> (exchange_rate.sui_amount == 0 || exchange_rate.pool_token_amount == 0) {
        <b>return</b> token_amount
    };
    <b>let</b> res = (exchange_rate.sui_amount <b>as</b> u128)
            * (token_amount <b>as</b> u128)
            / (exchange_rate.pool_token_amount <b>as</b> u128);
    (res <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x3_staking_pool_get_token_amount"></a>

## Function `get_token_amount`



<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_get_token_amount">get_token_amount</a>(exchange_rate: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">staking_pool::PoolTokenExchangeRate</a>, sui_amount: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_get_token_amount">get_token_amount</a>(exchange_rate: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a>, sui_amount: u64): u64 {
    // When either amount is 0, that means we have no stakes <b>with</b> this pool.
    // The other amount might be non-zero when there's dust left in the pool.
    <b>if</b> (exchange_rate.sui_amount == 0 || exchange_rate.pool_token_amount == 0) {
        <b>return</b> sui_amount
    };
    <b>let</b> res = (exchange_rate.pool_token_amount <b>as</b> u128)
            * (sui_amount <b>as</b> u128)
            / (exchange_rate.sui_amount <b>as</b> u128);
    (res <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x3_staking_pool_initial_exchange_rate"></a>

## Function `initial_exchange_rate`



<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_initial_exchange_rate">initial_exchange_rate</a>(): <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">staking_pool::PoolTokenExchangeRate</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_initial_exchange_rate">initial_exchange_rate</a>(): <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a> {
    <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_PoolTokenExchangeRate">PoolTokenExchangeRate</a> { sui_amount: 0, pool_token_amount: 0 }
}
</code></pre>



</details>

<a name="0x3_staking_pool_check_balance_invariants"></a>

## Function `check_balance_invariants`



<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_check_balance_invariants">check_balance_invariants</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">staking_pool::StakingPool</a>, epoch: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_check_balance_invariants">check_balance_invariants</a>(pool: &<a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_StakingPool">StakingPool</a>, epoch: u64) {
    <b>let</b> exchange_rate = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_pool_token_exchange_rate_at_epoch">pool_token_exchange_rate_at_epoch</a>(pool, epoch);
    // check that the pool token <a href="../../dependencies/sui-framework/balance.md#0x2_balance">balance</a> and <a href="../../dependencies/sui-framework/sui.md#0x2_sui">sui</a> <a href="../../dependencies/sui-framework/balance.md#0x2_balance">balance</a> ratio matches the exchange rate stored.
    <b>let</b> expected = <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_get_token_amount">get_token_amount</a>(&exchange_rate, pool.sui_balance);
    <b>let</b> actual = pool.pool_token_balance;
    <b>assert</b>!(expected == actual, <a href="../../dependencies/sui-system/staking_pool.md#0x3_staking_pool_ETokenBalancesDoNotMatchExchangeRate">ETokenBalancesDoNotMatchExchangeRate</a>)
}
</code></pre>



</details>
