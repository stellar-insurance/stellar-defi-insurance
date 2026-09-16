const BASE = "/api/v1";

export type PolicyStatus = "active" | "expired" | "claimed" | "cancelled";
export type ClaimStatus = "pending" | "approved" | "rejected" | "paid";

export interface Policy {
  id: number; holder: string; covered_contract: string;
  coverage_amount: number; premium_paid: number;
  start_time: number; expiry_time: number;
  status: PolicyStatus; ledger: number;
}

export interface Claim {
  id: number; policy_id: number; claimant: string;
  claim_amount: number; evidence: string;
  status: ClaimStatus; assessor: string | null;
  filed_at: number; reviewed_at: number | null;
}

export interface StatusResponse {
  status: string; version: string; uptime_seconds: number;
  network: string; contract_id: string;
  policy_count: number; claim_count: number;
  pool_balance: number; premium_rate_bps: number;
}

async function jsonFetch<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, { headers: { "Content-Type": "application/json" }, ...init });
  if (!res.ok) { const e = await res.json().catch(() => ({ message: res.statusText })); throw new Error(e.message || `HTTP ${res.status}`); }
  return res.json();
}

export const api = {
  health: () => jsonFetch<{ status: string; version: string; uptime_seconds: number }>("/health"),
  status: () => jsonFetch<StatusResponse>(`${BASE}/status`),
  listPolicies: () => jsonFetch<Policy[]>(`${BASE}/policies`),
  getPolicy: (id: number) => jsonFetch<Policy>(`${BASE}/policies/${id}`),
  policyCount: () => jsonFetch<{ count: number }>(`${BASE}/policies/count`),
  purchasePolicy: (holder: string, covered_contract: string, coverage_amount: number, duration_seconds: number) =>
    jsonFetch<{ success: boolean; message: string }>(`${BASE}/policies`, { method: "POST", body: JSON.stringify({ holder, covered_contract, coverage_amount, duration_seconds }) }),
  listClaims: () => jsonFetch<Claim[]>(`${BASE}/claims`),
  getClaim: (id: number) => jsonFetch<Claim>(`${BASE}/claims/${id}`),
  claimCount: () => jsonFetch<{ count: number }>(`${BASE}/claims/count`),
  fileClaim: (claimant: string, policy_id: number, claim_amount: number, evidence: string) =>
    jsonFetch<{ success: boolean; message: string }>(`${BASE}/claims`, { method: "POST", body: JSON.stringify({ claimant, policy_id, claim_amount, evidence }) }),
  approveClaim: (assessor: string, claim_id: number) =>
    jsonFetch<{ success: boolean; message: string }>(`${BASE}/claims/approve`, { method: "POST", body: JSON.stringify({ assessor, claim_id }) }),
  rejectClaim: (assessor: string, claim_id: number) =>
    jsonFetch<{ success: boolean; message: string }>(`${BASE}/claims/reject`, { method: "POST", body: JSON.stringify({ assessor, claim_id }) }),
  markPaid: (id: number) => jsonFetch<{ success: boolean; message: string }>(`${BASE}/claims/${id}/pay`, { method: "POST" }),
  pool: () => jsonFetch<{ pool_balance: number; premium_rate_bps: number }>(`${BASE}/pool`),
  getAdmin: () => jsonFetch<{ admin: string }>(`${BASE}/admin`),
  checkAssessor: (addr: string) => jsonFetch<{ address: string; is_assessor: boolean }>(`${BASE}/assessors/${addr}`),
  grantAssessor: (address: string) => jsonFetch(`${BASE}/assessors/grant`, { method: "POST", body: JSON.stringify({ admin: "", address }) }),
  revokeAssessor: (address: string) => jsonFetch(`${BASE}/assessors/revoke`, { method: "POST", body: JSON.stringify({ admin: "", address }) }),
  pause: () => jsonFetch(`${BASE}/pause`, { method: "POST" }),
  unpause: () => jsonFetch(`${BASE}/unpause`, { method: "POST" }),
  isPaused: () => jsonFetch<{ paused: boolean }>(`${BASE}/paused`),
};
