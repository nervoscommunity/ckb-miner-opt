use crate::types::{
    CellTransaction, LiveCell, LockHashCellOutput, LockHashIndex, LockHashIndexState,
    TransactionPoint,
};
use bincode::{deserialize, serialize};
use ckb_core::block::Block;
use ckb_core::transaction::{CellOutPoint, CellOutput};
use ckb_core::BlockNumber;
use ckb_db::{
    rocksdb::{RocksDB, RocksdbBatch},
    Col, DBConfig, DbBatch, IterableKeyValueDB, KeyValueDB,
};
use ckb_notify::NotifyController;
use ckb_shared::shared::Shared;
use ckb_store::ChainStore;
use ckb_traits::chain_provider::ChainProvider;
use crossbeam_channel::{self, select};
use log::{error, trace};
use numext_fixed_hash::H256;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::thread;

const WALLET_STORE_SUBSCRIBER: &str = "wallet_store";

const COLUMNS: u32 = 4;

/// +---------------------------------+---------------+--------------------------+
/// |             Column              |      Key      |          Value           |
/// +---------------------------------+---------------+--------------------------+
/// | COLUMN_LOCK_HASH_INDEX_STATE    | H256          | LockHashIndexState       |
/// | COLUMN_LOCK_HASH_LIVE_CELL      | LockHashIndex | CellOutput               |
/// | COLUMN_LOCK_HASH_TRANSACTION    | LockHashIndex | Option<TransactionPoint> |
/// | COLUMN_CELL_OUT_POINT_LOCK_HASH | CellOutPoint  | LockHashCellOutput       |
/// +---------------------------------+---------------+--------------------------+

const COLUMN_LOCK_HASH_INDEX_STATE: Col = 0;
const COLUMN_LOCK_HASH_LIVE_CELL: Col = 1;
const COLUMN_LOCK_HASH_TRANSACTION: Col = 2;
const COLUMN_CELL_OUT_POINT_LOCK_HASH: Col = 3;

pub trait WalletStore: Sync + Send {
    fn get_live_cells(&self, lock_hash: &H256, skip_num: usize, take_num: usize) -> Vec<LiveCell>;

    fn get_transactions(
        &self,
        lock_hash: &H256,
        skip_num: usize,
        take_num: usize,
    ) -> Vec<CellTransaction>;

    fn get_lock_hash_index_states(&self) -> HashMap<H256, LockHashIndexState>;

    fn insert_lock_hash(&self, lock_hash: &H256, index_from: Option<BlockNumber>) -> LockHashIndexState;

    fn remove_lock_hash(&self, lock_hash: &H256);
}

pub struct DefaultWalletStore<CS> {
    db: Arc<RocksDB>,
    shared: Shared<CS>,
}

impl<CS: ChainStore> Clone for DefaultWalletStore<CS> {
    fn clone(&self) -> Self {
        DefaultWalletStore {
            db: Arc::clone(&self.db),
            shared: self.shared.clone(),
        }
    }
}

