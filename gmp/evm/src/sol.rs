use alloy::{primitives::U256, sol, sol_types::SolValue};

use crate::{a_addr, t_addr};

// circle message v1 bytecode doesnt need to be changed so keeping it as const.
pub const CIRCLE_MESSAGE_LIB_BYTECODE: &str = "610106610034600b8282823980515f1a607314602857634e487b7160e01b5f525f60045260245ffd5b305f52607381538281f3fe7300000000000000000000000000000000000000003014608060405260043610603c575f3560e01c80635ced058e14604057806382c947b714606b575b5f80fd5b604e604b366004608f565b90565b6040516001600160a01b0390911681526020015b60405180910390f35b6082607636600460a5565b6001600160a01b031690565b6040519081526020016062565b5f60208284031215609e575f80fd5b5035919050565b5f6020828403121560b4575f80fd5b81356001600160a01b038116811460c9575f80fd5b939250505056fea2646970667358221220c4a4b6631fa0b2b666a9bfb1b09eddaaf35ed46e87ed97746419c81c07fd02cb64736f6c63430008190033";

// Codegen from ABI file to interact with the contract.
sol!(
	#[allow(clippy::too_many_arguments)]
	#[allow(missing_docs)]
	#[sol(rpc)]
	#[derive(Debug)]
	IExecutor,
	"../../analog-gmp/out/IExecutor.sol/IExecutor.json"
);

