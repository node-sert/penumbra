use std::{any::Any, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::Span;

#[cfg(feature = "metrics")]
use crate::metrics;
use crate::store::multistore::MultistoreConfig;
use crate::{store, StateRead};

mod rocks_wrapper;

pub(crate) use rocks_wrapper::RocksDbSnapshot;

/// A snapshot of the underlying storage at a specific state version, suitable
/// for read-only access by multiple threads, e.g., RPC calls.
///
/// Snapshots are cheap to create and clone.  Internally, they're implemented as
/// a wrapper around a [RocksDB snapshot](https://github.com/facebook/rocksdb/wiki/Snapshot)
/// with a pinned JMT version number for the snapshot.
#[derive(Clone)]
pub struct Snapshot(pub(crate) Arc<Inner>);

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snapshot")
            .field("version", &self.0.version)
            .finish_non_exhaustive()
    }
}

// We don't want to expose the `TreeReader` implementation outside of this crate.
#[derive(Debug)]
pub(crate) struct Inner {
    pub(crate) multistore: MultistoreConfig,
    pub(crate) snapshot: Arc<RocksDbSnapshot>,
    pub(crate) version: jmt::Version,
    // Used to retrieve column family handles.
    pub(crate) db: Arc<rocksdb::DB>,
}

impl Snapshot {
    /// Creates a new `Snapshot` with the given version and substore configs.
    pub(crate) fn new(
        db: Arc<rocksdb::DB>,
        version: jmt::Version,
        multistore: MultistoreConfig,
    ) -> Self {
        Self(Arc::new(Inner {
            snapshot: Arc::new(RocksDbSnapshot::new(db.clone())),
            version,
            db,
            multistore,
        }))
    }

    pub fn version(&self) -> jmt::Version {
        self.0.version
    }

    /// Returns some value corresponding to the key, along with an ICS23 existence proof
    /// up to the current JMT root hash. If the key is not present, returns `None` and a
    /// non-existence proof.
    pub async fn get_with_proof(
        &self,
        key: Vec<u8>,
    ) -> Result<(Option<Vec<u8>>, ics23::CommitmentProof)> {
        let span = tracing::Span::current();

        let (_, substore_config) = self.0.multistore.route_key_bytes(&key);
        let substore = store::substore::SubstoreSnapshot {
            config: substore_config,
            rocksdb_snapshot: self.0.snapshot.clone(),
            version: self.version(),
            db: self.0.db.clone(),
        };
        tokio::task::Builder::new()
            .name("Snapshot::get_with_proof")
            .spawn_blocking(move || span.in_scope(|| substore.get_with_proof(key)))?
            .await?
    }

    /// Returns some value corresponding to the key, along with an ICS23 existence proof
    /// up to the current JMT root hash. If the key is not present, returns `None` and a
    /// non-existence proof.
    pub async fn _get_with_proof_new_api(
        &self,
        key: Vec<u8>,
    ) -> Result<(Option<Vec<u8>>, Vec<ics23::CommitmentProof>)> {
        let span = tracing::Span::current();

        let (_, substore_config) = self.0.multistore.route_key_bytes(&key);
        let substore = store::substore::SubstoreSnapshot {
            config: substore_config.clone(),
            rocksdb_snapshot: self.0.snapshot.clone(),
            version: self.version(),
            db: self.0.db.clone(),
        };

        let prefix = substore_config.prefix.clone();
        let substore_root_hash = substore.root_hash()?;
        let (opt_value, substore_proof) = tokio::task::Builder::new()
            .name("Snapshot::get_with_proof")
            .spawn_blocking(move || span.in_scope(|| substore.get_with_proof(key)))?
            .await??;

        if !prefix.is_empty() {
            /*
               TODO(erwan): temporary artifact, this should of course be part of a proof spec.
            */
            let tmp_leaf_op = ics23::LeafOp {
                prefix: vec![],
                hash: ics23::HashOp::Sha256.into(),
                length: ics23::LengthOp::NoPrefix.into(),
                prehash_key: ics23::HashOp::NoHash.into(),
                prehash_value: ics23::HashOp::NoHash.into(),
            };
            /*
               Remove this.
            */

            let root_proof = ics23::CommitmentProof {
                proof: Some(ics23::commitment_proof::Proof::Exist(
                    ics23::ExistenceProof {
                        key: prefix.into(),
                        value: substore_root_hash.0.to_vec(),
                        path: vec![],
                        leaf: Some(tmp_leaf_op),
                    },
                )),
            };

            Ok((opt_value, vec![substore_proof, root_proof]))
        } else {
            /* otherwise, we're dealing with an entry that belongs to the main store.*/
            Ok((opt_value, vec![substore_proof]))
        }
    }

