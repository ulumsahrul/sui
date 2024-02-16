
<a name="0xb_usdt"></a>

# Module `0xb::usdt`



-  [Struct `USDT`](#0xb_usdt_USDT)
-  [Constants](#@Constants_0)
-  [Function `create`](#0xb_usdt_create)
-  [Function `decimal`](#0xb_usdt_decimal)
-  [Function `multiplier`](#0xb_usdt_multiplier)


<pre><code><b>use</b> <a href="dependencies/move-stdlib/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="dependencies/sui-framework/coin.md#0x2_coin">0x2::coin</a>;
<b>use</b> <a href="dependencies/sui-framework/math.md#0x2_math">0x2::math</a>;
<b>use</b> <a href="dependencies/sui-framework/transfer.md#0x2_transfer">0x2::transfer</a>;
<b>use</b> <a href="dependencies/sui-framework/tx_context.md#0x2_tx_context">0x2::tx_context</a>;
<b>use</b> <a href="dependencies/sui-framework/url.md#0x2_url">0x2::url</a>;
</code></pre>



<a name="0xb_usdt_USDT"></a>

## Struct `USDT`



<pre><code><b>struct</b> <a href="usdt.md#0xb_usdt_USDT">USDT</a> <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0xb_usdt_DECIMAL"></a>



<pre><code><b>const</b> <a href="usdt.md#0xb_usdt_DECIMAL">DECIMAL</a>: u8 = 6;
</code></pre>



<a name="0xb_usdt_create"></a>

## Function `create`



<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="usdt.md#0xb_usdt_create">create</a>(ctx: &<b>mut</b> <a href="dependencies/sui-framework/tx_context.md#0x2_tx_context_TxContext">tx_context::TxContext</a>): <a href="dependencies/sui-framework/coin.md#0x2_coin_TreasuryCap">coin::TreasuryCap</a>&lt;<a href="usdt.md#0xb_usdt_USDT">usdt::USDT</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="usdt.md#0xb_usdt_create">create</a>(ctx: &<b>mut</b> TxContext): TreasuryCap&lt;<a href="usdt.md#0xb_usdt_USDT">USDT</a>&gt; {
    <b>let</b> (treasury_cap, metadata) = <a href="dependencies/sui-framework/coin.md#0x2_coin_create_currency">coin::create_currency</a>(
        <a href="usdt.md#0xb_usdt_USDT">USDT</a> {},
        <a href="usdt.md#0xb_usdt_DECIMAL">DECIMAL</a>,
        b"<a href="usdt.md#0xb_usdt_USDT">USDT</a>",
        b"Tether",
        b"Bridged Tether token",
        <a href="dependencies/move-stdlib/option.md#0x1_option_none">option::none</a>(),
        ctx
    );
    <a href="dependencies/sui-framework/transfer.md#0x2_transfer_public_freeze_object">transfer::public_freeze_object</a>(metadata);
    treasury_cap
}
</code></pre>



</details>

<a name="0xb_usdt_decimal"></a>

## Function `decimal`



<pre><code><b>public</b> <b>fun</b> <a href="usdt.md#0xb_usdt_decimal">decimal</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="usdt.md#0xb_usdt_decimal">decimal</a>(): u8 {
    <a href="usdt.md#0xb_usdt_DECIMAL">DECIMAL</a>
}
</code></pre>



</details>

<a name="0xb_usdt_multiplier"></a>

## Function `multiplier`



<pre><code><b>public</b> <b>fun</b> <a href="usdt.md#0xb_usdt_multiplier">multiplier</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="usdt.md#0xb_usdt_multiplier">multiplier</a>(): u64 {
    pow(10, <a href="usdt.md#0xb_usdt_DECIMAL">DECIMAL</a>)
}
</code></pre>



</details>