sol! {
	#[derive(Debug, Default, PartialEq, Eq)]
	struct TssKey {
		uint8 yParity;
		uint256 xCoord;
	}

	#[derive(Debug, Default, PartialEq, Eq)]
	struct TssSignature {
		uint256 e;
		uint256 s;
	}

	#[derive(Debug, Default, PartialEq, Eq)]
	struct GmpMessage {
		bytes32 source;
		uint16 srcNetwork;
		address dest;
		uint16 destNetwork;
		uint64 gasLimit;
		uint64 nonce;
		bytes data;
	}

	#[derive(Debug, Default, PartialEq, Eq)]
	struct Route {
		uint16 networkId;
		uint64 gasLimit;
		uint128 baseFee;
		bytes32 gateway;
		uint256 relativeGasPriceNumerator;
		uint256 relativeGasPriceDenominator;
	}

	#[derive(Debug, Default, PartialEq, Eq)]
	struct Network {
		uint16 id;
		address gateway;
	}


	#[derive(Debug, Default, PartialEq, Eq)]
	struct CCTP {
		/// The attestation (obs: will be provided by the chronicle).
		bytes attestation;
		/// The message bytes emitted by the MessageSent event (must be provided).
		bytes message;
		/// Extra data field used by cctp implementers for custom usage
		bytes extraData;
	}

	contract GatewayProxy {
		constructor(address admin) payable;
	}

	#[derive(Debug, Default, PartialEq, Eq)]
	struct Signature {
		uint256 xCoord;
		uint256 e;
		uint256 s;
	}

	#[derive(Debug)]
	enum GmpStatus {
		NOT_FOUND,
		SUCCESS,
		REVERT,
		INSUFFICIENT_FUNDS,
		PENDING
	}

	struct InboundMessage {
		uint8 version;
		uint64 batchID;
		GatewayOp[] ops;
	}

	struct GatewayOp {
		Command command;
		bytes params;
	}


	#[derive(Debug)]
	enum Command {
		Invalid,
		GMP,
		RegisterShard,
		UnregisterShard,
		SetRoute
	}

	contract Gateway {
		constructor(uint16 network, address proxy) payable;
		function initialize(address admin, TssKey[] memory keys, Network[] calldata networks) external;
		function upgrade(address newImplementation) external payable;
		function execute(TssSignature memory signature, uint256 xCoord, bytes memory message) external;
		function execute(Signature calldata signature, GmpMessage calldata message)
			external
			returns (GmpStatus status, bytes32 result);
		function batchExecute(Signature calldata signature, InboundMessage calldata message) external;
		function admin() external view returns (address);
		function setAdmin(address admin) external payable;
		function shards() external view returns (TssKey[] memory);
		function setShards(TssKey[] calldata publicKeys) external;
		function routes() external view returns (Route[]);
		function setRoute(Route calldata info) external;
		function nonceOf(address account) external view returns (uint64);
		function estimateMessageCost(uint16 networkid, uint256 messageSize, uint256 gasLimit) external view returns (uint256);
		function withdraw(uint256 amount, address recipient, bytes calldata data) external returns (bytes memory output);

		event ShardsRegistered(TssKey[] keys);
		event ShardsUnregistered(TssKey[] keys);
		event GmpCreated(
			bytes32 indexed id,
			bytes32 indexed source,
			address indexed destinationAddress,
			uint16 destinationNetwork,
			uint64 executionGasLimit,
			uint64 gasCost,
			uint64 nonce,
			bytes data
		);
		#[derive(Debug)]
		event GmpExecuted(
			bytes32 indexed id,
			bytes32 indexed source,
			address indexed dest,
			GmpStatus status,
			bytes32 result
		);
		event BatchExecuted(
			uint64 batch,
		);
	}

	contract ZenSwapGmpPlugin {
		function initialize(
			address _gmpGateway,
			address _cctpMessenger,
			address _cctpReceiver,
			address _usdc,
			uint _fee
		) public initializer;


		struct PluginParams {
			// Plugin address on destination chain
			address destPlugin;
			// Extra data recipient (ZenSwap contract)
			address recipient;
			// USDC recipient in case of onReceived fail
			address fallbackRecipient;
			// CCTP destination domain
			uint32 cctpDestinationDomain;
			// GMP destination network id
			uint16 gmpDestNetwork;
			// GMP gas limit
			uint64 gmpGasLimit;
		}
	}

	contract ZenSwap {
		constructor(address _universalRouter, address _permit2) UniswapWrapper(_universalRouter, _permit2);
		struct SwapParams {
			address tokenIn;
			address tokenOut;
			uint256 deadline;
			bytes commands;
			bytes[] inputs;
		}

		function swapSend(
			bytes calldata pluginParams,
			SwapParams calldata sourceParams,
			SwapParams calldata destParams,
			address payable recipient,
			address plugin,
			uint256 amountIn
		) external payable;
	}

	contract ERC20Approval {
		function approve(address spender, uint256 amount) external returns (bool);
	}

	#[derive(Debug, Default, PartialEq, Eq)]
	struct ProxyContext {
		uint8 v;
		bytes32 r;
		bytes32 s;
		address implementation;
	}

	#[derive(Debug, Default, PartialEq, Eq)]
	struct ProxyDigest {
		address proxy;
		address implementation;
	}

	contract GmpTester {
		constructor(address gateway);
		function sendMessage(GmpMessage msg) payable returns (bytes32);
		function estimateMessageCost(uint256 messageSize, uint256 gasLimit) external view returns (uint256);
		event MessageReceived(GmpMessage msg);
	}

	// reference: https://github.com/Analog-Labs/universal-factory/blob/main/src/IUniversalFactory.sol
	interface IUniversalFactory {
		function create2(bytes32 salt, bytes calldata creationCode) external payable returns (address);
		function create2(bytes32 salt, bytes calldata creationCode, bytes calldata arguments, bytes calldata callback)
			external
			payable
			returns (address);
	}

	interface IGmpReceiver {
		function onGmpReceived(bytes32 id, uint128 network, bytes32 source, uint64 nonce, bytes calldata payload)
			external
			payable
			returns (bytes32);
	}
}

pub fn u256(bytes: &[u8]) -> U256 {
	U256::from_be_bytes(<[u8; 32]>::try_from(bytes).unwrap())
}

fn bytes32(u: U256) -> [u8; 32] {
	u.to_be_bytes::<32>()
}

impl From<time_primitives::TssPublicKey> for TssKey {
	fn from(key: time_primitives::TssPublicKey) -> Self {
		Self {
			yParity: key[0],
			xCoord: u256(&key[1..]),
		}
	}
}

impl From<TssKey> for time_primitives::TssPublicKey {
	fn from(key: TssKey) -> Self {
		let mut public = [0; 33];
		public[0] = key.yParity;
		public[1..].copy_from_slice(&bytes32(key.xCoord));
		public
	}
}

impl From<time_primitives::TssSignature> for TssSignature {
	fn from(sig: time_primitives::TssSignature) -> Self {
		Self {
			e: u256(&sig[..32]),
			s: u256(&sig[32..]),
		}
	}
}

