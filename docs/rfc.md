# CKB Time Oracle

## "TIME" sUDT

CKB Time Oracle 为了激励社区积极地更新 **Time Oracle Cell**，引入了 token 激励机制。"TIME" sUDT 是该 token 的名称，该 token 满足 sUDT 规范。"TIME" 的发行方式是，每次更新 **Time Oracle Cell** 的时间戳，将会增发不少于 1000 枚 "TIME" 给更新者。

```js
"TIME" sUDT Cell:
    capacity:
    output_data:
        amount: u128
    type:
        hash_type: "type"
        code_hash: simple_udt type script hash
        args: <Time Oracle Script hash>
```

## Time Oracle Cell

**Time Oracle Cell** 的 `output_data` 记录着上一次更新的区块信息，**Time Oracle Script** 作为 `type` 保障时间预言机的更新符合要求，**Always Success Script** 作为 `lock` 允许任何人解锁该 cell。

在最初创建 **Time Oracle Cell** 时，计算出 `ORACLE_ID` 并记录于 `type.args`，作为唯一标识。为了保障 `ORACLE_ID` 的唯一性，必须使用 **Time Oracle Script** 作为 `type` 而非 `lock`。

```js
Time Oracle Cell:
    capacity:
    output_data:
        last_updated_timestamp: u64,
    type:
        hash_type: "data1"
        code_hash: Time Oracle Script
        args: <ORACLE_ID>       // ORACLE_ID is `hash(init_transaction.inputs[0]) | Out_Index_Of_Time_Oracle_Cell`
    lock: Always Success Script
```

## Time Oracle Script

```js
// Ensure the ORACLE_ID is valid when initializes the Time Oracle Cell
let Time_Oracle_Script = load_current_script()
let Time_Oracle_Script_hash = Time_Oracle_Script.hash()     // load hash of current script, already covered ORACLE_ID
let input_Time_Oracle_Cell_n  = <count of input  cells with type_hash == Time_Oracle_Script_hash>
let output_Time_Oracle_Cell_n = <count of output cells with type_hash == Time_Oracle_Script_hash>
assert!(output_Time_Oracle_Cell_n == 1, "Not found output Time Oracle Cell")

if input_Time_Oracle_Cell_n == 0 {
    let ORACLE_ID = <hash(transaction.inputs[0]) | Out_Index_Of_Time_Oracle_Cell>
    assert!(ORACLE_ID == Time_Oracle_Script.args, "Unmatched Oracle ID")

    return 0
}
assert!(input_Time_Oracle_Cell_n == 1, "Should not reach here")


// Ensure that the updating timestamp is greater than or equal to `last_updated_timestamp + 60s`.
let Input_Time_Oracle_Cell  = <load the input Time Oracle Cell>
let Output_Time_Oracle_Cell = <load the output Time Oracle Cell>
let diff_timestamp = Output_Time_Oracle_Cell.output_data.last_updated_timestamp - Output_Time_Oracle_Cell.output_data.last_updated_timestamp
assert!(Output_Time_Oracle_Cell.output_data.last_updated_timestamp > Output_Time_Oracle_Cell.output_data.last_updated_timestamp, "Not allowed to update to a lesser timestamp")
assert!(diff_timestamp > 60s, "Not allowed to update in a time span less than 60s")


// Ensure that the anchored block header exists in the `tx.cell_deps`
let anchored_exist = false
let header_deps = load_header_deps()
for header_dep in header_deps {
    if header_dep.timestamp == Output_Time_Oracle_Cell.output_data.last_updated_block_timestamp {
        anchored_exist = true
        break
    }
}
assert!(anchored_exist, "Not found anchored header")


// Ensure the additional issued "TIME" token less than or equal to 1000
let TIME_sUDT_Script = {
    hash_type: "type"
    code_hash: <simple_udt type script hash>
    args: Time_Oracle_Script_hash
}
let inputs_token_sum  = <sum of  input "TIME" cells with TIME_sUDT_script.hash()>
let outputs_token_sum = <sum of output "TIME" cells with TIME_sUDT_script.hash()>
assert!(outputs_token_sum - inputs_token_sum < 1000, "Not allowed to issue more than 1000 tokens")
```

## Update Time Oracle Transaction

```js
Update Time Oracle Transaction:
    Inputs:
        - Time_Oracle_Cell:
            last_updated_timestamp: 123
        - ...
    Outputs:
        - Time_Oracle_Cell
            last_updated_timestamp: 133
        - sUDT cell
        - ...
    HeaderDeps:
        - Anchored Block Header
```