impl<CS: ChainStore + 'static> WalletStore for DefaultWalletStore<CS> {
    fn get_live_cells(&self, lock_hash: &H256, skip_num: usize, take_num: usize) -> Vec<LiveCell> {
        let iter = self
            .db
            .iter(COLUMN_LOCK_HASH_LIVE_CELL, lock_hash.as_bytes())
            .expect("wallet db iter should be ok");
        iter.skip(skip_num)
            .take(take_num)
            .take_while(|(key, _)| key.starts_with(lock_hash.as_bytes()))
            .map(|(key, value)| {
                let cell_output: CellOutput =
                    deserialize(&value).expect("deserialize CellOutput should be ok");
                let lock_hash_index = LockHashIndex::from_slice(&key);
                LiveCell {
                    created_by: lock_hash_index.into(),
                    cell_output,
                }
            })
            .collect()
    }

    fn get_transactions(
        &self,
        lock_hash: &H256,
        skip_num: usize,
        take_num: usize,
    ) -> Vec<CellTransaction> {
        let iter = self
            .db
            .iter(COLUMN_LOCK_HASH_TRANSACTION, lock_hash.as_bytes())
            .expect("wallet db iter should be ok");
        iter.skip(skip_num)
            .take(take_num)
            .take_while(|(key, _)| key.starts_with(lock_hash.as_bytes()))
            .map(|(key, value)| {
                let consumed_by: Option<TransactionPoint> =
                    deserialize(&value).expect("deserialize TransactionPoint should be ok");
                let lock_hash_index = LockHashIndex::from_slice(&key);
                CellTransaction {
                    created_by: lock_hash_index.into(),
                    consumed_by,
                }
            })
            .collect()
    }

    fn get_lock_hash_index_states(&self) -> HashMap<H256, LockHashIndexState> {
        self.db
            .iter(COLUMN_LOCK_HASH_INDEX_STATE, &[])
            .expect("wallet db iter should be ok")
            .map(|(key, value)| {
                (
                    H256::from_slice(&key).expect("db safe access"),
                    deserialize(&value).expect("deserialize LockHashIndexState should be ok"),
                )
            })
            .collect()
    }

    fn insert_lock_hash(&self, lock_hash: &H256, index_from: Option<BlockNumber>) -> LockHashIndexState {
        // need to lock chain state, avoids inconsistent state in processing
        let chain_state = self.shared.lock_chain_state();
        let index_state = LockHashIndexState {
            block_number: chain_state.tip_number(),
            block_hash: chain_state.tip_hash().to_owned(),
        };
        self.commit_batch(|batch| {
            if let Some(from_block_number) = index_from {
                let mut batch_buffer = HashMap::<CellOutPoint, LockHashCellOutput>::new();
                let mut index_lock_hashes = HashSet::new();
                index_lock_hashes.insert(lock_hash.to_owned());
                (from_block_number..=chain_state.tip_number()).for_each(|block_number| {
                    let block = self
                        .shared
                        .block_hash(block_number)
                        .and_then(|hash| self.shared.block(&hash))
                        .expect("block exists");
                    self.attach_block(batch, &mut batch_buffer, &index_lock_hashes, &block);
                });
            }
            batch.insert_lock_hash_index_state(lock_hash, &index_state);
        });
        index_state
    }

    fn remove_lock_hash(&self, lock_hash: &H256) {
        self.commit_batch(|batch| {
            let iter = self
                .db
                .iter(COLUMN_LOCK_HASH_LIVE_CELL, lock_hash.as_bytes())
                .expect("wallet db iter should be ok");

            iter.take_while(|(key, _)| key.starts_with(lock_hash.as_bytes()))
                .for_each(|(key, _)| {
                    let lock_hash_index = LockHashIndex::from_slice(&key);
                    batch.delete_lock_hash_live_cell(&lock_hash_index);
                    batch.delete_cell_out_point_lock_hash(&lock_hash_index.cell_out_point);
                });

            let iter = self
                .db
                .iter(COLUMN_LOCK_HASH_TRANSACTION, lock_hash.as_bytes())
                .expect("wallet db iter should be ok");

            iter.take_while(|(key, _)| key.starts_with(lock_hash.as_bytes()))
                .for_each(|(key, _)| {
                    let lock_hash_index = LockHashIndex::from_slice(&key);
                    batch.delete_lock_hash_transaction(&lock_hash_index);
                });

            batch.delete_lock_hash_index_state(&lock_hash);
        });
    }
}

