# CKB JSON-RPC Protocols


*   [`Chain`](#chain)
    *   [`get_block`](#get_block)
    *   [`get_block_by_number`](#get_block_by_number)
    *   [`get_block_hash`](#get_block_hash)
    *   [`get_cells_by_lock_hash`](#get_cells_by_lock_hash)
    *   [`get_current_epoch`](#get_current_epoch)
    *   [`get_epoch_by_number`](#get_epoch_by_number)
    *   [`get_live_cell`](#get_live_cell)
    *   [`get_tip_block_number`](#get_tip_block_number)
    *   [`get_tip_header`](#get_tip_header)
    *   [`get_transaction`](#get_transaction)
*   [`Experiment`](#experiment)
    *   [`_compute_code_hash`](#_compute_code_hash)
    *   [`_compute_script_hash`](#_compute_script_hash)
    *   [`_compute_transaction_hash`](#_compute_transaction_hash)
    *   [`dry_run_transaction`](#dry_run_transaction)
*   [`Net`](#net)
    *   [`get_peers`](#get_peers)
    *   [`local_node_info`](#local_node_info)
*   [`Pool`](#pool)
    *   [`send_transaction`](#send_transaction)
    *   [`tx_pool_info`](#tx_pool_info)
*   [`Stats`](#stats)
    *   [`get_blockchain_info`](#get_blockchain_info)
    *   [`get_peers_state`](#get_peers_state)

## Chain

### `get_block`

Returns the information about a block by hash.

#### Parameters

    hash - Hash of a block

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_block",
    "params": [
        "0x73d857817071a0352ab05ea6be1342d0980a8d4797ab015ece3d78e1d26e5b16"
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "header": {
            "difficulty": "0x3e8",
            "epoch": "0",
            "hash": "0x73d857817071a0352ab05ea6be1342d0980a8d4797ab015ece3d78e1d26e5b16",
            "number": "2",
            "parent_hash": "0xbd0ec245b8a5742a9733643841aa6d2064942a3f137a13d412343affebaafa4d",
            "proposals_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "seal": {
                "nonce": "0",
                "proof": "0x"
            },
            "timestamp": "1557310745",
            "transactions_root": "0x46ab01ddbbabef1af701f0843e11c7cfc0ce53f9aa9b554af74cadf8e3257d89",
            "uncles_count": "0",
            "uncles_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "version": "0",
            "witnesses_root": "0x2e001e76e6169e7e82aa57d0a46f2e000e4960c17859be994e98cedee152b832"
        },
        "proposals": [],
        "transactions": [
            {
                "deps": [],
                "hash": "0x46ab01ddbbabef1af701f0843e11c7cfc0ce53f9aa9b554af74cadf8e3257d89",
                "inputs": [
                    {
                        "previous_output": {
                            "block_hash": null,
                            "cell": null
                        },
                        "since": "2"
                    }
                ],
                "outputs": [
                    {
                        "capacity": "50000000000000",
                        "data": "0x",
                        "lock": {
                            "args": [],
                            "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
                        },
                        "type": null
                    }
                ],
                "version": "0",
                "witnesses": []
            }
        ],
        "uncles": []
    }
}
```

### `get_block_by_number`

Get block by number

#### Parameters

    block_number - Number of a block

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_block_by_number",
    "params": [
        "1024"
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "header": {
            "difficulty": "0x3e8",
            "epoch": "0",
            "hash": "0xf35168c2e2e0c494ec97233091d5de51b7e5af7376bbc3d7572fc6438e2bb032",
            "number": "1024",
            "parent_hash": "0xbc76eadfcb64cf8d401c1a8bd2c7a5a94f5b3d03238f10540c6dfad2afdb705b",
            "proposals_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "seal": {
                "nonce": "0",
                "proof": "0x"
            },
            "timestamp": "1557311767",
            "transactions_root": "0xd82b0e5ade18260d822639b1310e585db39fa9fb9eea2cb1f4dfd3f62cd1330b",
            "uncles_count": "0",
            "uncles_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "version": "0",
            "witnesses_root": "0x86fdbf80284ae9a87357fa7390695a09a9daaaeb9e68d4f0fd67e2cd9ff35b73"
        },
        "proposals": [],
        "transactions": [
            {
                "deps": [],
                "hash": "0xd82b0e5ade18260d822639b1310e585db39fa9fb9eea2cb1f4dfd3f62cd1330b",
                "inputs": [
                    {
                        "previous_output": {
                            "block_hash": null,
                            "cell": null
                        },
                        "since": "1024"
                    }
                ],
                "outputs": [
                    {
                        "capacity": "50000000000000",
                        "data": "0x",
                        "lock": {
                            "args": [],
                            "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
                        },
                        "type": null
                    }
                ],
                "version": "0",
                "witnesses": []
            }
        ],
        "uncles": []
    }
}
```

### `get_block_hash`

Returns the hash of a block in the best-block-chain by block number; block of No.0 is the genesis block.

#### Parameters

    block_number - Number of a block

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_block_hash",
    "params": [
        "2"
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": "0x73d857817071a0352ab05ea6be1342d0980a8d4797ab015ece3d78e1d26e5b16"
}
```

### `get_cells_by_lock_hash`

Returns the information about cells collection by the hash of lock script.

#### Parameters

    lock_hash - Cell lock script hash
    from - Start block number
    to - End block number

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_cells_by_lock_hash",
    "params": [
        "0x9a9a6bdbc38d4905eace1822f85237e3a1e238bb3f277aa7b7c8903441123510",
        "2",
        "5"
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": [
        {
            "capacity": "50000000000000",
            "lock": {
                "args": [],
                "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
            },
            "out_point": {
                "block_hash": "0x73d857817071a0352ab05ea6be1342d0980a8d4797ab015ece3d78e1d26e5b16",
                "cell": {
                    "index": "0",
                    "tx_hash": "0x46ab01ddbbabef1af701f0843e11c7cfc0ce53f9aa9b554af74cadf8e3257d89"
                }
            }
        },
        {
            "capacity": "50000000000000",
            "lock": {
                "args": [],
                "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
            },
            "out_point": {
                "block_hash": "0x53a14d463327ba1fa06f649a29fb880514d5f3e67bc4034b92a321d0a95dd7d3",
                "cell": {
                    "index": "0",
                    "tx_hash": "0x9268bf1da68c1dbd8f78018021d1b7d84a99a2c81060b699fe39fd12d6243d25"
                }
            }
        },
        {
            "capacity": "50000000000000",
            "lock": {
                "args": [],
                "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
            },
            "out_point": {
                "block_hash": "0x5f90447a3d76e412a638615957970fa0b577ff7fc48d800597f6c3bf44a564a1",
                "cell": {
                    "index": "0",
                    "tx_hash": "0x1bad8cf98d1424ec47b6b1998d503b267edc8a05ec41409782b70aa6e61850ae"
                }
            }
        },
        {
            "capacity": "50000000000000",
            "lock": {
                "args": [],
                "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
            },
            "out_point": {
                "block_hash": "0x274433300b7ce5854efb5c0ee9b9a5dc366ce1257051d5366ea99c879ac1859b",
                "cell": {
                    "index": "0",
                    "tx_hash": "0x26871490afa7ea889d361772f2dd4aeca17ddc8dc43411b836f7665f7d14804f"
                }
            }
        }
    ]
}
```

### `get_current_epoch`

Returns the information about the current epoch.


#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_current_epoch",
    "params": []
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "block_reward": "100000000000",
        "difficulty": "0x3e8",
        "last_block_hash_in_previous_epoch": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "length": "1250",
        "number": "0",
        "remainder_reward": "0",
        "start_number": "0"
    }
}
```

### `get_epoch_by_number`

Return the information corresponding the given epoch number.

#### Parameters

    epoch_number - Epoch number

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_epoch_by_number",
    "params": [
        "0"
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "block_reward": "100000000000",
        "difficulty": "0x3e8",
        "last_block_hash_in_previous_epoch": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "length": "1250",
        "number": "0",
        "remainder_reward": "0",
        "start_number": "0"
    }
}
```

### `get_live_cell`

Returns the information about a cell by out_point. If <block_hash> is not specific, returns the cell if it is live. If <block_hash> is specified, return the live cell only if the corresponding block contain this cell

#### Parameters

    out_point - OutPoint object {{"tx_hash": <tx_hash>, "index": <index>}, "block_hash": <block_hash>}.

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_live_cell",
    "params": [
        {
            "block_hash": null,
            "cell": {
                "index": "0",
                "tx_hash": "0x26871490afa7ea889d361772f2dd4aeca17ddc8dc43411b836f7665f7d14804f"
            }
        }
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "cell": {
            "capacity": "50000000000000",
            "data": "0x",
            "lock": {
                "args": [],
                "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
            },
            "type": null
        },
        "status": "live"
    }
}
```

### `get_tip_block_number`

Returns the number of blocks in the longest blockchain.


#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_tip_block_number",
    "params": []
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": "1024"
}
```

### `get_tip_header`

Returns the information about the tip header of the longest.


#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_tip_header",
    "params": []
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "difficulty": "0x3e8",
        "epoch": "0",
        "hash": "0xf35168c2e2e0c494ec97233091d5de51b7e5af7376bbc3d7572fc6438e2bb032",
        "number": "1024",
        "parent_hash": "0xbc76eadfcb64cf8d401c1a8bd2c7a5a94f5b3d03238f10540c6dfad2afdb705b",
        "proposals_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "seal": {
            "nonce": "0",
            "proof": "0x"
        },
        "timestamp": "1557311767",
        "transactions_root": "0xd82b0e5ade18260d822639b1310e585db39fa9fb9eea2cb1f4dfd3f62cd1330b",
        "uncles_count": "0",
        "uncles_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "version": "0",
        "witnesses_root": "0x86fdbf80284ae9a87357fa7390695a09a9daaaeb9e68d4f0fd67e2cd9ff35b73"
    }
}
```

### `get_transaction`

Returns the information about a transaction requested by transaction hash.

#### Parameters

    hash - Hash of a transaction

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_transaction",
    "params": [
        "0xd82b0e5ade18260d822639b1310e585db39fa9fb9eea2cb1f4dfd3f62cd1330b"
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "transaction": {
            "deps": [],
            "hash": "0xd82b0e5ade18260d822639b1310e585db39fa9fb9eea2cb1f4dfd3f62cd1330b",
            "inputs": [
                {
                    "previous_output": {
                        "block_hash": null,
                        "cell": null
                    },
                    "since": "1024"
                }
            ],
            "outputs": [
                {
                    "capacity": "50000000000000",
                    "data": "0x",
                    "lock": {
                        "args": [],
                        "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
                    },
                    "type": null
                }
            ],
            "version": "0",
            "witnesses": []
        },
        "tx_status": {
            "block_hash": "0xf35168c2e2e0c494ec97233091d5de51b7e5af7376bbc3d7572fc6438e2bb032",
            "status": "committed"
        }
    }
}
```

## Experiment

### `_compute_code_hash`

Returns code hash of given hex encoded data

**Deprecated**: will be removed in a later version

#### Parameters

    data - Hex encoded data

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "_compute_code_hash",
    "params": [
        "0x123456"
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": "0x7dacea2e6ae8131b7f187570135ebb1b217a69458b3eae350104942c06939783"
}
```

### `_compute_script_hash`

Returns script hash of given transaction script

**Deprecated**: will be removed in a later version

#### Parameters

    args - Hex encoded arguments passed to reference cell
    code_hash - Code hash of referenced cell

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "_compute_script_hash",
    "params": [
        {
            "args": [
                "0x123450",
                "0x678900"
            ],
            "code_hash": "0xb35557e7e9854206f7bc13e3c3a7fa4cf8892c84a09237fb0aab40aab3771eee"
        }
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": "0x7c72a3b5705bf5a4e7364fc358e2972f4eb376cf7937bf7ffd319f50f07e27a2"
}
```

### `_compute_transaction_hash`

Return the transaction hash

**Deprecated**: will be removed in a later version

#### Parameters

    transaction - The transaction object
    version - Transaction version
    deps - Dependent cells
    inputs - Transaction inputs
    outputs - Transaction outputs
    witnesses - Witnesses

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "_compute_transaction_hash",
    "params": [
        {
            "deps": [],
            "inputs": [
                {
                    "args": [],
                    "previous_output": {
                        "block_hash": "0xf35168c2e2e0c494ec97233091d5de51b7e5af7376bbc3d7572fc6438e2bb032",
                        "cell": {
                            "index": "0",
                            "tx_hash": "0xd82b0e5ade18260d822639b1310e585db39fa9fb9eea2cb1f4dfd3f62cd1330b"
                        }
                    },
                    "since": "0"
                }
            ],
            "outputs": [
                {
                    "capacity": "100000000000",
                    "data": "0x",
                    "lock": {
                        "args": [],
                        "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
                    },
                    "type": null
                }
            ],
            "version": "0",
            "witnesses": []
        }
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": "0x30fe101b60a8ca326221886f230f2dcabc83c6cf639e869f1abdbbb5a90c9df7"
}
```

### `dry_run_transaction`

Dry run transaction and return the execution cycles.

This method will not check the transaction validity, but only run the lock script
and type script and then return the execution cycles.
Used to debug transaction scripts and query how many cycles the scripts consume


#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "dry_run_transaction",
    "params": [
        {
            "deps": [
                {
                    "cell": {
                        "index": "0",
                        "tx_hash": "0x2cc5d4811fe06f7745507deb3ade1155ea5037aa7b6cd108b6abae2e625fc00a"
                    }
                }
            ],
            "inputs": [
                {
                    "args": [],
                    "previous_output": {
                        "block_hash": "0xf35168c2e2e0c494ec97233091d5de51b7e5af7376bbc3d7572fc6438e2bb032",
                        "cell": {
                            "index": "0",
                            "tx_hash": "0xd82b0e5ade18260d822639b1310e585db39fa9fb9eea2cb1f4dfd3f62cd1330b"
                        }
                    },
                    "since": "0"
                }
            ],
            "outputs": [
                {
                    "capacity": "100000000000",
                    "data": "0x",
                    "lock": {
                        "args": [],
                        "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
                    },
                    "type": null
                }
            ],
            "version": "0",
            "witnesses": []
        }
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "cycles": "12"
    }
}
```

## Net

### `get_peers`

Returns the connected peers information.


#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_peers",
    "params": []
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": [
        {
            "addresses": [
                {
                    "address": "/ip4/192.168.0.3/tcp/8115",
                    "score": "1"
                }
            ],
            "is_outbound": true,
            "node_id": "QmaaaLB4uPyDpZwTQGhV63zuYrKm4reyN2tF1j2ain4oE7",
            "version": "unknown"
        },
        {
            "addresses": [
                {
                    "address": "/ip4/192.168.0.4/tcp/8113",
                    "score": "255"
                }
            ],
            "is_outbound": false,
            "node_id": "QmRuGcpVC3vE7aEoB6fhUdq9uzdHbyweCnn1sDBSjfmcbM",
            "version": "unknown"
        },
        {
            "addresses": [],
            "node_id": "QmUddxwRqgTmT6tFujXbYPMLGLAE2Tciyv6uHGfdYFyDVa",
            "version": "unknown"
        }
    ]
}
```

### `local_node_info`

Returns the local node information.


#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "local_node_info",
    "params": []
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "addresses": [
            {
                "address": "/ip4/192.168.0.2/tcp/8112/p2p/QmTRHCdrRtgUzYLNCin69zEvPvLYdxUZLLfLYyHVY3DZAS",
                "score": "255"
            },
            {
                "address": "/ip4/0.0.0.0/tcp/8112/p2p/QmTRHCdrRtgUzYLNCin69zEvPvLYdxUZLLfLYyHVY3DZAS",
                "score": "1"
            }
        ],
        "is_outbound": null,
        "node_id": "QmTRHCdrRtgUzYLNCin69zEvPvLYdxUZLLfLYyHVY3DZAS",
        "version": "0.9.0"
    }
}
```

## Pool

### `send_transaction`

Send new transaction into transaction pool

If <block_hash> of <previsous_output> is not specified, loads the corresponding input cell. If <block_hash> is specified, load the corresponding input cell only if the corresponding block exist and contain this cell as output.

#### Parameters

    transaction - The transaction object
    version - Transaction version
    deps - Dependent cells
    inputs - Transaction inputs
    outputs - Transaction outputs
    witnesses - Witnesses

#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "send_transaction",
    "params": [
        {
            "deps": [
                {
                    "cell": {
                        "index": "0",
                        "tx_hash": "0x2cc5d4811fe06f7745507deb3ade1155ea5037aa7b6cd108b6abae2e625fc00a"
                    }
                }
            ],
            "inputs": [
                {
                    "args": [],
                    "previous_output": {
                        "block_hash": "0xf35168c2e2e0c494ec97233091d5de51b7e5af7376bbc3d7572fc6438e2bb032",
                        "cell": {
                            "index": "0",
                            "tx_hash": "0xd82b0e5ade18260d822639b1310e585db39fa9fb9eea2cb1f4dfd3f62cd1330b"
                        }
                    },
                    "since": "0"
                }
            ],
            "outputs": [
                {
                    "capacity": "100000000000",
                    "data": "0x",
                    "lock": {
                        "args": [],
                        "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5"
                    },
                    "type": null
                }
            ],
            "version": "0",
            "witnesses": []
        }
    ]
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": "0x63c455673efbf458259f73052e73383c9d8644396ff6d8bc26a652b68d853f0f"
}
```

### `tx_pool_info`

Return the transaction pool information


#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "tx_pool_info",
    "params": []
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "last_txs_updated_at": "0",
        "orphan": "0",
        "pending": "1",
        "proposed": "0",
        "total_tx_cycles": "12",
        "total_tx_size": "156"
    }
}
```

## Stats

### `get_blockchain_info`

Return state info of blockchain


#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_blockchain_info",
    "params": []
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": {
        "alerts": [],
        "chain": "main",
        "difficulty": "0x3e8",
        "epoch": "0",
        "is_initial_block_download": true,
        "median_time": "1557311762"
    }
}
```

### `get_peers_state`

Deprecating in 0.12.0: Return state info of peers


#### Examples

```bash
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "get_peers_state",
    "params": []
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:8114
```

```json
{
    "id": 2,
    "jsonrpc": "2.0",
    "result": [
        {
            "blocks_in_flight": "86",
            "last_updated": "1557289448237",
            "peer": "1"
        }
    ]
}
```

