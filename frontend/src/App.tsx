import { useCallback, useEffect, useState } from "react";
import { api, Policy, Claim, StatusResponse } from "./api";
import { connectWallet, disconnectWallet, getActiveAddress, getFreighterNetwork } from "./wallet";
import { getXlmBalance, NETWORK_NAME } from "./stellar";
import "./styles.css";

type Tab = "dashboard" | "policies" | "purchase" | "claims" | "admin" | "wallet";
type Toast = { kind: "success" | "error" | "info"; msg: string } | null;

export default function App() {
  const [tab, setTab] = useState<Tab>("dashboard");
  const [dark, setDark] = useState(() => localStorage.getItem("dark") === "1");
  const [toast, setToast] = useState<Toast>(null);
  const [addr, setAddr] = useState<string | null>(null);
  const [net, setNet] = useState<string | null>(null);
  const [bal, setBal] = useState<string | null>(null);
  const [status, setStatus] = useState<StatusResponse | null>(null);

  useEffect(() => { document.documentElement.className = dark ? "dark" : ""; localStorage.setItem("dark", dark ? "1" : "0"); }, [dark]);
  useEffect(() => { if (toast) { const t = setTimeout(() => setToast(null), 4000); return () => clearTimeout(t); } }, [toast]);
  useEffect(() => { (async () => { const a = await getActiveAddress(); if (a) { setAddr(a); const n = await getFreighterNetwork(); setNet(n?.network ?? null); } })(); }, []);

  const refresh = useCallback(async () => { try { setStatus(await api.status()); } catch {} }, []);
  useEffect(() => { refresh(); const i = setInterval(refresh, 30000); return () => clearInterval(i); }, [refresh]);

  const showToast = (k: "success" | "error" | "info", m: string) => setToast({ kind: k, msg: m });

  const onConnect = async () => {
    const r = await connectWallet();
    if (r.error) return showToast("error", r.error);
    setAddr(r.address!); const n = await getFreighterNetwork(); setNet(n?.network ?? null);
    try { setBal(await getXlmBalance(r.address!)); } catch {}
    showToast("success", "Wallet connected");
  };

  const onDisconnect = async () => { await disconnectWallet(); setAddr(null); setNet(null); setBal(null); showToast("info", "Disconnected"); };

  return (
    <div className="app">
      <header className="header">
        <div className="header-left">
          <h1 className="logo">🛡️ DeFi Insurance</h1>
          <span className="badge">v{status?.version || "—"}</span>
          {status && <span className="badge">Pool: {status.pool_balance.toLocaleString()}</span>}
        </div>
        <div className="header-right">
          <button className="icon-btn" onClick={() => setDark(!dark)}>{dark ? "☀️" : "🌙"}</button>
          {addr ? (
            <div className="wallet-pill"><code>{addr.slice(0, 6)}…{addr.slice(-4)}</code><button onClick={onDisconnect}>Disconnect</button></div>
          ) : <button className="primary" onClick={onConnect}>Connect Freighter</button>}
        </div>
      </header>

      <nav className="tabs">
        {(["dashboard", "policies", "purchase", "claims", "admin", "wallet"] as Tab[]).map((t) => (
          <button key={t} className={tab === t ? "tab active" : "tab"} onClick={() => setTab(t)}>
            {t === "dashboard" && "📊 Dashboard"}{t === "policies" && "📋 Policies"}{t === "purchase" && "🛒 Purchase"}{t === "claims" && "⚖️ Claims"}{t === "admin" && "⚙️ Admin"}{t === "wallet" && "👛 Wallet"}
          </button>
        ))}
      </nav>

      <main className="main">
        {tab === "dashboard" && <Dashboard status={status} onRefresh={refresh} />}
        {tab === "policies" && <PoliciesView showToast={showToast} />}
        {tab === "purchase" && <PurchaseForm holder={addr} showToast={showToast} onPurchased={refresh} />}
        {tab === "claims" && <ClaimsView addr={addr} showToast={showToast} onRefresh={refresh} />}
        {tab === "admin" && <AdminPanel showToast={showToast} onRefresh={refresh} />}
        {tab === "wallet" && <WalletView addr={addr} net={net} bal={bal} onConnect={onConnect} onRefresh={async () => { if (addr) try { setBal(await getXlmBalance(addr)); } catch {} }} />}
      </main>

      {toast && <div className={`toast ${toast.kind}`}>{toast.kind === "success" ? "✅" : toast.kind === "error" ? "❌" : "ℹ️"} {toast.msg}</div>}
      <footer className="footer"><p>Stellar DeFi Insurance · Testnet · MIT License</p></footer>
    </div>
  );
}

