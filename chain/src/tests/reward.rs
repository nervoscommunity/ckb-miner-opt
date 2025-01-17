use crate::tests::util::{create_always_success_out_point, create_always_success_tx, start_chain};
use ckb_chain_spec::consensus::Consensus;
use ckb_core::block::{Block, BlockBuilder};
use ckb_core::header::{Header, HeaderBuilder};
use ckb_core::script::Script;
use ckb_core::transaction::{
    CellInput, CellOutput, OutPoint, ProposalShortId, Transaction, TransactionBuilder,
};
use ckb_core::uncle::UncleBlock;
use ckb_core::{capacity_bytes, BlockNumber, Bytes, Capacity};
use ckb_traits::ChainProvider;
use std::sync::Arc;
use test_chain_utils::create_always_success_cell;

const TX_FEE: Capacity = capacity_bytes!(10);
const BLOCK_REWARD: Capacity = capacity_bytes!(1_000);

pub(crate) fn create_cellbase(
    number: BlockNumber,
    miner_lock: Script,
    reward_lock: Script,
    reward: Option<Capacity>,
) -> Transaction {
    TransactionBuilder::default()
        .input(CellInput::new_cellbase_input(number))
        .output(CellOutput::new(
            reward.unwrap_or(BLOCK_REWARD),
            Bytes::default(),
            reward_lock,
            None,
        ))
        .witness(miner_lock.into_witness())
        .build()
}

pub(crate) fn gen_block(
    parent_header: &Header,
    transactions: Vec<Transaction>,
    proposals: Vec<ProposalShortId>,
    uncles: Vec<UncleBlock>,
    miner_lock: Script,
    reward_lock: Script,
    reward: Option<Capacity>,
) -> Block {
    let number = parent_header.number() + 1;
    let cellbase = create_cellbase(number, miner_lock, reward_lock, reward);
    let header_builder = HeaderBuilder::default()
        .parent_hash(parent_header.hash().to_owned())
        .timestamp(parent_header.timestamp() + 20_000)
        .number(number)
        .difficulty(parent_header.difficulty().clone());

    BlockBuilder::default()
        .transaction(cellbase)
        .transactions(transactions)
        .uncles(uncles)
        .proposals(proposals)
        .header_builder(header_builder)
        .build()
}

pub(crate) fn create_transaction(parent: &Transaction, index: u32) -> Transaction {
    let (_, always_success_script) = create_always_success_cell();
    let always_success_out_point = create_always_success_out_point();

    TransactionBuilder::default()
        .output(CellOutput::new(
            parent.outputs()[0].capacity.safe_sub(TX_FEE).unwrap(),
            Bytes::default(),
            always_success_script.clone(),
            None,
        ))
        .input(CellInput::new(
            OutPoint::new_cell(parent.hash().to_owned(), index),
            0,
        ))
        .dep(always_success_out_point)
        .build()
}

#[test]
fn finalize_reward() {
    let (_, always_success_script) = create_always_success_cell();
    let tx = TransactionBuilder::default()
        .input(CellInput::new(OutPoint::null(), 0))
        .output(CellOutput::new(
            capacity_bytes!(5_000),
            Bytes::default(),
            always_success_script.clone(),
            None,
        ))
        .build();

    let genesis_block = BlockBuilder::default()
        .transaction(create_always_success_tx())
        .transaction(tx.clone())
        .build();

    let consensus = Consensus::default()
        .set_cellbase_maturity(0)
        .set_bootstrap_lock(always_success_script.clone())
        .set_genesis_block(genesis_block);

    let (chain_controller, shared, mut parent) = start_chain(Some(consensus));

    let mut txs = Vec::with_capacity(16);
    let mut tx_parent = tx;
    for _i in 0..16 {
        tx_parent = create_transaction(&tx_parent, 0);
        txs.push(tx_parent.clone());
    }

    let ids: Vec<_> = txs.iter().map(Transaction::proposal_short_id).collect();

    let mut blocks = Vec::with_capacity(24);
    let bob = Script {
        args: vec![Bytes::from(b"b0b".to_vec())],
        code_hash: always_success_script.code_hash.clone(),
    };

    let alice = Script {
        args: vec![Bytes::from(b"a11ce".to_vec())],
        code_hash: always_success_script.code_hash.clone(),
    };

    for i in 1..23 {
        let proposals = if i == 12 {
            ids.iter().take(8).cloned().collect()
        } else if i == 13 {
            ids.clone()
        } else {
            vec![]
        };

        let miner_lock = if i == 12 {
            bob.clone()
        } else if i == 13 {
            alice.clone()
        } else {
            always_success_script.clone()
        };

        let block_txs = if i == 22 {
            txs.iter().take(12).cloned().collect()
        } else {
            vec![]
        };

        let block = gen_block(
            &parent,
            block_txs,
            proposals,
            vec![],
            miner_lock,
            always_success_script.clone(),
            None,
        );

        parent = block.header().clone();

        chain_controller
            .process_block(Arc::new(block.clone()), true)
            .expect("process block ok");
        blocks.push(block);
    }

    let (target, reward) = shared.finalize_block_reward(blocks[21].header()).unwrap();
    assert_eq!(target, bob);

    // bob proposed 8 txs in 12, committed in 22
    // get all proposal reward
    let bob_reward = TX_FEE
        .safe_mul_ratio(shared.consensus().proposer_reward_ratio())
        .unwrap()
        .safe_mul(8u8) // 8 txs
        .unwrap()
        .safe_add(BLOCK_REWARD)
        .unwrap();
    assert_eq!(reward, bob_reward,);

    let block = gen_block(
        &parent,
        txs.iter().skip(12).cloned().collect(),
        vec![],
        vec![],
        always_success_script.clone(),
        target,
        Some(bob_reward),
    );

    parent = block.header().clone();

    chain_controller
        .process_block(Arc::new(block.clone()), true)
        .expect("process block ok");

    let (target, reward) = shared.finalize_block_reward(block.header()).unwrap();
    assert_eq!(target, alice);

    // alice proposed 16 txs in block 13, committed in 22, 23
    // but bob proposed 8 txs earlier
    // get 8 proposal reward
    let alice_reward = TX_FEE
        .safe_mul_ratio(shared.consensus().proposer_reward_ratio())
        .unwrap()
        .safe_mul(8u8)
        .unwrap()
        .safe_add(BLOCK_REWARD)
        .unwrap();
    assert_eq!(reward, alice_reward);

    let block = gen_block(
        &parent,
        vec![],
        vec![],
        vec![],
        always_success_script.clone(),
        target,
        Some(alice_reward),
    );

    chain_controller
        .process_block(Arc::new(block.clone()), true)
        .expect("process block ok");
}
