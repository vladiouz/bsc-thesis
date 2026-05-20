import { NETWORKS } from './config'
import type {
  AccountInfo,
  DashboardData,
  DashboardTransaction,
  Network,
  TokenBalance,
  TransactionDetails,
  TransactionSummary,
  VmQueryResponse,
} from './types'
import {
  base64ToBigInt,
  base64ToText,
  buildTokenFlows,
  parseExecuteTradesData,
} from './utils'

const TX_LIMIT = 25
const RELEVANT_FUNCTIONS = new Set([
  'executeTrades',
  'stake',
  'unstake',
  'claimWinnings',
  'restakeWinnings',
  'withdrawDevWinnings',
  'setOwnerWinningsPercentage',
  'pause',
  'unpause',
])

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url)

  if (!response.ok) {
    throw new Error(`Request failed (${response.status}) for ${url}`)
  }

  return (await response.json()) as T
}

async function postJson<T>(url: string, body: unknown): Promise<T> {
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
  })

  if (!response.ok) {
    throw new Error(`Request failed (${response.status}) for ${url}`)
  }

  return (await response.json()) as T
}

async function queryVmValue(
  gatewayUrl: string,
  contractAddress: string,
  funcName: string,
): Promise<VmQueryResponse> {
  const response = await postJson<{
    data?: { data?: VmQueryResponse }
  }>(`${gatewayUrl}/vm-values/query`, {
    scAddress: contractAddress,
    funcName,
    args: [],
  })

  return response.data?.data ?? {}
}

async function getContractState(
  network: Network,
  contractAddress: string,
): Promise<{
  stakedTokenId?: string
  devWinningsRaw?: bigint
  ownerWinningsPercentage?: bigint
}> {
  const config = NETWORKS[network]

  const [stakedTokenResult, devWinningsResult, ownerPctResult] = await Promise.all([
    queryVmValue(config.gatewayUrl, contractAddress, 'getStakedTokenId'),
    queryVmValue(config.gatewayUrl, contractAddress, 'getDevWinnings'),
    queryVmValue(config.gatewayUrl, contractAddress, 'getOwnerWinningsPercentage'),
  ])

  return {
    stakedTokenId: base64ToText(stakedTokenResult.returnData?.[0]),
    devWinningsRaw: base64ToBigInt(devWinningsResult.returnData?.[0]),
    ownerWinningsPercentage: base64ToBigInt(ownerPctResult.returnData?.[0]),
  }
}

async function getAccountInfo(network: Network, contractAddress: string): Promise<AccountInfo> {
  const config = NETWORKS[network]

  return fetchJson<AccountInfo>(`${config.apiUrl}/accounts/${contractAddress}`)
}

async function getContractBalances(
  network: Network,
  contractAddress: string,
): Promise<TokenBalance[]> {
  const config = NETWORKS[network]

  return fetchJson<TokenBalance[]>(
    `${config.apiUrl}/accounts/${contractAddress}/tokens?size=50`,
  )
}

async function getTokenDecimals(network: Network, tokenId: string): Promise<number> {
  const config = NETWORKS[network]

  try {
    const token = await fetchJson<{ decimals?: number }>(`${config.apiUrl}/tokens/${tokenId}`)
    return token.decimals ?? 0
  } catch {
    return 0
  }
}

async function getTransactions(
  network: Network,
  contractAddress: string,
): Promise<TransactionSummary[]> {
  const config = NETWORKS[network]

  const txs = await fetchJson<TransactionSummary[]>(
    `${config.apiUrl}/accounts/${contractAddress}/transactions?size=${TX_LIMIT}&status=success`,
  )

  return txs.filter((tx) => RELEVANT_FUNCTIONS.has(tx.function ?? ''))
}

async function getTransactionsCount(
  network: Network,
  contractAddress: string,
  functionName?: string,
): Promise<number> {
  const config = NETWORKS[network]
  const params = new URLSearchParams({ status: 'success' })

  if (functionName) {
    params.set('function', functionName)
  }

  return fetchJson<number>(
    `${config.apiUrl}/accounts/${contractAddress}/transactions/count?${params.toString()}`,
  )
}

async function getTransactionDetails(
  network: Network,
  txHash: string,
): Promise<TransactionDetails> {
  const config = NETWORKS[network]

  return fetchJson<TransactionDetails>(
    `${config.apiUrl}/transactions/${txHash}?withResults=true&withLogs=true&withOperations=true`,
  )
}

export async function getDashboardData(
  network: Network,
  contractAddress: string,
): Promise<DashboardData> {
  const [state, account, balances, txs, totalTransactionsCount, executeTradesCount] =
    await Promise.all([
    getContractState(network, contractAddress),
    getAccountInfo(network, contractAddress),
    getContractBalances(network, contractAddress),
    getTransactions(network, contractAddress),
    getTransactionsCount(network, contractAddress),
    getTransactionsCount(network, contractAddress, 'executeTrades'),
  ])

  const txDetails = await Promise.all(
    txs.map((tx) => getTransactionDetails(network, tx.txHash)),
  )

  const stakedTokenDecimals = state.stakedTokenId
    ? await getTokenDecimals(network, state.stakedTokenId)
    : 0

  const transactions: DashboardTransaction[] = txDetails.map((tx) => {
    const parsed = parseExecuteTradesData(tx.data)
    const tokenFlows = buildTokenFlows(tx.operations, contractAddress)

    const estimatedProfitRaw = state.stakedTokenId
      ? tokenFlows.find((flow) => flow.tokenId === state.stakedTokenId)?.netRaw
      : undefined

    return {
      txHash: tx.txHash,
      timestamp: tx.timestamp ?? 0,
      status: tx.status ?? 'unknown',
      functionName: tx.function ?? 'unknown',
      gasUsed: tx.gasUsed ?? 0,
      gasLimit: tx.gasLimit ?? 0,
      feeRaw: tx.fee ?? '0',
      tokenFlows,
      swapTokens: parsed.swapTokens,
      inputAmountRaw: parsed.inputAmountRaw,
      estimatedProfitRaw,
    }
  })

  return {
    account,
    totalTransactionsCount,
    executeTradesCount,
    stakedTokenId: state.stakedTokenId,
    stakedTokenDecimals,
    ownerWinningsPercentage: state.ownerWinningsPercentage,
    devWinningsRaw: state.devWinningsRaw,
    contractBalances: balances,
    transactions,
  }
}