function Dashboard({ status, onRefresh }: { status: StatusResponse | null; onRefresh: () => void }) {
  const [policies, setPolicies] = useState<Policy[]>([]);
  const [claims, setClaims] = useState<Claim[]>([]);

  useEffect(() => {
    (async () => { try { const [p, c] = await Promise.all([api.listPolicies(), api.listClaims()]); setPolicies(p.slice(0, 5)); setClaims(c.slice(0, 3)); } catch {} })();
  }, [status?.policy_count]);

  return (
    <div>
      <div className="stats-grid">
        <div className="stat-card"><span className="stat-label">Policies</span><span className="stat-value">{status?.policy_count ?? "—"}</span></div>
        <div className="stat-card"><span className="stat-label">Claims</span><span className="stat-value">{status?.claim_count ?? "—"}</span></div>
        <div className="stat-card"><span className="stat-label">Pool Balance</span><span className="stat-value">{status?.pool_balance?.toLocaleString() ?? "—"}</span></div>
        <div className="stat-card"><span className="stat-label">Premium Rate</span><span className="stat-value">{status ? `${(status.premium_rate_bps / 100).toFixed(1)}%` : "—"}</span></div>
      </div>
      <div className="card"><h2>Recent Policies</h2>
        {policies.length === 0 ? <p className="muted">No policies yet.</p> : (
          <div className="table-scroll"><table className="table"><thead><tr><th>#</th><th>Holder</th><th>Coverage</th><th>Premium</th><th>Status</th></tr></thead>
          <tbody>{policies.map((p) => (<tr key={p.id}><td>{p.id}</td><td><code className="addr">{p.holder.slice(0,10)}…</code></td><td>{p.coverage_amount.toLocaleString()}</td><td>{p.premium_paid.toLocaleString()}</td><td><span className={`status status-${p.status}`}>{p.status}</span></td></tr>))}</tbody></table></div>
        )}
      </div>
      <div className="card"><h2>Recent Claims</h2>
        {claims.length === 0 ? <p className="muted">No claims yet.</p> : (
          <div className="table-scroll"><table className="table"><thead><tr><th>#</th><th>Policy</th><th>Amount</th><th>Status</th></tr></thead>
          <tbody>{claims.map((c) => (<tr key={c.id}><td>{c.id}</td><td>{c.policy_id}</td><td>{c.claim_amount.toLocaleString()}</td><td><span className={`status status-${c.status}`}>{c.status}</span></td></tr>))}</tbody></table></div>
        )}
      </div>
      <button onClick={onRefresh}>🔄 Refresh</button>
    </div>
  );
}

function PoliciesView({ showToast }: { showToast: (k: "success" | "error" | "info", m: string) => void }) {
  const [policies, setPolicies] = useState<Policy[]>([]);
  const [loading, setLoading] = useState(true);
  const load = useCallback(async () => { setLoading(true); try { setPolicies(await api.listPolicies()); } catch (e: any) { showToast("error", e.message); } finally { setLoading(false); } }, [showToast]);
  useEffect(() => { load(); }, [load]);
  return (
    <div className="card"><h2>All Policies</h2>
      {loading ? <p className="muted">Loading…</p> : policies.length === 0 ? <p className="muted">No policies.</p> : (
        <div className="table-scroll"><table className="table"><thead><tr><th>#</th><th>Holder</th><th>Covered Contract</th><th>Coverage</th><th>Premium</th><th>Start</th><th>Expiry</th><th>Status</th></tr></thead>
        <tbody>{policies.map((p) => (<tr key={p.id}>
          <td>{p.id}</td><td><code className="addr">{p.holder.slice(0,10)}…</code></td><td><code className="addr">{p.covered_contract.slice(0,10)}…</code></td>
          <td>{p.coverage_amount.toLocaleString()}</td><td>{p.premium_paid.toLocaleString()}</td>
          <td className="ts">{new Date(p.start_time * 1000).toLocaleDateString()}</td><td className="ts">{new Date(p.expiry_time * 1000).toLocaleDateString()}</td>
          <td><span className={`status status-${p.status}`}>{p.status}</span></td>
        </tr>))}</tbody></table></div>
      )}
    </div>
  );
}