    /// Returns the root hash of this `State`.
    ///
    /// If the `State` is empty, the all-zeros hash will be returned as a placeholder value.
    ///
    /// This method may only be used on a clean [`State`] fork, and will error
    /// if [`is_dirty`] returns `true`.
    pub async fn root_hash_for(&self, prefix: &str) -> Result<crate::RootHash> {
        let span = tracing::Span::current();

        let (_, substore_config) = self.0.multistore.route_key_str(&prefix);
        let substore = store::substore::SubstoreSnapshot {
            config: substore_config,
            rocksdb_snapshot: self.0.snapshot.clone(),
            version: self.version(),
            db: self.0.db.clone(),
        };

        tokio::task::Builder::new()
            .name("Snapshot::root_hash_for")
            .spawn_blocking(move || span.in_scope(|| substore.root_hash()))?
            .await?
    }

    pub async fn root_hash(&self) -> Result<crate::RootHash> {
        self.root_hash_for("").await
    }
}

#[async_trait]
impl StateRead for Snapshot {
    type GetRawFut = crate::future::SnapshotFuture;
    type PrefixRawStream =
        tokio_stream::wrappers::ReceiverStream<anyhow::Result<(String, Vec<u8>)>>;
    type PrefixKeysStream = tokio_stream::wrappers::ReceiverStream<anyhow::Result<String>>;
    type NonconsensusPrefixRawStream =
        tokio_stream::wrappers::ReceiverStream<anyhow::Result<(Vec<u8>, Vec<u8>)>>;
    type NonconsensusRangeRawStream =
        tokio_stream::wrappers::ReceiverStream<anyhow::Result<(Vec<u8>, Vec<u8>)>>;

    /// Fetch a key from the JMT column family.
    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        let span = Span::current();
        let (key, substore_config) = self.0.multistore.route_key_str(key);

        let substore = store::substore::SubstoreSnapshot {
            config: substore_config,
            rocksdb_snapshot: self.0.snapshot.clone(),
            version: self.version(),
            db: self.0.db.clone(),
        };
        let key_hash = jmt::KeyHash::with::<sha2::Sha256>(key);