impl<CS: ChainStore + 'static> DefaultWalletStore<CS> {
    pub fn new(db_config: &DBConfig, shared: Shared<CS>) -> Self {
        let db = RocksDB::open(db_config, COLUMNS);
        DefaultWalletStore {
            db: Arc::new(db),
            shared,
        }
    }

    pub fn start<S: ToString>(self, thread_name: Option<S>, notify: &NotifyController) {
        let mut thread_builder = thread::Builder::new();
        if let Some(name) = thread_name {
            thread_builder = thread_builder.name(name.to_string());
        }

        let new_tip_receiver = notify.subscribe_new_tip(WALLET_STORE_SUBSCRIBER);
        thread_builder
            .spawn(move || loop {
                select! {
                    recv(new_tip_receiver) -> msg => match msg {
                        Ok(tip_changes) => self.update(&tip_changes.detached_blocks, &tip_changes.attached_blocks),
                        _ => {
                            error!(target: "wallet", "new_tip_receiver closed");
                            break;
                        }
                    },
                }
            })
            .expect("start DefaultWalletStore failed");
    }

    // helper function
    fn commit_batch<F>(&self, process: F)
    where
        F: FnOnce(&mut WalletStoreBatch),
    {
        match self.db.batch() {
            Ok(batch) => {
                let mut batch = WalletStoreBatch { batch };
                process(&mut batch);
                batch.commit();
            }
            Err(err) => {
                error!(target: "wallet", "wallet db failed to create new batch, error: {:?}", err);
            }
        }
    }

    pub fn sync_index_states(&self) {
        let mut lock_hash_index_states = self.get_lock_hash_index_states();
        if lock_hash_index_states.is_empty() {
            return;
        }
        // need to lock chain state, avoids inconsistent state in processing
        let chain_state = self.shared.lock_chain_state();
        // retains the lock hashes on fork chain and detach blocks
        lock_hash_index_states.retain(|_, index_state| {
            self.shared.block_number(&index_state.block_hash) != Some(index_state.block_number)
        });
        lock_hash_index_states
            .iter()
            .for_each(|(lock_hash, index_state)| {
                let mut index_lock_hashes = HashSet::new();
                index_lock_hashes.insert(lock_hash.to_owned());

                let mut block = self
                    .shared
                    .block(&index_state.block_hash)
                    .expect("block exists");
                // detach blocks until reach a block on main chain
                self.commit_batch(|batch| {
                    self.detach_block(batch, &index_lock_hashes, &block);
                    while self.shared.block_hash(block.header().number() - 1)
                        != Some(block.header().parent_hash().to_owned())
                    {
                        block = self
                            .shared
                            .block(block.header().parent_hash())
                            .expect("block exists");
                        self.detach_block(batch, &index_lock_hashes, &block);
                    }
                    let index_state = LockHashIndexState {
                        block_number: block.header().number() - 1,
                        block_hash: block.header().parent_hash().to_owned(),
                    };
                    batch.insert_lock_hash_index_state(lock_hash, &index_state);
                });
            });

        // attach blocks until reach tip
        let lock_hash_index_states = self.get_lock_hash_index_states();
        let min_block_number: BlockNumber = lock_hash_index_states
            .values()
            .min_by_key(|index_state| index_state.block_number)
            .expect("none empty index states")
            .block_number;
        self.commit_batch(|batch| {
            let mut batch_buffer = HashMap::<CellOutPoint, LockHashCellOutput>::new();
            let index_lock_hashes = lock_hash_index_states.keys().cloned().collect();
            (min_block_number + 1..=chain_state.tip_number()).for_each(|block_number| {
                let block = self
                    .shared
                    .block_hash(block_number)
                    .and_then(|hash| self.shared.block(&hash))
                    .expect("block exists");
                self.attach_block(batch, &mut batch_buffer, &index_lock_hashes, &block);
            });
            let index_state = LockHashIndexState {
                block_number: chain_state.tip_number(),
                block_hash: chain_state.tip_hash().to_owned(),
            };
            index_lock_hashes.iter().for_each(|lock_hash| {
                batch.insert_lock_hash_index_state(lock_hash, &index_state);
            })
        });
    }

    pub(crate) fn update(&self, detached_blocks: &[Block], attached_blocks: &[Block]) {
        let index_lock_hashes: HashSet<H256> =
            self.get_lock_hash_index_states().keys().cloned().collect();
        if !index_lock_hashes.is_empty() {
            self.commit_batch(|batch| {
                detached_blocks
                    .iter()
                    .for_each(|block| self.detach_block(batch, &index_lock_hashes, block));
                // rocksdb rust binding doesn't support transactional batch read, have to use a batch buffer here.
                let mut batch_buffer = HashMap::<CellOutPoint, LockHashCellOutput>::new();
                attached_blocks.iter().for_each(|block| {
                    self.attach_block(batch, &mut batch_buffer, &index_lock_hashes, block)
                });
                if let Some(block) = attached_blocks.last() {
                    let index_state = LockHashIndexState {
                        block_number: block.header().number(),
                        block_hash: block.header().hash().to_owned(),
                    };
                    index_lock_hashes.iter().for_each(|lock_hash| {
                        batch.insert_lock_hash_index_state(lock_hash, &index_state);
                    })
                }
            });
        }
    }

    fn detach_block(
        &self,
        batch: &mut WalletStoreBatch,
        index_lock_hashes: &HashSet<H256>,
        block: &Block,
    ) {
        trace!(target: "wallet", "detach block {:x}", block.header().hash());
        let block_number = block.header().number();
        block.transactions().iter().for_each(|tx| {
            let tx_hash = tx.hash();
            if !tx.is_cellbase() {
                tx.inputs().iter().enumerate().for_each(|(index, input)| {
                    let index = index as u32;
                    let cell_out_point = input.previous_output.cell.clone().expect("cell exists");
                    if let Some(mut lock_hash_cell_output) =
                        self.get_lock_hash_cell_output(&cell_out_point)
                    {
                        if index_lock_hashes.contains(&lock_hash_cell_output.lock_hash) {
                            let lock_hash_index = LockHashIndex::new(
                                lock_hash_cell_output.lock_hash.clone(),
                                block_number,
                                tx_hash.clone(),
                                index,
                            );
                            batch.insert_lock_hash_live_cell(
                                &lock_hash_index,
                                &lock_hash_cell_output
                                    .cell_output
                                    .expect("inconsistent state"),
                            );
                            batch.insert_lock_hash_transaction(&lock_hash_index, &None);

                            lock_hash_cell_output.cell_output = None;
                            batch.insert_cell_out_point_lock_hash(
                                &cell_out_point,
                                &lock_hash_cell_output,
                            );
                        }
                    }
                });
            }

            tx.outputs().iter().enumerate().for_each(|(index, output)| {
                let index = index as u32;
                let lock_hash = output.lock.hash();
                if index_lock_hashes.contains(&lock_hash) {
                    let lock_hash_index =
                        LockHashIndex::new(lock_hash, block_number, tx_hash.clone(), index);

                    batch.delete_lock_hash_live_cell(&lock_hash_index);
                    batch.delete_lock_hash_transaction(&lock_hash_index);
                    batch.delete_cell_out_point_lock_hash(&lock_hash_index.cell_out_point);
                }
            });
        })
    }

    fn attach_block(
        &self,
        batch: &mut WalletStoreBatch,
        batch_buffer: &mut HashMap<CellOutPoint, LockHashCellOutput>,
        index_lock_hashes: &HashSet<H256>,
        block: &Block,
    ) {
        trace!(target: "wallet", "attach block {:x}", block.header().hash());
        let block_number = block.header().number();
        block.transactions().iter().for_each(|tx| {
            let tx_hash = tx.hash();
            tx.outputs().iter().enumerate().for_each(|(index, output)| {
                let index = index as u32;
                let lock_hash = output.lock.hash();
                if index_lock_hashes.contains(&lock_hash) {
                    let lock_hash_index =
                        LockHashIndex::new(lock_hash.clone(), block_number, tx_hash.clone(), index);
                    batch.insert_lock_hash_live_cell(&lock_hash_index, output);
                    batch.insert_lock_hash_transaction(&lock_hash_index, &None);

                    let mut lock_hash_cell_output = LockHashCellOutput {
                        lock_hash,
                        block_number,
                        cell_output: None,
                    };
                    let cell_out_point = CellOutPoint {
                        tx_hash: tx_hash.clone(),
                        index,
                    };
                    batch.insert_cell_out_point_lock_hash(&cell_out_point, &lock_hash_cell_output);

                    // insert lock_hash_cell_output as a cached value
                    lock_hash_cell_output.cell_output = Some(output.clone());
                    batch_buffer.insert(cell_out_point, lock_hash_cell_output);
                }
            });

            if !tx.is_cellbase() {
                tx.inputs().iter().enumerate().for_each(|(index, input)| {
                    // lookup lock_hash in the batch buffer and store
                    let index = index as u32;
                    let cell_out_point = input.previous_output.cell.clone().expect("cell exists");
                    if let Some(lock_hash_cell_output) = batch_buffer
                        .get(&cell_out_point)
                        .cloned()
                        .or_else(|| self.get_lock_hash_cell_output(&cell_out_point))
                    {
                        if index_lock_hashes.contains(&lock_hash_cell_output.lock_hash) {
                            batch.insert_cell_out_point_lock_hash(
                                &cell_out_point,
                                &lock_hash_cell_output,
                            );
                            let lock_hash_index = LockHashIndex::new(
                                lock_hash_cell_output.lock_hash,
                                lock_hash_cell_output.block_number,
                                cell_out_point.tx_hash,
                                cell_out_point.index,
                            );
                            let consumed_by = TransactionPoint {
                                block_number,
                                tx_hash: tx_hash.clone(),
                                index,
                            };
                            batch.delete_lock_hash_live_cell(&lock_hash_index);
                            batch
                                .insert_lock_hash_transaction(&lock_hash_index, &Some(consumed_by));
                        }
                    }
                });
            }
        })
    }

    fn get_lock_hash_cell_output(
        &self,
        cell_out_point: &CellOutPoint,
    ) -> Option<LockHashCellOutput> {
        self.db
            .read(
                COLUMN_CELL_OUT_POINT_LOCK_HASH,
                &serialize(cell_out_point).expect("serialize OutPoint should be ok"),
            )
            .expect("wallet db read should be ok")
            .map(|value| deserialize(&value).expect("deserialize LockHashCellOutput should be ok"))
    }
}

