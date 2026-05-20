import type { Operation, TokenFlow } from './types'

const EGLD_DECIMALS = 18

export function base64ToText(input?: string): string {
  if (!input) {
    return ''
  }

  try {
    return atob(input)
  } catch {
    return ''
  }
}

export function base64ToBigInt(input?: string): bigint | undefined {
  if (!input) {
    return undefined
  }

  const raw = base64ToText(input)
  if (!raw) {
    return undefined
  }

  let value = 0n
  for (let i = 0; i < raw.length; i += 1) {
    value = (value << 8n) + BigInt(raw.charCodeAt(i))
  }

  return value
}

export function formatTokenAmount(
  value: bigint | string | number,
  decimals = 0,
  fractionDigits = 4,
): string {
  const raw = typeof value === 'bigint' ? value : BigInt(value)
  const negative = raw < 0n
  const abs = negative ? raw * -1n : raw

  if (decimals === 0) {
    return `${negative ? '-' : ''}${abs.toString()}`
  }

  const divisor = 10n ** BigInt(decimals)
  const whole = abs / divisor
  const fraction = abs % divisor

  if (fraction === 0n) {
    return `${negative ? '-' : ''}${whole.toString()}`
  }

  const fractionStr = fraction
    .toString()
    .padStart(decimals, '0')
    .slice(0, fractionDigits)
    .replace(/0+$/, '')

  const body = fractionStr.length > 0 ? `${whole.toString()}.${fractionStr}` : whole.toString()

  return `${negative ? '-' : ''}${body}`
}

export function formatEgldFromWei(wei?: string): string {
  if (!wei) {
    return '0'
  }

  return formatTokenAmount(BigInt(wei), EGLD_DECIMALS, 6)
}

export function shortAddress(address: string, head = 8, tail = 8): string {
  if (address.length <= head + tail) {
    return address
  }

  return `${address.slice(0, head)}...${address.slice(-tail)}`
}

function hexToText(hex: string): string {
  if (hex.length % 2 !== 0) {
    return ''
  }

  let out = ''

  for (let i = 0; i < hex.length; i += 2) {
    const code = Number.parseInt(hex.slice(i, i + 2), 16)
    if (Number.isNaN(code)) {
      return ''
    }

    out += String.fromCharCode(code)
  }

  return out
}

export function parseExecuteTradesData(encoded?: string): {
  inputAmountRaw?: bigint
  swapTokens: string[]
} {
  if (!encoded) {
    return { swapTokens: [] }
  }

  const decoded = base64ToText(encoded)
  if (!decoded || !decoded.startsWith('executeTrades')) {
    return { swapTokens: [] }
  }

  const segments = decoded.split('@')
  const amountHex = segments[1]

  let inputAmountRaw: bigint | undefined
  if (amountHex) {
    try {
      inputAmountRaw = BigInt(`0x${amountHex}`)
    } catch {
      inputAmountRaw = undefined
    }
  }

  const swapTokens: string[] = []

  for (let i = 3; i < segments.length; i += 2) {
    const tokenHex = segments[i]
    if (!tokenHex) {
      continue
    }

    const token = hexToText(tokenHex)
    if (token) {
      swapTokens.push(token)
    }
  }

  return { inputAmountRaw, swapTokens }
}

export function buildTokenFlows(
  operations: Operation[] | undefined,
  contractAddress: string,
): TokenFlow[] {
  if (!operations || operations.length === 0) {
    return []
  }

  const flows = new Map<string, TokenFlow>()

  for (const op of operations) {
    if (op.action !== 'transfer' || op.type !== 'esdt') {
      continue
    }

    if (!op.identifier || !op.value) {
      continue
    }

    const value = BigInt(op.value)
    const sender = op.sender ?? ''
    const receiver = op.receiver ?? ''

    let delta = 0n
    if (sender === contractAddress) {
      delta -= value
    }
    if (receiver === contractAddress) {
      delta += value
    }

    if (delta === 0n) {
      continue
    }

    const existing = flows.get(op.identifier)
    if (existing) {
      existing.netRaw += delta
      continue
    }

    flows.set(op.identifier, {
      tokenId: op.identifier,
      ticker: op.ticker ?? op.identifier,
      decimals: op.decimals ?? 0,
      netRaw: delta,
    })
  }

  return [...flows.values()].sort((a, b) => {
    if (a.netRaw === b.netRaw) {
      return 0
    }

    return a.netRaw > b.netRaw ? -1 : 1
  })
}

export function unixToLocaleDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString()
}