        crate::future::SnapshotFuture(
            tokio::task::Builder::new()
                .name("Snapshot::get_raw")
                .spawn_blocking(move || {
                    span.in_scope(|| {
                        let _start = std::time::Instant::now();
                        let rsp = substore.get_jmt(key_hash);
                        #[cfg(feature = "metrics")]
                        metrics::histogram!(metrics::STORAGE_GET_RAW_DURATION, _start.elapsed());
                        rsp
                    })
                })
                .expect("spawning threads is possible"),
        )
    }

    fn nonverifiable_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        let span = Span::current();
        let inner = self.0.clone();
        let snapshot = self.clone();
        let (key, substore_config) = inner.multistore.route_key_bytes(key);

        let substore = store::substore::SubstoreSnapshot {
            config: substore_config,
            rocksdb_snapshot: self.0.snapshot.clone(),
            version: self.version(),
            db: self.0.db.clone(),
        };
        let key: Vec<u8> = key.to_vec();
        crate::future::SnapshotFuture(
            tokio::task::Builder::new()
                .name("Snapshot::nonverifiable_get_raw")
                .spawn_blocking(move || {
                    span.in_scope(|| {
                        let _start = std::time::Instant::now();

                        let cf_nonverifiable = substore.config.cf_nonverifiable(&snapshot.0.db);

                        let rsp = inner
                            .snapshot
                            .get_cf(cf_nonverifiable, key)
                            .map_err(Into::into);
                        #[cfg(feature = "metrics")]
                        metrics::histogram!(
                            metrics::STORAGE_NONCONSENSUS_GET_RAW_DURATION,
                            _start.elapsed()
                        );
                        rsp
                    })
                })
                .expect("spawning threads is possible"),
        )
    }

    fn prefix_raw(&self, prefix: &str) -> Self::PrefixRawStream {
        let span = Span::current();

        let mut options = rocksdb::ReadOptions::default();
        options.set_iterate_range(rocksdb::PrefixRange(prefix.as_bytes()));
        let mode = rocksdb::IteratorMode::Start;

        let (_, substore_config) = self.0.multistore.route_key_bytes(prefix.as_bytes());
        let snapshot = self.clone();

        let (tx_prefix_item, rx_prefix_query) = mpsc::channel(10);

        // Since the JMT keys are hashed, we can't use a prefix iterator directly.
        // We need to first prefix range the key preimages column family, then use the hashed matches to fetch the values
        // from the JMT column family.
        tokio::task::Builder::new()
            .name("Snapshot::prefix_raw")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let substore = store::substore::SubstoreSnapshot {
                        config: substore_config.clone(),
                        rocksdb_snapshot: snapshot.0.snapshot.clone(),
                        version: snapshot.version(),
                        db: snapshot.0.db.clone(),
                    };
                    let cf_jmt_keys = substore.config.cf_jmt_keys(&snapshot.0.db);
                    let jmt_keys_iterator =
                        snapshot
                            .0
                            .snapshot
                            .iterator_cf_opt(cf_jmt_keys, options, mode);

                    for tuple in jmt_keys_iterator {
                        // For each key that matches the prefix, fetch the value from the JMT column family.
                        let (key_preimage, _) = tuple?;

                        let k = std::str::from_utf8(key_preimage.as_ref())
                            .expect("saved jmt keys are utf-8 strings")
                            .to_string();

                        let key_hash = jmt::KeyHash::with::<sha2::Sha256>(k.as_bytes());

                        let substore = store::substore::SubstoreSnapshot {
                            config: substore_config.clone(),
                            rocksdb_snapshot: snapshot.0.snapshot.clone(),
                            version: snapshot.version(),
                            db: snapshot.0.db.clone(),
                        };
                        let v = substore
                            .get_jmt(key_hash)?
                            .expect("keys in jmt_keys should have a corresponding value in jmt");
                        tracing::debug!(%k, "prefix_raw");

                        tx_prefix_item.blocking_send(Ok((k, v)))?;
                    }
                    anyhow::Ok(())
                })
            })
            .expect("should be able to spawn_blocking");

        tokio_stream::wrappers::ReceiverStream::new(rx_prefix_query)
    }

    // NOTE: this implementation is almost the same as the above, but without
    // fetching the values. not totally clear if this could be combined, or if that would
    // be better overall.
    fn prefix_keys(&self, prefix: &str) -> Self::PrefixKeysStream {
        let span = Span::current();
        let snapshot = self.clone();

        let (prefix, substore_config) = self.0.multistore.route_key_str(prefix);
        let mut options = rocksdb::ReadOptions::default();
        options.set_iterate_range(rocksdb::PrefixRange(prefix.as_bytes()));
        let mode = rocksdb::IteratorMode::Start;

        let substore = store::substore::SubstoreSnapshot {
            config: substore_config,
            rocksdb_snapshot: self.0.snapshot.clone(),
            version: self.version(),
            db: self.0.db.clone(),
        };
        let (tx, rx) = mpsc::channel(10);
        tokio::task::Builder::new()
            .name("Snapshot::prefix_keys")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let cf_keys = substore.config.cf_jmt_keys(&snapshot.0.db);
                    let iter = snapshot.0.snapshot.iterator_cf_opt(cf_keys, options, mode);
                    for i in iter {
                        // For each key that matches the prefix, fetch the value from the JMT column family.
                        let (key_preimage, _key_hash) = i?;
                        let k = std::str::from_utf8(key_preimage.as_ref())
                            .expect("saved jmt keys are utf-8 strings")
                            .to_string();
                        tx.blocking_send(Ok(k))?;
                    }
                    anyhow::Ok(())
                })
            })
            .expect("should be able to spawn_blocking");

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }

    fn nonverifiable_prefix_raw(&self, prefix: &[u8]) -> Self::NonconsensusPrefixRawStream {
        let span = Span::current();
        let snapshot = self.clone();

        let (prefix, substore_config) = self.0.multistore.route_key_bytes(prefix);
        let mut options = rocksdb::ReadOptions::default();
        options.set_iterate_range(rocksdb::PrefixRange(prefix));
        let mode = rocksdb::IteratorMode::Start;

        let substore = store::substore::SubstoreSnapshot {
            config: substore_config,
            rocksdb_snapshot: self.0.snapshot.clone(),
            version: self.version(),
            db: self.0.db.clone(),
        };

        let (tx_prefix_query, rx_prefix_query) = mpsc::channel(10);

        // Here we're operating on the nonverifiable data, which is a raw k/v store,
        // so we just iterate over the keys.
        tokio::task::Builder::new()
            .name("Snapshot::nonverifiable_prefix_raw")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let cf_nonverifiable = substore.config.cf_nonverifiable(&snapshot.0.db);
                    let iter = snapshot
                        .0
                        .snapshot
                        .iterator_cf_opt(cf_nonverifiable, options, mode);
                    for i in iter {
                        let (key, value) = i?;
                        tx_prefix_query.blocking_send(Ok((key.into(), value.into())))?;
                    }
                    anyhow::Ok(())
                })
            })
            .expect("should be able to spawn_blocking");

        tokio_stream::wrappers::ReceiverStream::new(rx_prefix_query)
    }

    fn nonverifiable_range_raw(
        &self,
        prefix: Option<&[u8]>,
        range: impl std::ops::RangeBounds<Vec<u8>>,
    ) -> anyhow::Result<Self::NonconsensusRangeRawStream> {
        let span = Span::current();
        let snapshot = self.clone();

        let (prefix, substore_config) = self
            .0
            .multistore
            .route_key_bytes(prefix.unwrap_or_default());
        let (_range, (start, end)) = crate::utils::convert_bounds(range)?;
        let mut options = rocksdb::ReadOptions::default();

        let (start, end) = (start.unwrap_or_default(), end.unwrap_or_default());
        let end_is_empty = end.is_empty();

        let mut prefix_start = Vec::with_capacity(prefix.len() + start.len());
        let mut prefix_end = Vec::with_capacity(prefix.len() + end.len());

        prefix_start.extend(prefix);
        prefix_start.extend(start);
        prefix_end.extend(prefix);
        prefix_end.extend(end);

        tracing::debug!(
            ?prefix_start,
            ?prefix_end,
            ?prefix,
            "nonverifiable_range_raw"
        );

        options.set_iterate_lower_bound(prefix_start);

        // Our range queries implementation relies on forward iteration, which
        // means that if the upper key is unbounded and a prefix has been set
        // we cannot set the upper bound to the prefix. This is because the
        // prefix is used as a lower bound for the iterator, and the upper bound
        // is used to stop the iteration.
        // If we set the upper bound to the prefix, we would get a range consisting of:
        // ```
        // "compactblock/001" to "compactblock/"
        // ```
        // which would not return anything.
        if !end_is_empty {
            options.set_iterate_upper_bound(prefix_end);
        }

        let mode = rocksdb::IteratorMode::Start;
        let prefix = prefix.to_vec();

        let substore = store::substore::SubstoreSnapshot {
            config: substore_config,
            rocksdb_snapshot: self.0.snapshot.clone(),
            version: self.version(),
            db: self.0.db.clone(),
        };
        let (tx, rx) = mpsc::channel::<Result<(Vec<u8>, Vec<u8>)>>(10);
        tokio::task::Builder::new()
            .name("Snapshot::nonverifiable_range_raw")
            .spawn_blocking(move || {
                span.in_scope(|| {
                    let cf_nonverifiable = substore.config.cf_nonverifiable(&snapshot.0.db);
                    let iter = snapshot
                        .0
                        .snapshot
                        .iterator_cf_opt(cf_nonverifiable, options, mode);
                    for i in iter {
                        let (key, value) = i?;

                        // This is a bit of a hack, but RocksDB doesn't let us express the "prefixed range-queries",
                        // that we want to support. In particular, we want to be able to do a prefix query that starts
                        // at a particular key, and does not have an upper bound. Since we can't create an iterator that
                        // cover this range, we have to filter out the keys that don't match the prefix.
                        if !prefix.is_empty() && !key.starts_with(&prefix) {
                            break;
                        }
                        tx.blocking_send(Ok((key.into(), value.into())))?;
                    }
                    Ok::<(), anyhow::Error>(())
                })
            })
            .expect("should be able to spawn_blocking");

        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    fn object_get<T: Any + Send + Sync + Clone>(&self, _key: &str) -> Option<T> {
        // No-op -- this will never be called internally, and `Snapshot` is not exposed in public API
        None
    }

    fn object_type(&self, _key: &str) -> Option<std::any::TypeId> {
        // No-op -- this will never be called internally, and `Snapshot` is not exposed in public API
        None
    }
}
