use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use bitcell_state::{Account, StorageManager};
use tempfile::TempDir;

fn bench_block_storage(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path()).unwrap();
    
    let mut group = c.benchmark_group("block_storage");
    
    // Benchmark single block storage
    group.bench_function("store_header", |b| {
        let mut height = 0u64;
        b.iter(|| {
            let hash = format!("hash_{:032}", height);
            let header = format!("header_data_{}", height);
            storage.store_header(
                black_box(height),
                black_box(hash.as_bytes()),
                black_box(header.as_bytes())
            ).unwrap();
            height += 1;
        });
    });
    
    // Benchmark block retrieval by height
    // First, store some blocks
    for i in 0..1000 {
        let hash = format!("hash_{:032}", i);
        let header = format!("header_data_{}", i);
        storage.store_header(i, hash.as_bytes(), header.as_bytes()).unwrap();
    }
    
    group.bench_function("get_header_by_height", |b| {
        let mut height = 0u64;
        b.iter(|| {
            let result = storage.get_header_by_height(black_box(height % 1000)).unwrap();
            height += 1;
            result
        });
    });
    
    group.bench_function("get_header_by_hash", |b| {
        let mut height = 0u64;
        b.iter(|| {
            let hash = format!("hash_{:032}", height % 1000);
            let result = storage.get_header_by_hash(black_box(hash.as_bytes())).unwrap();
            height += 1;
            result
        });
    });
    
    group.finish();
}

fn bench_transaction_indexing(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path()).unwrap();
    
    let mut group = c.benchmark_group("transaction_indexing");
    group.throughput(Throughput::Elements(1));
    
    // Benchmark single transaction storage
    group.bench_function("store_transaction", |b| {
        let mut tx_num = 0u64;
        b.iter(|| {
            let tx_hash = format!("tx_hash_{:032}", tx_num);
            let sender = format!("sender_{:034}", tx_num % 100);
            let tx_data = format!("tx_data_{}", tx_num);
            
            storage.store_transaction(
                black_box(tx_hash.as_bytes()),
                black_box(sender.as_bytes()),
                black_box(tx_data.as_bytes()),
                black_box(tx_num)
            ).unwrap();
            tx_num += 1;
        });
    });
    
    // Benchmark batch transaction storage
    for batch_size in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("store_transactions_batch", batch_size),
            batch_size,
            |b, &size| {
                let mut start_num = 0u64;
                b.iter(|| {
                    let mut batch = Vec::with_capacity(size);
                    for i in 0..size {
                        let tx_num = start_num + i as u64;
                        let tx_hash = format!("tx_hash_{:032}", tx_num);
                        let sender = format!("sender_{:034}", tx_num % 100);
                        let tx_data = format!("tx_data_{}", tx_num);
                        
                        // Note: We need to keep these strings alive for the batch
                        batch.push((tx_hash, sender, tx_data));
                    }
                    
                    let batch_refs: Vec<(&[u8], &[u8], &[u8], u64)> = batch
                        .iter()
                        .enumerate()
                        .map(|(i, (h, s, d))| {
                            (h.as_bytes(), s.as_bytes(), d.as_bytes(), start_num + i as u64)
                        })
                        .collect();
                    
                    storage.store_transactions_batch(batch_refs).unwrap();
                    start_num += size as u64;
                });
            }
        );
    }
    
    // Store transactions for retrieval benchmarks
    for i in 0..10000 {
        let tx_hash = format!("tx_hash_{:032}", i);
        let sender = format!("sender_{:034}", i % 100);
        let tx_data = format!("tx_data_{}", i);
        storage.store_transaction(
            tx_hash.as_bytes(),
            sender.as_bytes(),
            tx_data.as_bytes(),
            i
        ).unwrap();
    }
    
    // Benchmark transaction retrieval by hash
    group.throughput(Throughput::Elements(1));
    group.bench_function("get_transaction", |b| {
        let mut tx_num = 0u64;
        b.iter(|| {
            let tx_hash = format!("tx_hash_{:032}", tx_num % 10000);
            let result = storage.get_transaction(black_box(tx_hash.as_bytes())).unwrap();
            tx_num += 1;
            result
        });
    });
    
    // Benchmark getting transactions by sender
    group.bench_function("get_transactions_by_sender", |b| {
        let mut sender_id = 0u64;
        b.iter(|| {
            let sender = format!("sender_{:034}", sender_id % 100);
            let result = storage.get_transactions_by_sender(
                black_box(sender.as_bytes()),
                black_box(0)
            ).unwrap();
            sender_id += 1;
            result
        });
    });
    
    // Benchmark with limit
    group.bench_function("get_transactions_by_sender_limit_10", |b| {
        let mut sender_id = 0u64;
        b.iter(|| {
            let sender = format!("sender_{:034}", sender_id % 100);
            let result = storage.get_transactions_by_sender(
                black_box(sender.as_bytes()),
                black_box(10)
            ).unwrap();
            sender_id += 1;
            result
        });
    });
    
    group.finish();
}