function PurchaseForm({ holder, showToast, onPurchased }: { holder: string | null; showToast: (k: "success" | "error" | "info", m: string) => void; onPurchased: () => void }) {
  const [covered, setCovered] = useState("");
  const [coverage, setCoverage] = useState("10000");
  const [duration, setDuration] = useState("3600");
  const [submitting, setSubmitting] = useState(false);
  const [pool, setPool] = useState<{ pool_balance: number; premium_rate_bps: number } | null>(null);

  useEffect(() => { api.pool().then(setPool).catch(() => {}); }, []);

  const premium = pool ? Math.floor((parseInt(coverage) * pool.premium_rate_bps) / 10000) : 0;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!holder) return showToast("error", "Connect wallet first");
    if (!covered) return showToast("error", "Enter covered contract address");
    setSubmitting(true);
    try {
      const r = await api.purchasePolicy(holder, covered, parseInt(coverage), parseInt(duration));
      showToast("success", r.message); setCovered(""); onPurchased();
    } catch (e: any) { showToast("error", e.message); } finally { setSubmitting(false); }
  };

  return (
    <div className="card"><h2>Purchase Insurance Policy</h2>
      <p className="muted">Holder: {holder ? <code>{holder.slice(0,12)}…</code> : "⚠️ Connect wallet"}</p>
      {pool && <p className="muted">Current premium rate: {(pool.premium_rate_bps / 100).toFixed(1)}% · Pool: {pool.pool_balance.toLocaleString()}</p>}
      <form onSubmit={handleSubmit} className="form">
        <label>Covered Contract Address<input value={covered} onChange={(e) => setCovered(e.target.value)} placeholder="C… or G… address to insure" /></label>
        <label>Coverage Amount<input type="number" value={coverage} onChange={(e) => setCoverage(e.target.value)} min="1000" /></label>
        <label>Duration (seconds)<input type="number" value={duration} onChange={(e) => setDuration(e.target.value)} min="1" max="7776000" /></label>
        <div className="premium-preview"><strong>Estimated Premium: {premium.toLocaleString()} units</strong></div>
        <button type="submit" className="primary" disabled={submitting || !holder}>{submitting ? "Processing…" : "Purchase Policy"}</button>
      </form>
    </div>
  );
}

function ClaimsView({ addr, showToast, onRefresh }: { addr: string | null; showToast: (k: "success" | "error" | "info", m: string) => void; onRefresh: () => void }) {
  const [claims, setClaims] = useState<Claim[]>([]);
  const [filing, setFiling] = useState(false);
  const [policyId, setPolicyId] = useState("");
  const [amount, setAmount] = useState("");
  const [evidence, setEvidence] = useState("");

  const load = useCallback(async () => { try { setClaims(await api.listClaims()); } catch {} }, []);
  useEffect(() => { load(); }, [load]);

  const handleFile = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!addr) return showToast("error", "Connect wallet first");
    setFiling(true);
    try {
      const r = await api.fileClaim(addr, parseInt(policyId), parseInt(amount), evidence);
      showToast("success", r.message); setPolicyId(""); setAmount(""); setEvidence(""); load(); onRefresh();
    } catch (e: any) { showToast("error", e.message); } finally { setFiling(false); }
  };

  return (
    <div>
      <div className="card"><h2>File a Claim</h2>
        <form onSubmit={handleFile} className="form">
          <label>Policy ID<input type="number" value={policyId} onChange={(e) => setPolicyId(e.target.value)} placeholder="0" /></label>
          <label>Claim Amount<input type="number" value={amount} onChange={(e) => setAmount(e.target.value)} placeholder="5000" /></label>
          <label>Evidence / Description<textarea value={evidence} onChange={(e) => setEvidence(e.target.value)} placeholder="Describe the exploit or loss…" rows={3} maxLength={1024} /></label>
          <button type="submit" className="primary" disabled={filing || !addr}>{filing ? "Filing…" : "File Claim"}</button>
        </form>
      </div>
      <div className="card"><h2>All Claims</h2>
        {claims.length === 0 ? <p className="muted">No claims filed.</p> : (
          <div className="table-scroll"><table className="table"><thead><tr><th>#</th><th>Policy</th><th>Claimant</th><th>Amount</th><th>Status</th><th>Assessor</th><th>Filed</th></tr></thead>
          <tbody>{claims.map((c) => (<tr key={c.id}>
            <td>{c.id}</td><td>{c.policy_id}</td><td><code className="addr">{c.claimant.slice(0,10)}…</code></td>
            <td>{c.claim_amount.toLocaleString()}</td><td><span className={`status status-${c.status}`}>{c.status}</span></td>
            <td>{c.assessor ? <code className="addr">{c.assessor.slice(0,10)}…</code> : "—"}</td>
            <td className="ts">{new Date(c.filed_at * 1000).toLocaleDateString()}</td>
          </tr>))}</tbody></table></div>
        )}
      </div>
    </div>
  );
}

