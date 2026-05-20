import { useEffect, useMemo, useState } from 'react'
import './App.css'
import { getDashboardData } from './lib/api'
import { getContractAddress, getNetworkFromUrl, NETWORKS } from './lib/config'
import type { DashboardData, Network } from './lib/types'
import {
  formatEgldFromWei,
  formatTokenAmount,
  shortAddress,
  unixToLocaleDate,
} from './lib/utils'

function getInitialNetwork(): Network {
  return getNetworkFromUrl(window.location.search)
}

function getInitialContract(network: Network): string {
  return getContractAddress(network, window.location.search)
}

function updateUrl(network: Network, contractAddress: string): void {
  const params = new URLSearchParams(window.location.search)
  params.set('network', network)

  if (params.get('contract')) {
    params.set('contract', contractAddress)
  }

  const next = `${window.location.pathname}?${params.toString()}`
  window.history.replaceState(null, '', next)
}

function App() {
  const [network, setNetwork] = useState<Network>(() => getInitialNetwork())
  const [contractAddress, setContractAddress] = useState<string>(() =>
    getInitialContract(getInitialNetwork()),
  )
  const [loading, setLoading] = useState<boolean>(true)
  const [error, setError] = useState<string>('')
  const [data, setData] = useState<DashboardData | null>(null)

  useEffect(() => {
    const nextAddress = getContractAddress(network, window.location.search)
    setContractAddress(nextAddress)
    updateUrl(network, nextAddress)
  }, [network])

  useEffect(() => {
    if (!contractAddress) {
      setError('No contract address configured for this network.')
      setLoading(false)
      setData(null)
      return
    }

    let cancelled = false

    setLoading(true)
    setError('')

    getDashboardData(network, contractAddress)
      .then((result) => {
        if (cancelled) {
          return
        }

        setData(result)
      })
      .catch((err) => {
        if (cancelled) {
          return
        }

        setError(err instanceof Error ? err.message : 'Failed to fetch dashboard data')
        setData(null)
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false)
        }
      })

    return () => {
      cancelled = true
    }
  }, [network, contractAddress])

  const computed = useMemo(() => {
    if (!data) {
      return {
        totalFeesRaw: 0n,
        totalProfitRaw: 0n,
      }
    }

    let totalFeesRaw = 0n
    let totalProfitRaw = 0n

    for (const tx of data.transactions) {
      totalFeesRaw += BigInt(tx.feeRaw)

      if (tx.functionName === 'executeTrades') {
        if (typeof tx.estimatedProfitRaw === 'bigint') {
          totalProfitRaw += tx.estimatedProfitRaw
        }
      }
    }

    return {
      totalFeesRaw,
      totalProfitRaw,
    }
  }, [data])

  const stakedTokenTicker = useMemo(() => {
    if (!data?.stakedTokenId) {
      return 'token'
    }

    const balance = data.contractBalances.find(
      (token) => token.identifier === data.stakedTokenId,
    )

    return balance?.ticker || data.stakedTokenId
  }, [data])

  return (
    <main className="layout">
      <header className="topbar">
        <div>
          <p className="eyebrow">Arbitrage Dashboard</p>
          <h1>Smart Contract Overview</h1>
        </div>
        <label className="network-select" htmlFor="network-select">
          Network
          <select
            id="network-select"
            value={network}
            onChange={(event) => setNetwork(event.target.value as Network)}
          >
            <option value="devnet">Devnet</option>
            <option value="mainnet">Mainnet</option>
          </select>
        </label>
      </header>

      <section className="meta-card">
        <p>
          Contract: <code>{contractAddress || 'N/A'}</code>
        </p>
        <p>
          Explorer:{' '}
          <a
            href={`${NETWORKS[network].explorerUrl}/accounts/${contractAddress}`}
            target="_blank"
            rel="noreferrer"
          >
            Open account
          </a>
        </p>
      </section>

      {loading ? <p className="state">Loading dashboard data...</p> : null}
      {error ? <p className="state state-error">{error}</p> : null}

      {data ? (
        <>
          <section className="summary-grid">
            <article className="summary-card">
              <h2>Total Transactions (All Time)</h2>
              <p>{data.totalTransactionsCount}</p>
            </article>
            <article className="summary-card">
              <h2>executeTrades Calls (All Time)</h2>
              <p>{data.executeTradesCount}</p>
            </article>
            <article className="summary-card">
              <h2>Estimated Profit (Loaded Rows)</h2>
              <p>
                {formatTokenAmount(
                  computed.totalProfitRaw,
                  data.stakedTokenDecimals,
                  6,
                )}{' '}
                {stakedTokenTicker}
              </p>
            </article>
            <article className="summary-card">
              <h2>Accumulated Dev Winnings</h2>
              <p>
                {formatTokenAmount(
                  data.devWinningsRaw ?? 0n,
                  data.stakedTokenDecimals,
                  6,
                )}{' '}
                {stakedTokenTicker}
              </p>
            </article>
            <article className="summary-card">
              <h2>Owner Winnings %</h2>
              <p>{(data.ownerWinningsPercentage ?? 0n).toString()}%</p>
            </article>
            <article className="summary-card">
              <h2>Gas Fees (Loaded Rows)</h2>
              <p>{formatEgldFromWei(computed.totalFeesRaw.toString())} EGLD</p>
            </article>
          </section>

          <section className="meta-grid">
            <article className="panel">
              <h3>Contract Metadata</h3>
              <p>Owner: {shortAddress(data.account.ownerAddress ?? 'N/A')}</p>
              <p>
                Deployed:{' '}
                {data.account.deployedAt
                  ? unixToLocaleDate(data.account.deployedAt)
                  : 'N/A'}
              </p>
              <p>
                Staked Token: <code>{data.stakedTokenId ?? 'N/A'}</code>
              </p>
            </article>

            <article className="panel">
              <h3>Token Balances</h3>
              <ul className="token-list">
                {data.contractBalances.slice(0, 8).map((token) => (
                  <li key={token.identifier}>
                    <span>{token.ticker ?? token.identifier}</span>
                    <span>
                      {formatTokenAmount(
                        BigInt(token.balance ?? '0'),
                        token.decimals ?? 0,
                        4,
                      )}
                    </span>
                  </li>
                ))}
              </ul>
            </article>
          </section>

          <section className="table-panel">
            <h3>Latest Transactions ({data.transactions.length} loaded)</h3>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>Time</th>
                    <th>Function</th>
                    <th>Estimated Profit</th>
                    <th>Token Flow</th>
                    <th>Swap Tokens</th>
                    <th>Fee</th>
                    <th>Tx</th>
                  </tr>
                </thead>
                <tbody>
                  {data.transactions.map((tx) => (
                    <tr key={tx.txHash}>
                      <td>{unixToLocaleDate(tx.timestamp)}</td>
                      <td>{tx.functionName}</td>
                      <td>
                        {typeof tx.estimatedProfitRaw === 'bigint'
                          ? `${formatTokenAmount(tx.estimatedProfitRaw, data.stakedTokenDecimals, 6)} ${stakedTokenTicker}`
                          : 'N/A'}
                      </td>
                      <td>
                        {tx.tokenFlows.length > 0
                          ? tx.tokenFlows
                              .map(
                                (flow) =>
                                  `${flow.netRaw >= 0n ? '+' : ''}${formatTokenAmount(flow.netRaw, flow.decimals, 4)} ${flow.ticker}`,
                              )
                              .join(' | ')
                          : 'N/A'}
                      </td>
                      <td>{tx.swapTokens.length > 0 ? tx.swapTokens.join(' -> ') : 'N/A'}</td>
                      <td>{formatEgldFromWei(tx.feeRaw)} EGLD</td>
                      <td>
                        <a
                          href={`${NETWORKS[network].explorerUrl}/transactions/${tx.txHash}`}
                          target="_blank"
                          rel="noreferrer"
                        >
                          {shortAddress(tx.txHash, 8, 6)}
                        </a>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>
        </>
      ) : null}
    </main>
  )
}

export default App