fn bench_state_snapshots(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path()).unwrap();
    
    let mut group = c.benchmark_group("state_snapshots");
    
    // Benchmark snapshot creation with various sizes
    for data_size in [1024, 10240, 102400].iter() {
        group.throughput(Throughput::Bytes(*data_size as u64));
        group.bench_with_input(
            BenchmarkId::new("create_snapshot", data_size),
            data_size,
            |b, &size| {
                let mut height = 0u64;
                let state_root = vec![0u8; 32];
                let accounts_data = vec![0u8; size];
                
                b.iter(|| {
                    storage.create_snapshot(
                        black_box(height),
                        black_box(&state_root),
                        black_box(&accounts_data)
                    ).unwrap();
                    height += 1;
                });
            }
        );
    }
    
    // Store snapshots for retrieval benchmarks
    for i in 0..100 {
        let state_root = vec![i as u8; 32];
        let accounts_data = vec![i as u8; 10240];
        storage.create_snapshot(i * 1000, &state_root, &accounts_data).unwrap();
    }
    
    // Benchmark snapshot retrieval
    group.throughput(Throughput::Elements(1));
    group.bench_function("get_latest_snapshot", |b| {
        b.iter(|| {
            storage.get_latest_snapshot().unwrap()
        });
    });
    
    group.bench_function("get_snapshot", |b| {
        let mut idx = 0u64;
        b.iter(|| {
            let height = (idx % 100) * 1000;
            let result = storage.get_snapshot(black_box(height)).unwrap();
            idx += 1;
            result
        });
    });
    
    group.finish();
}

fn bench_account_operations(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path()).unwrap();
    
    let mut group = c.benchmark_group("account_operations");
    
    // Benchmark account storage
    group.bench_function("store_account", |b| {
        let mut account_id = 0u64;
        b.iter(|| {
            let address = {
                let mut addr = [0u8; 33];
                addr[0..8].copy_from_slice(&account_id.to_le_bytes());
                addr
            };
            let account = Account {
                balance: 1000 + account_id,
                nonce: account_id,
            };
            
            storage.store_account(black_box(&address), black_box(&account)).unwrap();
            account_id += 1;
        });
    });
    
    // Store accounts for retrieval benchmarks
    for i in 0u64..1000 {
        let address = {
            let mut addr = [0u8; 33];
            addr[0..8].copy_from_slice(&i.to_le_bytes());
            addr
        };
        let account = Account {
            balance: 1000 + i,
            nonce: i,
        };
        storage.store_account(&address, &account).unwrap();
    }
    
    // Benchmark account retrieval
    group.bench_function("get_account", |b| {
        let mut account_id = 0u64;
        b.iter(|| {
            let address = {
                let mut addr = [0u8; 33];
                addr[0..8].copy_from_slice(&(account_id % 1000).to_le_bytes());
                addr
            };
            let result = storage.get_account(black_box(&address)).unwrap();
            account_id += 1;
            result
        });
    });
    
    group.finish();
}

fn bench_pruning(c: &mut Criterion) {
    let mut group = c.benchmark_group("pruning");
    group.sample_size(10); // Pruning is expensive, use fewer samples
    
    // Benchmark simple pruning
    for block_count in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("prune_old_blocks", block_count),
            block_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        // Setup: Create fresh database with blocks
                        let temp_dir = TempDir::new().unwrap();
                        let storage = StorageManager::new(temp_dir.path()).unwrap();
                        
                        for i in 0..count {
                            let hash = format!("hash_{:032}", i);
                            let header = format!("header_{}", i);
                            storage.store_header(i, hash.as_bytes(), header.as_bytes()).unwrap();
                        }
                        
                        (storage, temp_dir)
                    },
                    |(storage, _temp_dir)| {
                        // Benchmark: Prune keeping last 50 blocks
                        storage.prune_old_blocks(black_box(50)).unwrap();
                    },
                    criterion::BatchSize::LargeInput
                );
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_block_storage,
    bench_transaction_indexing,
    bench_state_snapshots,
    bench_account_operations,
    bench_pruning
);
criterion_main!(benches);
