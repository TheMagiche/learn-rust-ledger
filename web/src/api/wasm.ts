import initWasm, { handle_request, reset_ledger } from "../wasm/spend_wasm"

export type WasmResponse<T = unknown> = {
  status: number
  handler: string
  concepts: string[]
  data: T
}

let wasmInitPromise: Promise<void> | null = null

export async function ensureWasmInitialized(): Promise<void> {
  if (!wasmInitPromise) {
    wasmInitPromise = (async () => {
      await initWasm()
    })()
  }
  return wasmInitPromise
}

export async function wasmRequest<T>(options: {
  method: string
  url: string
  body?: unknown
}): Promise<WasmResponse<T>> {
  await ensureWasmInitialized()
  const bodyString = options.body !== undefined ? JSON.stringify(options.body) : null
  const response = handle_request(options.method, options.url, bodyString) as WasmResponse<T>
  return response
}

export async function resetWasmLedger(): Promise<void> {
  await ensureWasmInitialized()
  reset_ledger()
}