struct WalletStoreBatch {
    pub batch: RocksdbBatch,
}

impl WalletStoreBatch {
    fn insert_lock_hash_index_state(&mut self, lock_hash: &H256, index_state: &LockHashIndexState) {
        self.batch
            .insert(
                COLUMN_LOCK_HASH_INDEX_STATE,
                lock_hash.as_bytes(),
                &serialize(index_state).expect("serialize LockHashIndexState should be ok"),
            )
            .expect("batch insert COLUMN_LOCK_HASH_INDEX_STATE failed");
    }

    fn insert_lock_hash_live_cell(
        &mut self,
        lock_hash_index: &LockHashIndex,
        cell_output: &CellOutput,
    ) {
        self.batch
            .insert(
                COLUMN_LOCK_HASH_LIVE_CELL,
                &lock_hash_index.to_vec(),
                &serialize(cell_output).expect("serialize CellOutput should be ok"),
            )
            .expect("batch insert COLUMN_LOCK_HASH_LIVE_CELL failed");
    }

    fn insert_lock_hash_transaction(
        &mut self,
        lock_hash_index: &LockHashIndex,
        consumed_by: &Option<TransactionPoint>,
    ) {
        self.batch
            .insert(
                COLUMN_LOCK_HASH_TRANSACTION,
                &lock_hash_index.to_vec(),
                &serialize(consumed_by).expect("serialize TransactionPoint should be ok"),
            )
            .expect("batch insert COLUMN_LOCK_HASH_TRANSACTION failed");
    }

