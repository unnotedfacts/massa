use crate::export_active_block::{
    ExportActiveBlock, ExportActiveBlockDeserializer, ExportActiveBlockSerializer,
};
use massa_hash::HashDeserializer;
use massa_models::{
    clique::{Clique, CliqueDeserializer, CliqueSerializer},
    prehash::{Map, Set},
    BlockId,
};
use massa_serialization::{
    Deserializer, SerializeError, Serializer, U32VarIntDeserializer, U32VarIntSerializer,
    U64VarIntDeserializer, U64VarIntSerializer,
};
use nom::{error::context, multi::length_count, sequence::tuple, IResult, Parser};
use nom::{
    error::{ContextError, ParseError},
    multi::count,
};
use serde::{Deserialize, Serialize};
use std::ops::Bound::Included;

/// Bootstrap graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapableGraph {
    /// list of final blocks
    pub final_blocks: Vec<ExportActiveBlock>,
}

/// Basic serializer for `BootstrapableGraph`
#[derive(Default)]
pub struct BootstrapableGraphSerializer {
    block_count_serializer: U32VarIntSerializer,
    export_active_block_serializer: ExportActiveBlockSerializer,
}

impl BootstrapableGraphSerializer {
    /// Creates a `BootstrapableGraphSerializer`
    pub fn new() -> Self {
        Self {
            block_count_serializer: U32VarIntSerializer::new(),
            export_active_block_serializer: ExportActiveBlockSerializer::new(),
        }
    }
}

impl Serializer<BootstrapableGraph> for BootstrapableGraphSerializer {
    /// ## Example
    /// ```rust
    /// use massa_graph::{BootstrapableGraph, BootstrapableGraphSerializer, ledger::ConsensusLedgerSubset};
    /// use massa_serialization::Serializer;
    /// use massa_hash::Hash;
    /// use massa_models::{prehash::Map, BlockId, constants::THREAD_COUNT};
    /// let mut bootstrapable_graph = BootstrapableGraph {
    ///   active_blocks: Map::default(),
    ///   best_parents: Vec::new(),
    ///   latest_final_blocks_periods: Vec::new(),
    ///   gi_head: Map::default(),
    ///   max_cliques: Vec::new(),
    ///   ledger: ConsensusLedgerSubset::default(),
    /// };
    /// for i in 0..THREAD_COUNT {
    ///   bootstrapable_graph.best_parents.push((BlockId(Hash::compute_from("nvqFJ38Gixhv5Pf3QaJofXa2kW37bHtnk2QKaA6ZRycJLv8FH".as_bytes())), 7));
    ///   bootstrapable_graph.latest_final_blocks_periods.push((BlockId(Hash::compute_from("nvqFJ38Gixhv5Pf3QaJofXa2kW37bHtnk2QKaA6ZRycJLv8FH".as_bytes())), 10));
    /// }
    /// let mut buffer = Vec::new();
    /// BootstrapableGraphSerializer::new().serialize(&bootstrapable_graph, &mut buffer).unwrap();
    /// ```
    fn serialize(
        &self,
        value: &BootstrapableGraph,
        buffer: &mut Vec<u8>,
    ) -> Result<(), SerializeError> {
        // block count
        self.block_count_serializer.serialize(
            &value
                .final_blocks
                .len()
                .try_into()
                .map_err(|_| SerializeError::NumberTooBig("Too many final blocks".to_string()))?,
            buffer,
        )?;

        // final blocks
        for export_active_block in &value.final_blocks {
            self.export_active_block_serializer
                .serialize(export_active_block, buffer)?;
        }

        Ok(())
    }
}

/// Basic deserializer for `BootstrapableGraph`
pub struct BootstrapableGraphDeserializer {
    block_count_deserializer: U32VarIntDeserializer,
    export_active_block_deserializer: ExportActiveBlockDeserializer,
}

impl BootstrapableGraphDeserializer {
    /// Creates a `BootstrapableGraphDeserializer`
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        thread_count: u8,
        endorsement_count: u32,
        max_bootstrap_blocks: u32,
        max_datastore_value_length: u64,
        max_function_name_length: u16,
        max_parameters_size: u32,
        max_operations_per_block: u32,
    ) -> Self {
        Self {
            block_count_deserializer: U32VarIntDeserializer::new(
                Included(0),
                Included(max_bootstrap_blocks),
            ),
            export_active_block_deserializer: ExportActiveBlockDeserializer::new(
                thread_count,
                endorsement_count,
                max_operations_per_block,
                max_datastore_value_length,
                max_function_name_length,
                max_parameters_size,
            ),
        }
    }
}

impl Deserializer<BootstrapableGraph> for BootstrapableGraphDeserializer {
    /// ## Example
    /// ```rust
    /// use massa_graph::{BootstrapableGraph, BootstrapableGraphDeserializer, BootstrapableGraphSerializer, ledger::ConsensusLedgerSubset};
    /// use massa_serialization::{Deserializer, Serializer, DeserializeError};
    /// use massa_hash::Hash;
    /// use massa_models::{prehash::Map, BlockId, constants::THREAD_COUNT};
    /// let mut bootstrapable_graph = BootstrapableGraph {
    ///   active_blocks: Map::default(),
    ///   best_parents: Vec::new(),
    ///   latest_final_blocks_periods: Vec::new(),
    ///   gi_head: Map::default(),
    ///   max_cliques: Vec::new(),
    ///   ledger: ConsensusLedgerSubset::default(),
    /// };
    /// for i in 0..32 {
    ///   bootstrapable_graph.best_parents.push((BlockId(Hash::compute_from("nvqFJ38Gixhv5Pf3QaJofXa2kW37bHtnk2QKaA6ZRycJLv8FH".as_bytes())), 7));
    ///   bootstrapable_graph.latest_final_blocks_periods.push((BlockId(Hash::compute_from("nvqFJ38Gixhv5Pf3QaJofXa2kW37bHtnk2QKaA6ZRycJLv8FH".as_bytes())), 10));
    /// }
    /// let mut buffer = Vec::new();
    /// BootstrapableGraphSerializer::new().serialize(&bootstrapable_graph, &mut buffer).unwrap();
    /// let (rest, bootstrapable_graph_deserialized) = BootstrapableGraphDeserializer::new(32, 9, 10, 10, 100, 1000, 1000, 1000, 10000, 10000).deserialize::<DeserializeError>(&buffer).unwrap();
    /// let mut buffer2 = Vec::new();
    /// BootstrapableGraphSerializer::new().serialize(&bootstrapable_graph_deserialized, &mut buffer2).unwrap();
    /// assert_eq!(buffer, buffer2);
    /// assert_eq!(rest.len(), 0);
    /// ```
    fn deserialize<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        &self,
        buffer: &'a [u8],
    ) -> IResult<&'a [u8], BootstrapableGraph, E> {
        context(
            "Failed BootstrapableGraph deserialization",
            tuple((context(
                "Failed active_blocks deserialization",
                length_count(
                    context("Failed final block count deserialization", |input| {
                        self.block_count_deserializer.deserialize(input)
                    }),
                    context("Failed export_active_block deserialization", |input| {
                        self.export_active_block_deserializer.deserialize(input)
                    }),
                ),
            ),)),
        )
        .map(|(final_blocks,)| BootstrapableGraph { final_blocks })
        .parse(buffer)
    }
}
