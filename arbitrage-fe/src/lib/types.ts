export type Network = 'devnet' | 'mainnet'

export interface NetworkConfig {
  network: Network
  label: string
  apiUrl: string
  gatewayUrl: string
  explorerUrl: string
}

export interface AccountInfo {
  address: string
  ownerAddress?: string
  deployedAt?: number
  developerReward?: string
}

export interface VmQueryResponse {
  returnData?: string[]
  returnCode?: string
  returnMessage?: string
}

export interface TokenBalance {
  identifier: string
  ticker?: string
  decimals?: number
  balance?: string
}

export interface TransactionSummary {
  txHash: string
  timestamp?: number
  timestampMs?: number
  status?: string
  fee?: string
  function?: string
}

export interface Operation {
  identifier?: string
  ticker?: string
  value?: string
  decimals?: number
  sender?: string
  receiver?: string
  action?: string
  type?: string
}

export interface TransactionDetails extends TransactionSummary {
  sender?: string
  receiver?: string
  gasUsed?: number
  gasLimit?: number
  data?: string
  operations?: Operation[]
}

export interface TokenFlow {
  tokenId: string
  ticker: string
  decimals: number
  netRaw: bigint
}

export interface DashboardTransaction {
  txHash: string
  timestamp: number
  status: string
  functionName: string
  gasUsed: number
  gasLimit: number
  feeRaw: string
  tokenFlows: TokenFlow[]
  swapTokens: string[]
  inputAmountRaw?: bigint
  estimatedProfitRaw?: bigint
}

export interface DashboardData {
  account: AccountInfo
  totalTransactionsCount: number
  executeTradesCount: number
  stakedTokenId?: string
  stakedTokenDecimals: number
  ownerWinningsPercentage?: bigint
  devWinningsRaw?: bigint
  contractBalances: TokenBalance[]
  transactions: DashboardTransaction[]
}