    fn insert_cell_out_point_lock_hash(
        &mut self,
        cell_out_point: &CellOutPoint,
        lock_hash_cell_output: &LockHashCellOutput,
    ) {
        self.batch
            .insert(
                COLUMN_CELL_OUT_POINT_LOCK_HASH,
                &serialize(&cell_out_point).expect("serialize OutPoint should be ok"),
                &serialize(&lock_hash_cell_output)
                    .expect("serialize LockHashCellOutput should be ok"),
            )
            .expect("batch insert COLUMN_CELL_OUT_POINT_LOCK_HASH failed");
    }

    fn delete_lock_hash_index_state(&mut self, lock_hash: &H256) {
        self.batch
            .delete(COLUMN_LOCK_HASH_INDEX_STATE, lock_hash.as_bytes())
            .expect("batch delete COLUMN_LOCK_HASH_INDEX_STATE failed");
    }

    fn delete_lock_hash_live_cell(&mut self, lock_hash_index: &LockHashIndex) {
        self.batch
            .delete(COLUMN_LOCK_HASH_LIVE_CELL, &lock_hash_index.to_vec())
            .expect("batch delete COLUMN_LOCK_HASH_LIVE_CELL failed");
    }

    fn delete_lock_hash_transaction(&mut self, lock_hash_index: &LockHashIndex) {
        self.batch
            .delete(COLUMN_LOCK_HASH_TRANSACTION, &lock_hash_index.to_vec())
            .expect("batch delete COLUMN_LOCK_HASH_TRANSACTION failed");
    }

    fn delete_cell_out_point_lock_hash(&mut self, cell_out_point: &CellOutPoint) {
        self.batch
            .delete(
                COLUMN_CELL_OUT_POINT_LOCK_HASH,
                &serialize(cell_out_point).expect("serialize CellOutPoint should be ok"),
            )
            .expect("batch delete COLUMN_CELL_OUT_POINT_LOCK_HASH failed");
    }

