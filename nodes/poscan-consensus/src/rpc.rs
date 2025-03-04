//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use runtime::{opaque::Block, AccountId, Balance, BlockNumber, Hash, Index};
use jsonrpsee::RpcModule;
use sc_client_api::BlockBackend;
use sc_rpc::dev::{Dev, DevApiServer};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

pub use sc_rpc_api::DenyUnsafe;

/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	// /// A command stream to send authoring commands to manual seal consensus engine
	// pub command_sink:Sender<EngineComman>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>,
	C: BlockBackend<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_contracts_rpc::ContractsRuntimeApi<Block, AccountId, Balance, BlockNumber, Hash>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
{
	use pallet_contracts_rpc::{Contracts, ContractsApiServer};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};
	use crate::mining_rpc::{MiningRpc, PoscanMiningRpcApiServer};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(MiningRpc::new(client.clone()).into_rpc())?;

	// Add a silly RPC that returns constant values
	// io.extend_with(crate::mining_rpc::PoscanMiningRpc::to_delegate(
	// 	crate::mining_rpc::MiningRpc::<C, Block>::new(deps.client.clone()),
	// ));

	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	// `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`

	// Contracts RPC API extension
	module.merge(Contracts::new(client.clone()).into_rpc())?;

	// Dev RPC API extension
	module.merge(Dev::new(client, deny_unsafe).into_rpc())?;

	Ok(module)
}
