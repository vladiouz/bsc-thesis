import { devnetContractAddressFromState } from '../generated/stateContract'
import type { Network, NetworkConfig } from './types'

export const NETWORKS: Record<Network, NetworkConfig> = {
  devnet: {
    network: 'devnet',
    label: 'Devnet',
    apiUrl: 'https://devnet-api.multiversx.com',
    gatewayUrl: 'https://devnet-gateway.multiversx.com',
    explorerUrl: 'https://devnet-explorer.multiversx.com',
  },
  mainnet: {
    network: 'mainnet',
    label: 'Mainnet',
    apiUrl: 'https://api.multiversx.com',
    gatewayUrl: 'https://gateway.multiversx.com',
    explorerUrl: 'https://explorer.multiversx.com',
  },
}

export const DEFAULT_NETWORK: Network =
  import.meta.env.VITE_DEFAULT_NETWORK === 'mainnet' ? 'mainnet' : 'devnet'

export function getNetworkFromUrl(search: string): Network {
  const params = new URLSearchParams(search)
  const requested = params.get('network')

  if (requested === 'devnet' || requested === 'mainnet') {
    return requested
  }

  return DEFAULT_NETWORK
}

export function getContractAddress(network: Network, search: string): string {
  const params = new URLSearchParams(search)
  const fromQuery = params.get('contract')?.trim()

  if (fromQuery) {
    return fromQuery
  }

  if (network === 'mainnet') {
    return (import.meta.env.VITE_MAINNET_CONTRACT_ADDRESS ?? '').trim()
  }

  return (
    (import.meta.env.VITE_DEVNET_CONTRACT_ADDRESS ?? '').trim() ||
    devnetContractAddressFromState
  )
}