    fn commit(self) {
        // only log the error, wallet store commit failure should not causing the thread to panic entirely.
        if let Err(err) = self.batch.commit() {
            error!(target: "wallet", "wallet db failed to commit batch, error: {:?}", err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckb_chain::chain::{ChainController, ChainService};
    use ckb_chain_spec::consensus::Consensus;
    use ckb_core::block::BlockBuilder;
    use ckb_core::header::HeaderBuilder;
    use ckb_core::script::{Script, DAO_CODE_HASH};
    use ckb_core::transaction::{CellInput, CellOutput, OutPoint, TransactionBuilder};
    use ckb_core::{capacity_bytes, Bytes, Capacity};
    use ckb_db::{DBConfig, MemoryKeyValueDB};
    use ckb_notify::NotifyService;
    use ckb_shared::shared::{Shared, SharedBuilder};
    use ckb_store::ChainKVStore;
    use numext_fixed_uint::U256;
    use std::sync::Arc;
    use tempfile;

    fn setup(
        prefix: &str,
    ) -> (
        DefaultWalletStore<ChainKVStore<MemoryKeyValueDB>>,
        ChainController,
        Shared<ChainKVStore<MemoryKeyValueDB>>,
    ) {
        let builder = SharedBuilder::<MemoryKeyValueDB>::new();
        let shared = builder.consensus(Consensus::default()).build().unwrap();

        let tmp_dir = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
        let config = DBConfig {
            path: tmp_dir.as_ref().to_path_buf(),
            ..Default::default()
        };
        let notify = NotifyService::default().start::<&str>(None);
        let chain_service = ChainService::new(shared.clone(), notify);
        let chain_controller = chain_service.start::<&str>(None);
        (
            DefaultWalletStore::new(&config, shared.clone()),
            chain_controller,
            shared,
        )
    }

    #[test]
    fn lock_hash_index() {
        let (store, _, _) = setup("lock_hash_index");
        store.insert_lock_hash(&DAO_CODE_HASH, None);
        store.insert_lock_hash(&H256::zero(), None);

        assert_eq!(2, store.get_lock_hash_index_states().len());

        store.remove_lock_hash(&DAO_CODE_HASH);
        assert_eq!(1, store.get_lock_hash_index_states().len());
    }

    #[test]
    fn get_live_cells() {
        let (store, _, _) = setup("get_live_cells");
        let script1 = Script::new(Vec::new(), DAO_CODE_HASH);
        let script2 = Script::default();
        store.insert_lock_hash(&script1.hash(), None);
        store.insert_lock_hash(&script2.hash(), None);

        let tx11 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(1000),
                Bytes::new(),
                script1.clone(),
                None,
            ))
            .build();

        let tx12 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(2000),
                Bytes::new(),
                script2.clone(),
                None,
            ))
            .build();

        let block1 = BlockBuilder::default()
            .transaction(tx11.clone())
            .transaction(tx12.clone())
            .header_builder(HeaderBuilder::default().number(1))
            .build();