impl From<TssSignature> for time_primitives::TssSignature {
	fn from(sig: TssSignature) -> Self {
		let mut bytes = [0; 64];
		bytes[..32].copy_from_slice(&bytes32(sig.e));
		bytes[32..].copy_from_slice(&bytes32(sig.s));
		bytes
	}
}

impl From<time_primitives::Route> for Route {
	fn from(route: time_primitives::Route) -> Self {
		Self {
			networkId: route.network_id,
			gateway: route.gateway.into(),
			relativeGasPriceNumerator: u256(&route.relative_gas_price.0.to_big_endian()),
			relativeGasPriceDenominator: u256(&route.relative_gas_price.1.to_big_endian()),
			gasLimit: route.gas_limit,
			baseFee: route.gmp_base_fee,
		}
	}
}

impl From<Route> for time_primitives::Route {
	fn from(route: Route) -> Self {
		Self {
			network_id: route.networkId,
			gateway: route.gateway.into(),
			relative_gas_price: (
				time_primitives::U256::from_big_endian(&bytes32(route.relativeGasPriceNumerator)),
				time_primitives::U256::from_big_endian(&bytes32(route.relativeGasPriceDenominator)),
			),
			gas_limit: route.gasLimit,
			gmp_base_fee: route.baseFee,
		}
	}
}

impl From<GmpMessage> for time_primitives::GmpMessage {
	fn from(msg: GmpMessage) -> Self {
		Self {
			src_network: msg.srcNetwork,
			dest_network: msg.destNetwork,
			src: msg.source.into(),
			dest: t_addr(msg.dest),
			nonce: msg.nonce,
			gas_limit: msg.gasLimit.into(),
			gas_cost: 0,
			bytes: msg.data.into(),
		}
	}
}

impl From<time_primitives::GmpMessage> for GmpMessage {
	fn from(msg: time_primitives::GmpMessage) -> Self {
		Self {
			srcNetwork: msg.src_network,
			destNetwork: msg.dest_network,
			source: msg.src.into(),
			dest: a_addr(msg.dest),
			nonce: msg.nonce,
			gasLimit: msg.gas_limit as u64,
			data: msg.bytes.into(),
		}
	}
}

impl From<time_primitives::GatewayOp> for GatewayOp {
	fn from(msg: time_primitives::GatewayOp) -> Self {
		match msg {
			time_primitives::GatewayOp::SendMessage(msg) => GatewayOp {
				command: Command::GMP,
				params: Into::<GmpMessage>::into(msg).abi_encode().into(),
			},
			time_primitives::GatewayOp::RegisterShard(shard_id) => GatewayOp {
				command: Command::RegisterShard,
				params: Into::<TssKey>::into(shard_id).abi_encode().into(),
			},
			time_primitives::GatewayOp::UnregisterShard(shard_id) => GatewayOp {
				command: Command::UnregisterShard,
				params: Into::<TssKey>::into(shard_id).abi_encode().into(),
			},
		}
	}
}

impl From<time_primitives::GatewayOp> for IExecutor::GatewayOp {
	fn from(msg: time_primitives::GatewayOp) -> Self {
		match msg {
			time_primitives::GatewayOp::SendMessage(msg) => IExecutor::GatewayOp {
				command: Command::GMP.into(),
				params: Into::<GmpMessage>::into(msg).abi_encode().into(),
			},
			time_primitives::GatewayOp::RegisterShard(shard_id) => IExecutor::GatewayOp {
				command: Command::RegisterShard.into(),
				params: Into::<TssKey>::into(shard_id).abi_encode().into(),
			},
			time_primitives::GatewayOp::UnregisterShard(shard_id) => IExecutor::GatewayOp {
				command: Command::UnregisterShard.into(),
				params: Into::<TssKey>::into(shard_id).abi_encode().into(),
			},
		}
	}
}

impl CCTP {
	pub fn get_version(&self) -> anyhow::Result<u32> {
		if self.message.len() < 4 {
			return Err(anyhow::anyhow!("Message is too short to contain a version field"));
		}
		let version_bytes: [u8; 4] = self.message[0..4]
			.try_into()
			.map_err(|_| anyhow::anyhow!("Failed to extract version bytes"))?;
		let version = u32::from_be_bytes(version_bytes);
		Ok(version)
	}
}
