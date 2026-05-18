import React from 'react';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  BarChart, Bar,
} from 'recharts';
import {
  getOverview, getDau, getPageviews, getPageRanking, getFeedback, getSkills,
  triggerCrawl, clearToken,
  type Overview, type DauCount, type PvCount, type PageRank, type Feedback,
  type AdminSkill, type PaginatedSkills,
} from '../api/client';

function todayDefault() {
  const d = new Date();
  return d.toISOString().slice(0, 10);
}
function thirtyDaysAgo() {
  const d = new Date();
  d.setDate(d.getDate() - 30);
  return d.toISOString().slice(0, 10);
}

export default function Dashboard() {
  const [from, setFrom] = React.useState(thirtyDaysAgo);
  const [to, setTo] = React.useState(todayDefault);
  const [overview, setOverview] = React.useState<Overview | null>(null);
  const [dau, setDau] = React.useState<DauCount[]>([]);
  const [pv, setPv] = React.useState<PvCount[]>([]);
  const [pages, setPages] = React.useState<PageRank[]>([]);
  const [feedback, setFeedback] = React.useState<Feedback[]>([]);
  const [skillsData, setSkillsData] = React.useState<PaginatedSkills | null>(null);
  const [loading, setLoading] = React.useState(true);
  const [tab, setTab] = React.useState<'overview' | 'feedback' | 'skills'>('overview');
  const [skillsPage, setSkillsPage] = React.useState(1);
  const [crawling, setCrawling] = React.useState(false);
  const [crawlMsg, setCrawlMsg] = React.useState<string | null>(null);

  const fetchData = React.useCallback(async () => {
    setLoading(true);
    const settled = await Promise.allSettled([
      getOverview(from, to),
      getDau(from, to),
      getPageviews(from, to),
      getPageRanking(from, to),
      getFeedback(),
    ]);
    const [ov, dauData, pvData, pagesData, fb] = settled;
    if (ov.status === 'fulfilled') setOverview(ov.value);
    if (dauData.status === 'fulfilled') setDau(dauData.value);
    if (pvData.status === 'fulfilled') setPv(pvData.value);
    if (pagesData.status === 'fulfilled') setPages(pagesData.value);
    if (fb.status === 'fulfilled') setFeedback(fb.value);
    for (const r of settled) {
      if (r.status === 'rejected') console.error('Failed to load stats:', r.reason);
    }
    setLoading(false);
  }, [from, to]);

  const fetchSkills = React.useCallback(async (page: number) => {
    try {
      const data = await getSkills(page, 20);
      setSkillsData(data);
    } catch (e) {
      console.error('Failed to load skills:', e);
    }
  }, []);

  React.useEffect(() => { fetchData(); }, [fetchData]);

  // Fetch skills when tab switches to skills or page changes
  React.useEffect(() => {
    if (tab === 'skills') {
      fetchSkills(skillsPage);
    }
  }, [tab, skillsPage, fetchSkills]);

  const totalPages = skillsData ? Math.ceil(skillsData.total / skillsData.per_page) : 0;

  return (
    <div className="min-h-screen">
      {/* Header */}
      <header className="bg-white border-b border-slate-200 px-6 py-4 flex items-center justify-between">
        <h1 className="text-lg font-semibold text-slate-800">SkillBase Admin</h1>
        <div className="flex items-center gap-3 text-sm">
          {crawlMsg && (
            <span className={`text-xs ${crawlMsg.includes('success') ? 'text-green-600' : 'text-red-500'}`}>
              {crawlMsg}
            </span>
          )}
          <button
            onClick={async () => {
              setCrawling(true);
              setCrawlMsg(null);
              try {
                const res = await triggerCrawl();
                setCrawlMsg(res.message);
                setTimeout(() => setCrawlMsg(null), 3000);
              } catch (e) {
                setCrawlMsg('Crawl failed');
              } finally {
                setCrawling(false);
              }
            }}
            disabled={crawling}
            className="bg-amber-500 text-white px-3 py-1.5 rounded text-sm hover:bg-amber-600 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {crawling ? 'Crawling...' : 'Trigger Crawl'}
          </button>
          <button onClick={() => { clearToken(); window.location.reload(); }}
            className="text-slate-400 hover:text-slate-600">
            Logout
          </button>
          {tab !== 'skills' && (
            <>
              <label className="text-slate-500">From:</label>
              <input type="date" value={from} onChange={e => setFrom(e.target.value)}
                className="border border-slate-300 rounded px-2 py-1 text-sm" />
              <label className="text-slate-500">To:</label>
              <input type="date" value={to} onChange={e => setTo(e.target.value)}
                className="border border-slate-300 rounded px-2 py-1 text-sm" />
            </>
          )}
          <button onClick={() => tab === 'skills' ? fetchSkills(skillsPage) : fetchData()}
            className="bg-blue-600 text-white px-3 py-1.5 rounded text-sm hover:bg-blue-700">
            Refresh
          </button>
        </div>
      </header>

      {/* Tabs */}
      <div className="border-b border-slate-200 bg-white px-6">
        {(['overview', 'feedback', 'skills'] as const).map(t => (
          <button key={t} onClick={() => { setTab(t); if (t === 'skills') setSkillsPage(1); }}
            className={`px-4 py-3 text-sm font-medium border-b-2 transition-colors capitalize ${
              tab === t ? 'border-blue-600 text-blue-600' : 'border-transparent text-slate-500 hover:text-slate-700'
            }`}>
            {t}
            {t === 'feedback' && feedback.length > 0 && <span className="ml-1 text-xs text-slate-400">({feedback.length})</span>}
            {t === 'skills' && skillsData && <span className="ml-1 text-xs text-slate-400">({skillsData.total})</span>}
          </button>
        ))}
      </div>

      <div className="p-6">
        {loading && tab !== 'skills' && <div className="text-center text-slate-400 py-12">Loading...</div>}

        {/* Overview tab */}
        {!loading && tab === 'overview' && (
          <div className="space-y-6">
            {overview && (
              <div className="grid grid-cols-3 gap-4">
                <div className="bg-white rounded-lg border border-slate-200 p-5">
                  <p className="text-xs text-slate-500 uppercase tracking-wide">Daily Active Users</p>
                  <p className="text-2xl font-bold text-slate-800 mt-1">{overview.dau}</p>
                </div>
                <div className="bg-white rounded-lg border border-slate-200 p-5">
                  <p className="text-xs text-slate-500 uppercase tracking-wide">Page Views</p>
                  <p className="text-2xl font-bold text-slate-800 mt-1">{overview.pv}</p>
                </div>
                <div className="bg-white rounded-lg border border-slate-200 p-5">
                  <p className="text-xs text-slate-500 uppercase tracking-wide">Total Feedback</p>
                  <p className="text-2xl font-bold text-slate-800 mt-1">{overview.totalFeedback}</p>
                </div>
              </div>
            )}
            <div className="bg-white rounded-lg border border-slate-200 p-5">
              <h3 className="text-sm font-medium text-slate-700 mb-4">Daily Active Users</h3>
              <ResponsiveContainer width="100%" height={250}>
                <LineChart data={dau}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
                  <XAxis dataKey="date" tick={{ fontSize: 12 }} stroke="#94a3b8" />
                  <YAxis allowDecimals={false} tick={{ fontSize: 12 }} stroke="#94a3b8" />
                  <Tooltip />
                  <Line type="monotone" dataKey="count" stroke="#2563eb" strokeWidth={2} dot={{ r: 3 }} name="DAU" />
                </LineChart>
              </ResponsiveContainer>
            </div>
            <div className="bg-white rounded-lg border border-slate-200 p-5">
              <h3 className="text-sm font-medium text-slate-700 mb-4">Page Views</h3>
              <ResponsiveContainer width="100%" height={250}>
                <BarChart data={pv}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
                  <XAxis dataKey="date" tick={{ fontSize: 12 }} stroke="#94a3b8" />
                  <YAxis allowDecimals={false} tick={{ fontSize: 12 }} stroke="#94a3b8" />
                  <Tooltip />
                  <Bar dataKey="count" fill="#6366f1" radius={[4, 4, 0, 0]} name="PV" />
                </BarChart>
              </ResponsiveContainer>
            </div>
            <div className="bg-white rounded-lg border border-slate-200 p-5">
              <h3 className="text-sm font-medium text-slate-700 mb-4">Page Ranking</h3>
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-slate-200">
                    <th className="text-left py-2 text-slate-500 font-medium">#</th>
                    <th className="text-left py-2 text-slate-500 font-medium">Page</th>
                    <th className="text-right py-2 text-slate-500 font-medium">Visits</th>
                  </tr>
                </thead>
                <tbody>
                  {pages.map((p, i) => (
                    <tr key={p.page} className="border-b border-slate-100">
                      <td className="py-2 text-slate-400">{i + 1}</td>
                      <td className="py-2 font-mono text-xs text-slate-700">{p.page}</td>
                      <td className="py-2 text-right text-slate-600">{p.count}</td>
                    </tr>
                  ))}
                  {pages.length === 0 && (
                    <tr><td colSpan={3} className="py-4 text-center text-slate-400">No data</td></tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* Feedback tab */}
        {!loading && tab === 'feedback' && (
          <div className="bg-white rounded-lg border border-slate-200">
            <div className="p-5 border-b border-slate-200">
              <h3 className="text-sm font-medium text-slate-700">Feedback Submissions</h3>
            </div>
            {feedback.length === 0 ? (
              <div className="p-8 text-center text-slate-400">No feedback yet</div>
            ) : (
              <div className="divide-y divide-slate-100">
                {feedback.map(fb => (
                  <div key={fb.id} className="p-5">
                    <div className="flex items-start justify-between mb-2">
                      <h4 className="text-sm font-medium text-slate-800">{fb.title}</h4>
                      <span className="text-xs text-slate-400">
                        {new Date(fb.createdAt).toLocaleString()}
                      </span>
                    </div>
                    <p className="text-sm text-slate-600 whitespace-pre-wrap mb-2">{fb.description}</p>
                    {fb.submitterIp && (
                      <p className="text-xs text-slate-400">IP: {fb.submitterIp}</p>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Skills tab */}
        {tab === 'skills' && (
          <div className="bg-white rounded-lg border border-slate-200">
            <div className="p-5 border-b border-slate-200 flex items-center justify-between">
              <h3 className="text-sm font-medium text-slate-700">Crawled Skills</h3>
              {skillsData && (
                <span className="text-xs text-slate-400">
                  Page {skillsData.page} of {totalPages} ({skillsData.total} total)
                </span>
              )}
            </div>
            {!skillsData || skillsData.data.length === 0 ? (
              <div className="p-8 text-center text-slate-400">No skills crawled yet</div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-slate-200 bg-slate-50">
                      <th className="text-left py-2.5 px-4 text-slate-500 font-medium">Name</th>
                      <th className="text-left py-2.5 px-4 text-slate-500 font-medium">Source</th>
                      <th className="text-left py-2.5 px-4 text-slate-500 font-medium">License</th>
                      <th className="text-right py-2.5 px-4 text-slate-500 font-medium">Installs</th>
                      <th className="text-right py-2.5 px-4 text-slate-500 font-medium">Created</th>
                    </tr>
                  </thead>
                  <tbody>
                    {skillsData.data.map((s: AdminSkill) => (
                      <tr key={s.id} className="border-b border-slate-100 hover:bg-slate-50">
                        <td className="py-2.5 px-4 text-slate-800 font-medium max-w-[300px] truncate">{s.name}</td>
                        <td className="py-2.5 px-4">
                          <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-700">
                            {s.source}
                          </span>
                        </td>
                        <td className="py-2.5 px-4 text-slate-500 text-xs">{s.license || '-'}</td>
                        <td className="py-2.5 px-4 text-right text-slate-600">{s.install_count}</td>
                        <td className="py-2.5 px-4 text-right text-xs text-slate-400">
                          {new Date(s.created_at).toLocaleDateString()}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
            {/* Pagination */}
            {totalPages > 1 && (
              <div className="flex items-center justify-center gap-2 p-4 border-t border-slate-200">
                <button
                  disabled={skillsPage <= 1}
                  onClick={() => setSkillsPage(p => Math.max(1, p - 1))}
                  className="px-3 py-1 text-sm border border-slate-300 rounded hover:bg-slate-50 disabled:opacity-40 disabled:cursor-not-allowed"
                >Prev</button>
                <span className="text-xs text-slate-500">{skillsPage} / {totalPages}</span>
                <button
                  disabled={skillsPage >= totalPages}
                  onClick={() => setSkillsPage(p => Math.min(totalPages, p + 1))}
                  className="px-3 py-1 text-sm border border-slate-300 rounded hover:bg-slate-50 disabled:opacity-40 disabled:cursor-not-allowed"
                >Next</button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