        let tx21 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(3000),
                Bytes::new(),
                script1.clone(),
                None,
            ))
            .build();

        let tx22 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(4000),
                Bytes::new(),
                script2.clone(),
                None,
            ))
            .build();

        let block2 = BlockBuilder::default()
            .transaction(tx21.clone())
            .transaction(tx22.clone())
            .header_builder(HeaderBuilder::default().number(2))
            .build();

        let tx31 = TransactionBuilder::default()
            .input(CellInput::new(
                OutPoint::new_cell(tx11.hash().to_owned(), 0),
                0,
                vec![],
            ))
            .output(CellOutput::new(
                capacity_bytes!(5000),
                Bytes::new(),
                script1.clone(),
                None,
            ))
            .build();

        let tx32 = TransactionBuilder::default()
            .input(CellInput::new(
                OutPoint::new_cell(tx12.hash().to_owned(), 0),
                0,
                vec![],
            ))
            .output(CellOutput::new(
                capacity_bytes!(6000),
                Bytes::new(),
                script2.clone(),
                None,
            ))
            .build();

        let block3 = BlockBuilder::default()
            .transaction(tx31.clone())
            .transaction(tx32.clone())
            .header_builder(HeaderBuilder::default().number(3))
            .build();

        store.update(&[], &[block1, block2.clone()]);
        let cells = store.get_live_cells(&script1.hash(), 0, 100);
        assert_eq!(2, cells.len());
        assert_eq!(capacity_bytes!(1000), cells[0].cell_output.capacity);
        assert_eq!(capacity_bytes!(3000), cells[1].cell_output.capacity);

        let cells = store.get_live_cells(&script2.hash(), 0, 100);
        assert_eq!(2, cells.len());
        assert_eq!(capacity_bytes!(2000), cells[0].cell_output.capacity);
        assert_eq!(capacity_bytes!(4000), cells[1].cell_output.capacity);

        store.update(&[block2], &[block3]);
        let cells = store.get_live_cells(&script1.hash(), 0, 100);
        assert_eq!(1, cells.len());
        assert_eq!(capacity_bytes!(5000), cells[0].cell_output.capacity);

        let cells = store.get_live_cells(&script2.hash(), 0, 100);
        assert_eq!(1, cells.len());
        assert_eq!(capacity_bytes!(6000), cells[0].cell_output.capacity);

        // remove script1's lock hash should remove its indexed data also
        store.remove_lock_hash(&script1.hash());
        let cells = store.get_live_cells(&script1.hash(), 0, 100);
        assert_eq!(0, cells.len());
        let cells = store.get_live_cells(&script2.hash(), 0, 100);
        assert_eq!(1, cells.len());
    }

    #[test]
    fn get_transactions() {
        let (store, _, _) = setup("get_transactions");
        let script1 = Script::new(Vec::new(), DAO_CODE_HASH);
        let script2 = Script::default();
        store.insert_lock_hash(&script1.hash(), None);
        store.insert_lock_hash(&script2.hash(), None);

        let tx11 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(1000),
                Bytes::new(),
                script1.clone(),
                None,
            ))
            .build();

        let tx12 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(2000),
                Bytes::new(),
                script2.clone(),
                None,
            ))
            .build();

        let block1 = BlockBuilder::default()
            .transaction(tx11.clone())
            .transaction(tx12.clone())
            .header_builder(HeaderBuilder::default().number(1))
            .build();

        let tx21 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(3000),
                Bytes::new(),
                script1.clone(),
                None,
            ))
            .build();

        let tx22 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(4000),
                Bytes::new(),
                script2.clone(),
                None,
            ))
            .build();

        let block2 = BlockBuilder::default()
            .transaction(tx21.clone())
            .transaction(tx22.clone())
            .header_builder(HeaderBuilder::default().number(2))
            .build();

        let tx31 = TransactionBuilder::default()
            .input(CellInput::new(
                OutPoint::new_cell(tx11.hash().to_owned(), 0),
                0,
                vec![],
            ))
            .output(CellOutput::new(
                capacity_bytes!(5000),
                Bytes::new(),
                script1.clone(),
                None,
            ))
            .build();

        let tx32 = TransactionBuilder::default()
            .input(CellInput::new(
                OutPoint::new_cell(tx12.hash().to_owned(), 0),
                0,
                vec![],
            ))
            .output(CellOutput::new(
                capacity_bytes!(6000),
                Bytes::new(),
                script2.clone(),
                None,
            ))
            .build();

        let block3 = BlockBuilder::default()
            .transaction(tx31.clone())
            .transaction(tx32.clone())
            .header_builder(HeaderBuilder::default().number(3))
            .build();

        store.update(&[], &[block1, block2.clone()]);
        let transactions = store.get_transactions(&script1.hash(), 0, 100);
        assert_eq!(2, transactions.len());
        assert_eq!(tx11.hash().to_owned(), transactions[0].created_by.tx_hash);
        assert_eq!(tx21.hash().to_owned(), transactions[1].created_by.tx_hash);

        let transactions = store.get_transactions(&script2.hash(), 0, 100);
        assert_eq!(2, transactions.len());
        assert_eq!(tx12.hash().to_owned(), transactions[0].created_by.tx_hash);
        assert_eq!(tx22.hash().to_owned(), transactions[1].created_by.tx_hash);

        store.update(&[block2], &[block3]);
        let transactions = store.get_transactions(&script1.hash(), 0, 100);
        assert_eq!(2, transactions.len());
        assert_eq!(tx11.hash().to_owned(), transactions[0].created_by.tx_hash);
        assert_eq!(
            Some(tx31.hash().to_owned()),
            transactions[0]
                .consumed_by
                .as_ref()
                .map(|transaction_point| transaction_point.tx_hash.clone())
        );
        assert_eq!(tx31.hash().to_owned(), transactions[1].created_by.tx_hash);

        let transactions = store.get_transactions(&script2.hash(), 0, 100);
        assert_eq!(2, transactions.len());
        assert_eq!(tx12.hash().to_owned(), transactions[0].created_by.tx_hash);
        assert_eq!(tx32.hash().to_owned(), transactions[1].created_by.tx_hash);

        // remove script1's lock hash should remove its indexed data also
        store.remove_lock_hash(&script1.hash());
        let transactions = store.get_transactions(&script1.hash(), 0, 100);
        assert_eq!(0, transactions.len());
        let transactions = store.get_transactions(&script2.hash(), 0, 100);
        assert_eq!(2, transactions.len());
    }

    #[test]
    fn sync_index_states() {
        let (store, chain, shared) = setup("sync_index_states");
        let script1 = Script::new(Vec::new(), DAO_CODE_HASH);
        let script2 = Script::default();
        store.insert_lock_hash(&script1.hash(), None);
        store.insert_lock_hash(&script2.hash(), None);

        let tx11 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(1000),
                Bytes::new(),
                script1.clone(),
                None,
            ))
            .build();

        let tx12 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(2000),
                Bytes::new(),
                script2.clone(),
                None,
            ))
            .build();

        let block1 = BlockBuilder::default()
            .transaction(tx11.clone())
            .transaction(tx12.clone())
            .header_builder(
                HeaderBuilder::default()
                    .difficulty(U256::from(1u64))
                    .number(1)
                    .parent_hash(shared.genesis_hash().to_owned()),
            )
            .build();

        let tx21 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(3000),
                Bytes::new(),
                script1.clone(),
                None,
            ))
            .build();

        let tx22 = TransactionBuilder::default()
            .output(CellOutput::new(
                capacity_bytes!(4000),
                Bytes::new(),
                script2.clone(),
                None,
            ))
            .build();

        let block2 = BlockBuilder::default()
            .transaction(tx21.clone())
            .transaction(tx22.clone())
            .header_builder(
                HeaderBuilder::default()
                    .difficulty(U256::from(2u64))
                    .number(2)
                    .parent_hash(block1.header().hash().to_owned()),
            )
            .build();

        let tx31 = TransactionBuilder::default()
            .input(CellInput::new(
                OutPoint::new_cell(tx11.hash().to_owned(), 0),
                0,
                vec![],
            ))
            .output(CellOutput::new(
                capacity_bytes!(5000),
                Bytes::new(),
                script1.clone(),
                None,
            ))
            .build();

        let tx32 = TransactionBuilder::default()
            .input(CellInput::new(
                OutPoint::new_cell(tx12.hash().to_owned(), 0),
                0,
                vec![],
            ))
            .output(CellOutput::new(
                capacity_bytes!(6000),
                Bytes::new(),
                script2.clone(),
                None,
            ))
            .build();

        let block2_fork = BlockBuilder::default()
            .header_builder(
                HeaderBuilder::default()
                    .difficulty(U256::from(20u64))
                    .number(2)
                    .parent_hash(block1.header().hash().to_owned()),
            )
            .build();

        let block3 = BlockBuilder::default()
            .transaction(tx31.clone())
            .transaction(tx32.clone())
            .header_builder(
                HeaderBuilder::default()
                    .difficulty(U256::from(21u64))
                    .number(3)
                    .parent_hash(block2_fork.header().hash().to_owned()),
            )
            .build();

        chain.process_block(Arc::new(block1), false).unwrap();
        chain.process_block(Arc::new(block2), false).unwrap();

        store.sync_index_states();

        let transactions = store.get_transactions(&script1.hash(), 0, 100);
        assert_eq!(2, transactions.len());
        assert_eq!(tx11.hash().to_owned(), transactions[0].created_by.tx_hash);
        assert_eq!(tx21.hash().to_owned(), transactions[1].created_by.tx_hash);

        let transactions = store.get_transactions(&script2.hash(), 0, 100);
        assert_eq!(2, transactions.len());
        assert_eq!(tx12.hash().to_owned(), transactions[0].created_by.tx_hash);
        assert_eq!(tx22.hash().to_owned(), transactions[1].created_by.tx_hash);

        chain.process_block(Arc::new(block2_fork), false).unwrap();
        chain.process_block(Arc::new(block3), false).unwrap();

        store.sync_index_states();
        let transactions = store.get_transactions(&script1.hash(), 0, 100);
        assert_eq!(2, transactions.len());
        assert_eq!(tx11.hash().to_owned(), transactions[0].created_by.tx_hash);
        assert_eq!(
            Some(tx31.hash().to_owned()),
            transactions[0]
                .consumed_by
                .as_ref()
                .map(|transaction_point| transaction_point.tx_hash.clone())
        );
        assert_eq!(tx31.hash().to_owned(), transactions[1].created_by.tx_hash);

        let transactions = store.get_transactions(&script2.hash(), 0, 100);
        assert_eq!(2, transactions.len());
        assert_eq!(tx12.hash().to_owned(), transactions[0].created_by.tx_hash);
        assert_eq!(tx32.hash().to_owned(), transactions[1].created_by.tx_hash);
    }
}