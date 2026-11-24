(() => {
  'use strict';

  const root = document.documentElement;
  const $ = (sel, el = document) => el.querySelector(sel);
  const $$ = (sel, el = document) => Array.from(el.querySelectorAll(sel));
  const el = (tag, cls, text) => {
    const node = document.createElement(tag);
    if (cls) node.className = cls;
    if (text != null) node.textContent = text;
    return node;
  };

  // ---------------------------------------------------------------- theme

  function currentTheme() {
    if (root.dataset.theme === 'light' || root.dataset.theme === 'dark') return root.dataset.theme;
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }

  function toggleTheme() {
    const next = currentTheme() === 'dark' ? 'light' : 'dark';
    root.dataset.theme = next;
    try {
      localStorage.setItem('urashyl-theme', next);
    } catch {
      /* storage unavailable */
    }
  }

  $('#theme-toggle')?.addEventListener('click', toggleTheme);

  // ------------------------------------------------------------ clipboard

  const copyBtn = $('#copy-env-btn');
  copyBtn?.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText($('#env-snippet')?.textContent || '');
      copyBtn.textContent = 'Copied';
      setTimeout(() => (copyBtn.textContent = 'Copy'), 1600);
    } catch {
      /* clipboard blocked */
    }
  });

  // ----------------------------------------------------------- formatting

  const STATE = {
    connected: ['state-live', 'Live', 'Everything is reporting normally.'],
    degraded: ['state-degraded', 'Degraded', 'Some metrics are missing, so parts of this page may be empty.'],
    error: ['state-error', 'Error', 'The last attempt to read from Grafana failed.'],
    unconfigured: [
      'state-standby',
      'Standby',
      "Monitoring isn't connected on this server, so there's nothing to show yet.",
    ],
  };
  const stateInfo = (s) => STATE[s] || STATE.unconfigured;
  const STATE_CLASSES = ['state-live', 'state-degraded', 'state-error', 'state-standby'];

  const isNum = (v) => typeof v === 'number' && Number.isFinite(v);

  function fmtMetric(v, unit) {
    if (!isNum(v)) return '—';
    if (unit === '%') return `${v.toFixed(1)}%`;
    if (unit === 'guests') return String(Math.round(v));
    if (unit === 'MB/s') return v >= 100 ? `${v.toFixed(0)} MB/s` : `${v.toFixed(2)} MB/s`;
    if (!unit) return v.toFixed(2);
    return `${v.toFixed(1)} ${unit}`;
  }

  function fmtUptime(secs) {
    if (!isNum(secs) || secs < 0) return '—';
    const t = Math.floor(secs);
    const d = Math.floor(t / 86400);
    const h = Math.floor((t % 86400) / 3600);
    const m = Math.floor((t % 3600) / 60);
    if (d > 0) return `${d}d ${h}h ${m}m`;
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  }

  function uptimeProse(secs) {
    if (!isNum(secs) || secs < 0) return null;
    const plural = (n, u) => (n === 1 ? `1 ${u}` : `${n} ${u}s`);
    if (secs >= 86400) return plural(Math.floor(secs / 86400), 'day');
    if (secs >= 3600) return plural(Math.floor(secs / 3600), 'hour');
    return plural(Math.floor(secs / 60), 'minute');
  }

  const RANGE = { '1h': 'hour', '3h': '3 hours', '12h': '12 hours', '24h': '24 hours', '7d': 'week' };

  function liveCaption(snap) {
    const m = snap.metrics || {};
    const cpu = m.cpu || {};
    if ((!cpu.points || cpu.points.length === 0) && !isNum(cpu.current)) {
      return 'Live metrics from the homelab are unavailable right now.';
    }
    let out = `CPU on the homelab host over the past ${RANGE[snap.time_range] || '6 hours'}`;
    const parts = [];
    if (isNum(cpu.current)) parts.push(`${cpu.current.toFixed(0)}% CPU`);
    if (m.memory && isNum(m.memory.current)) parts.push(`${m.memory.current.toFixed(0)}% memory`);
    if (parts.length) out += `. Now: ${parts.join(', ')}`;
    const up = uptimeProse(m.uptime_seconds);
    if (up) out += `, up ${up}`;
    return `${out}.`;
  }

  function fmtIso(iso) {
    if (iso == null) return 'never';
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return String(iso);
    const p = (n) => String(n).padStart(2, '0');
    return `${d.getUTCFullYear()}-${p(d.getUTCMonth() + 1)}-${p(d.getUTCDate())} ${p(d.getUTCHours())}:${p(d.getUTCMinutes())} UTC`;
  }

  function fmtEventTime(ms) {
    const d = new Date(ms);
    if (Number.isNaN(d.getTime())) return String(ms);
    const mon = d.toLocaleString('en-US', { month: 'short', timeZone: 'UTC' });
    const p = (n) => String(n).padStart(2, '0');
    return `${mon} ${d.getUTCDate()}, ${p(d.getUTCHours())}:${p(d.getUTCMinutes())} UTC`;
  }

  // --------------------------------------------------------------- charts

  function computePlot(points, xMin, xMax, yTop, yBottom) {
    if (!Array.isArray(points) || points.length === 0) return null;
    const f = (n) => n.toFixed(1);
    if (points.length === 1) {
      const mid = (yTop + yBottom) / 2;
      return {
        line: `${f(xMin)},${f(mid)} ${f(xMax)},${f(mid)}`,
        area: `M ${f(xMin)} ${f(yBottom)} L ${f(xMin)} ${f(mid)} L ${f(xMax)} ${f(mid)} L ${f(xMax)} ${f(yBottom)} Z`,
        lastX: f(xMax),
        lastY: f(mid),
      };
    }
    let lo = Infinity;
    let hi = -Infinity;
    for (const p of points) {
      lo = Math.min(lo, p.value);
      hi = Math.max(hi, p.value);
    }
    const span = Math.abs(hi - lo);
    const vLo = span < 1e-6 ? lo - 1 : lo - span * 0.1;
    const vHi = span < 1e-6 ? hi + 1 : hi + span * 0.1;
    const range = Math.max(vHi - vLo, 1e-6);
    const t0 = points[0].ts_ms;
    const t1 = points[points.length - 1].ts_ms;
    const byIdx = t1 <= t0;
    const coords = points.map((p, i) => {
      const nx = byIdx ? i / (points.length - 1) : (p.ts_ms - t0) / (t1 - t0);
      const ny = Math.min(1, Math.max(0, (p.value - vLo) / range));
      return [xMin + nx * (xMax - xMin), yBottom - ny * (yBottom - yTop)];
    });
    const first = coords[0];
    const last = coords[coords.length - 1];
    let area = `M ${f(first[0])} ${f(yBottom)}`;
    for (const [x, y] of coords) area += ` L ${f(x)} ${f(y)}`;
    area += ` L ${f(last[0])} ${f(yBottom)} Z`;
    return {
      line: coords.map(([x, y]) => `${f(x)},${f(y)}`).join(' '),
      area,
      lastX: f(last[0]),
      lastY: f(last[1]),
    };
  }

  function drawSvg(svg, points, box) {
    const plot = computePlot(points, ...box);
    const line = $('[data-chart-line]', svg);
    const area = $('[data-chart-area]', svg);
    const dot = $('[data-chart-dot]', svg);
    line?.setAttribute('points', plot ? plot.line : '');
    area?.setAttribute('d', plot ? plot.area : '');
    if (dot) {
      if (plot) {
        dot.setAttribute('cx', plot.lastX);
        dot.setAttribute('cy', plot.lastY);
      }
      dot.classList.toggle('hidden', !plot);
    }
    return Boolean(plot);
  }

  // --------------------------------------------------------- live updates

  function probeInfo(up) {
    if (up === true) return ['probe-up', 'Up'];
    if (up === false) return ['probe-down', 'Down'];
    return ['probe-unknown', 'Not monitored'];
  }

  function setText(sel, text) {
    $$(sel).forEach((n) => (n.textContent = text));
  }

  function apply(snap) {
    if (!snap || typeof snap !== 'object') return;
    const [cls, label, sentence] = stateInfo(snap.state);
    const m = snap.metrics || {};

    $$('[data-state-dot]').forEach((n) => {
      n.classList.remove(...STATE_CLASSES);
      n.classList.add(cls);
      if (n.title) n.title = `Server: ${label}`;
    });
    $$('[data-status-pill]').forEach((n) => {
      n.classList.remove(...STATE_CLASSES);
      n.classList.add(cls);
    });
    setText('[data-status-label]', label);
    setText('[data-status-sentence]', sentence);
    setText('[data-live-caption]', liveCaption(snap));
    setText('[data-last-sync]', fmtIso(snap.last_synced_at));

    const err = $('[data-telemetry-error]');
    if (err) {
      err.textContent = snap.last_error || '';
      err.classList.toggle('hidden', !snap.last_error);
    }

    if (isNum(m.uptime_seconds)) setText('[data-uptime-val]', fmtUptime(m.uptime_seconds));

    const firing = Array.isArray(snap.alerts)
      ? snap.alerts.filter((a) => String(a.state || '').toLowerCase() !== 'suppressed').length
      : 0;
    setText('[data-alerts-count]', String(firing));
    if (Array.isArray(snap.dashboards)) setText('[data-dash-count]', String(snap.dashboards.length));
    setText('[data-grafana-version]', snap.grafana_version ? `v${snap.grafana_version}` : '—');
    setText('[data-active-datasource]', snap.active_datasource ? snap.active_datasource.name : '—');

    const stats = $('[data-homelab-stats]');
    if (stats && m.cpu && isNum(m.cpu.current) && m.memory && isNum(m.memory.current)) {
      stats.textContent = `Right now the host is at ${m.cpu.current.toFixed(0)}% CPU and ${m.memory.current.toFixed(0)}% memory, and has been up for ${fmtUptime(m.uptime_seconds)}.`;
    }

    for (const key of ['cpu', 'memory', 'disk', 'load', 'network']) {
      const s = m[key];
      if (!s) continue;
      if (key === 'load') setText('[data-metric-title="load"]', s.unit === 'guests' ? 'Running guests' : 'Load average');
      setText(`[data-metric-value="${key}"]`, fmtMetric(s.current, s.unit));
      setText(`[data-metric-min="${key}"]`, fmtMetric(s.min, s.unit));
      setText(`[data-metric-avg="${key}"]`, fmtMetric(s.avg, s.unit));
      setText(`[data-metric-max="${key}"]`, fmtMetric(s.max, s.unit));
      $$(`svg[data-trace="${key}"]`).forEach((svg) => drawSvg(svg, s.points, [0, 1000, 10, 110]));
      $$(`svg[data-chart="${key}"]`).forEach((svg) => {
        const drawn = drawSvg(svg, s.points, [8, 472, 14, 134]);
        const empty = $(`[data-chart-empty="${key}"]`);
        if (empty) {
          empty.classList.toggle('hidden', drawn);
          if (!drawn) {
            empty.replaceChildren(
              el('span', s.query_error ? 'err-text' : '', s.query_error ? `Query failed: ${s.query_error}` : 'No samples yet')
            );
          }
        }
      });
    }

    if (Array.isArray(snap.service_probes)) {
      for (const probe of snap.service_probes) {
        const [pcls, ptxt] = probeInfo(probe.up);
        $$(`[data-service-badge="${probe.service_id}"]`).forEach((b) => {
          b.className = `probe ${pcls}`;
          b.textContent = ptxt;
        });
      }
      const body = $('[data-probes-body]');
      if (body) {
        body.replaceChildren(
          ...snap.service_probes.map((probe) => {
            const [pcls, ptxt] = probeInfo(probe.up);
            const tr = el('tr');
            const status = el('td');
            status.appendChild(el('span', `probe ${pcls}`, ptxt));
            tr.append(el('td', '', probe.name), el('td', 'muted', probe.source), status);
            return tr;
          })
        );
      }
    }

    const alertsBody = $('[data-alerts-body]');
    if (alertsBody && Array.isArray(snap.alerts)) {
      if (snap.alerts.length === 0) {
        alertsBody.replaceChildren(
          el('p', 'muted', snap.configured ? 'No alerts are firing.' : 'Alerts appear here once monitoring is connected.')
        );
      } else {
        const ul = el('ul', 'plain-list');
        for (const a of snap.alerts) {
          const li = el('li', 'alert');
          li.append(el('strong', '', a.name), el('span', 'muted', ` ${a.severity}, ${a.state}`));
          if (a.summary) li.appendChild(el('p', '', a.summary));
          ul.appendChild(li);
        }
        alertsBody.replaceChildren(ul);
      }
    }

    const annBody = $('[data-annotations-body]');
    if (annBody && Array.isArray(snap.annotations)) {
      if (snap.annotations.length === 0) {
        annBody.replaceChildren(el('p', 'muted', 'Nothing logged in this window.'));
      } else {
        const ul = el('ul', 'plain-list events');
        for (const ann of snap.annotations.slice(0, 8)) {
          const li = el('li');
          li.append(el('time', 'muted tabular', fmtEventTime(ann.time_ms)), el('span', '', ann.text));
          ul.appendChild(li);
        }
        annBody.replaceChildren(ul);
      }
    }

    const diag = $('[data-diagnostics-list]');
    if (diag && Array.isArray(snap.diagnostics)) {
      diag.replaceChildren(...snap.diagnostics.map((d) => el('li', '', d)));
    }
  }

  const wantsLive = $('[data-state-dot], [data-live], [data-status-pill], [data-service-badge], [data-metric-card]');
  const sse = $('#sse-indicator');
  if (wantsLive && typeof EventSource !== 'undefined') {
    const es = new EventSource('/api/telemetry/stream');
    es.addEventListener('telemetry', (evt) => {
      if (sse) sse.textContent = 'This page updates on its own.';
      try {
        apply(JSON.parse(evt.data));
      } catch {
        /* malformed frame */
      }
    });
    es.addEventListener('error', () => {
      if (sse) sse.textContent = 'Live updates paused, reconnecting.';
    });
  }

  // --------------------------------------------------------------- search

  const palette = $('#cmd-palette');
  const input = $('#cmd-input');
  const results = $('#cmd-results');
  let items = [];
  let selected = 0;
  let timer = null;
  let lastFocus = null;

  const KIND = { page: 'Page', project: 'Project', service: 'Service', dashboard: 'Dashboard', alert: 'Alert', action: 'Action' };

  function render(list) {
    items = list;
    selected = 0;
    if (!results) return;
    if (list.length === 0) {
      results.replaceChildren(el('div', 'cmd-empty', 'No matches. Try a project name, a service, or a tool.'));
      return;
    }
    results.replaceChildren(
      ...list.map((item, i) => {
        const btn = el('button', i === 0 ? 'cmd-item selected' : 'cmd-item');
        btn.type = 'button';
        btn.setAttribute('role', 'option');
        const left = el('div');
        left.append(el('div', 'cmd-item-title', item.title), el('div', 'cmd-item-sub', item.subtitle));
        btn.append(left, el('span', 'cmd-item-kind', KIND[item.kind] || item.kind));
        btn.addEventListener('click', () => go(item));
        btn.addEventListener('mousemove', () => select(i));
        return btn;
      })
    );
  }

  function select(i) {
    selected = i;
    $$('.cmd-item', results).forEach((row, idx) => row.classList.toggle('selected', idx === i));
    $$('.cmd-item', results)[i]?.scrollIntoView({ block: 'nearest' });
  }

  function go(item) {
    close();
    if (typeof item.action === 'function') item.action();
    else if (item.external) window.open(item.href, '_blank', 'noopener,noreferrer');
    else if (item.href) window.location.href = item.href;
  }

  async function search(q) {
    const actions = [{ title: 'Switch color theme', subtitle: 'Light or dark', kind: 'action', action: toggleTheme }];
    const ql = q.trim().toLowerCase();
    const matched = ql ? actions.filter((a) => a.title.toLowerCase().includes(ql)) : actions;
    try {
      const res = await fetch(`/api/search?q=${encodeURIComponent(q)}`);
      const data = await res.json();
      render([...(Array.isArray(data.results) ? data.results : []), ...matched]);
    } catch {
      render(matched);
    }
  }

  function open() {
    if (!palette || !input) return;
    lastFocus = document.activeElement;
    palette.classList.remove('hidden');
    input.value = '';
    input.focus();
    search('');
  }

  function close() {
    if (!palette || palette.classList.contains('hidden')) return;
    palette.classList.add('hidden');
    if (lastFocus && typeof lastFocus.focus === 'function') lastFocus.focus();
  }

  $('#open-cmd-palette')?.addEventListener('click', open);
  $('#cmd-close')?.addEventListener('click', close);
  palette?.addEventListener('click', (e) => {
    if (e.target === palette) close();
  });

  input?.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(() => search(input.value), 120);
  });
  input?.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowDown' && items.length) {
      e.preventDefault();
      select((selected + 1) % items.length);
    } else if (e.key === 'ArrowUp' && items.length) {
      e.preventDefault();
      select((selected - 1 + items.length) % items.length);
    } else if (e.key === 'Enter' && items[selected]) {
      e.preventDefault();
      go(items[selected]);
    }
  });

  document.addEventListener('keydown', (e) => {
    const typing = ['INPUT', 'TEXTAREA', 'SELECT'].includes(document.activeElement?.tagName || '');
    const hidden = !palette || palette.classList.contains('hidden');
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      hidden ? open() : close();
    } else if (e.key === '/' && hidden && !typing) {
      e.preventDefault();
      open();
    } else if (e.key === 'Escape' && !hidden) {
      close();
    }
  });
})();