function AdminPanel({ showToast, onRefresh }: { showToast: (k: "success" | "error" | "info", m: string) => void; onRefresh: () => void }) {
  const [paused, setPaused] = useState(false);
  const [checkAddr, setCheckAddr] = useState("");
  const [assessorResult, setAssessorResult] = useState<boolean | null>(null);
  const [grantAddr, setGrantAddr] = useState("");
  const [claims, setClaims] = useState<Claim[]>([]);

  useEffect(() => { api.isPaused().then(r => setPaused(r.paused)).catch(() => {}); api.listClaims().then(setClaims).catch(() => {}); }, []);

  const togglePause = async () => { try { if (paused) { await api.unpause(); setPaused(false); showToast("success", "Resumed"); } else { await api.pause(); setPaused(true); showToast("info", "Paused"); } onRefresh(); } catch (e: any) { showToast("error", e.message); } };
  const checkA = async () => { try { const r = await api.checkAssessor(checkAddr); setAssessorResult(r.is_assessor); } catch (e: any) { showToast("error", e.message); } };
  const grantA = async () => { try { await api.grantAssessor(grantAddr); showToast("success", "Assessor granted"); setGrantAddr(""); } catch (e: any) { showToast("error", e.message); } };

  const approveC = async (id: number) => { try { await api.approveClaim("GA2C5RFPE6GCKMY3US5GAB29QZYE7DYXTZPGHLDJ5YBJO7VOZKAUBP2X", id); showToast("success", `Claim ${id} approved`); api.listClaims().then(setClaims); } catch (e: any) { showToast("error", e.message); } };
  const rejectC = async (id: number) => { try { await api.rejectClaim("GA2C5RFPE6GCKMY3US5GAB29QZYE7DYXTZPGHLDJ5YBJO7VOZKAUBP2X", id); showToast("info", `Claim ${id} rejected`); api.listClaims().then(setClaims); } catch (e: any) { showToast("error", e.message); } };
  const payC = async (id: number) => { try { await api.markPaid(id); showToast("success", `Claim ${id} paid`); api.listClaims().then(setClaims); onRefresh(); } catch (e: any) { showToast("error", e.message); } };

  const pendingClaims = claims.filter(c => c.status === "pending");

  return (
    <div className="admin-panel">
      <div className="card"><h2>Protocol Controls</h2>
        <div className="detail-row"><span className="detail-label">Status:</span><span className={paused ? "warn" : "ok"}>{paused ? "⏸ Paused" : "✅ Active"}</span></div>
        <button className={paused ? "primary" : ""} onClick={togglePause}>{paused ? "▶️ Resume" : "⏸ Pause"}</button>
      </div>
      <div className="card"><h2>Pending Claims ({pendingClaims.length})</h2>
        {pendingClaims.length === 0 ? <p className="muted">No pending claims.</p> : pendingClaims.map(c => (
          <div key={c.id} className="claim-item">
            <p><strong>Claim #{c.id}</strong> · Policy #{c.policy_id} · Amount: {c.claim_amount.toLocaleString()}</p>
            <p className="muted evidence">{c.evidence}</p>
            <div className="actions-row"><button className="primary" onClick={() => approveC(c.id)}>Approve</button><button onClick={() => rejectC(c.id)}>Reject</button></div>
          </div>
        ))}
      </div>
      <div className="card"><h2>Assessor Management</h2>
        <div className="inline-form"><input value={checkAddr} onChange={(e) => setCheckAddr(e.target.value)} placeholder="G… address to check" /><button onClick={checkA}>Check</button></div>
        {assessorResult !== null && <p className={assessorResult ? "ok" : "warn"}>{assessorResult ? "✅ Is assessor" : "❌ Not assessor"}</p>}
        <div className="inline-form"><input value={grantAddr} onChange={(e) => setGrantAddr(e.target.value)} placeholder="G… to grant" /><button className="primary" onClick={grantA}>Grant</button></div>
      </div>
    </div>
  );
}

function WalletView({ addr, net, bal, onConnect, onRefresh }: { addr: string | null; net: string | null; bal: string | null; onConnect: () => void; onRefresh: () => void }) {
  return (
    <div className="card"><h2>Freighter Wallet</h2>
      {!addr ? (<div><p className="muted">Connect to purchase policies and file claims.</p><button className="primary" onClick={onConnect}>Connect Freighter</button></div>) : (
        <div>
          <div className="detail-row"><span className="detail-label">Address:</span><code className="mono">{addr}</code></div>
          <div className="detail-row"><span className="detail-label">Network:</span><span className={net === NETWORK_NAME ? "ok" : "warn"}>{net ?? "unknown"}{net !== NETWORK_NAME ? " (switch to Testnet!)" : ""}</span></div>
          <div className="detail-row"><span className="detail-label">XLM:</span><strong>{bal ?? "—"}</strong></div>
          <button onClick={onRefresh}>🔄 Refresh Balance</button>
        </div>
      )}
    </div>
  );
}
