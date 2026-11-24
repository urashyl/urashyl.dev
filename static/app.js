(() => {
  const root = document.documentElement;
  const themeBtn = document.getElementById('theme-toggle');
  const themeLabel = document.getElementById('theme-label');

  function syncThemeUI() {
    const current = root.dataset.theme === 'light' ? 'light' : 'dark';
    if (themeLabel) {
      themeLabel.textContent = current === 'light' ? 'Dark' : 'Light';
    }
  }

  function toggleTheme() {
    const next = root.dataset.theme === 'light' ? 'dark' : 'light';
    root.dataset.theme = next;
    try {
      localStorage.setItem('urashyl-theme', next);
    } catch {
      // Ignore storage failures
    }
    syncThemeUI();
  }

  syncThemeUI();
  if (themeBtn) {
    themeBtn.addEventListener('click', toggleTheme);
  }

  // Mobile menu
  const mobileToggle = document.getElementById('mobile-nav-toggle');
  const mobileNav = document.getElementById('mobile-nav');
  if (mobileToggle && mobileNav) {
    mobileToggle.addEventListener('click', () => {
      const open = mobileNav.classList.toggle('hidden') === false;
      mobileToggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    });
  }

  // Copy buttons
  function bindCopy(btnId, labelId, getText, defaultText, copiedText) {
    const btn = document.getElementById(btnId);
    const lbl = labelId ? document.getElementById(labelId) : btn;
    if (!btn || !lbl) return;
    btn.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(getText());
        lbl.textContent = copiedText;
        setTimeout(() => {
          lbl.textContent = defaultText;
        }, 1800);
      } catch {
        // Ignore clipboard errors in restricted contexts
      }
    });
  }

  bindCopy(
    'copy-domain-btn',
    'copy-domain-label',
    () => 'https://urashyl.dev',
    'Copy urashyl.dev',
    'Copied URL'
  );
  bindCopy(
    'copy-env-btn',
    'copy-env-btn',
    () => {
      const el = document.getElementById('env-snippet');
      return el ? el.textContent || '' : '';
    },
    'Copy .env',
    'Copied'
  );

  // Formatting helpers matching Rust server output
  function stateLabel(state) {
    switch (state) {
      case 'connected':
        return 'Live';
      case 'degraded':
        return 'Degraded';
      case 'error':
        return 'Error';
      default:
        return 'Standby';
    }
  }

  function stateClass(state) {
    switch (state) {
      case 'connected':
        return 'state-live';
      case 'degraded':
        return 'state-degraded';
      case 'error':
        return 'state-error';
      default:
        return 'state-standby';
    }
  }

  function formatMetricValue(val, unit) {
    if (typeof val !== 'number' || !Number.isFinite(val)) return '—';
    if (unit === '%') return `${val.toFixed(1)}%`;
    if (unit === 'guests') return String(Math.round(val));
    if (unit === 'MB/s') return val >= 100 ? `${val.toFixed(0)} MB/s` : `${val.toFixed(2)} MB/s`;
    if (!unit) return val.toFixed(2);
    return `${val.toFixed(1)} ${unit}`;
  }

  function formatUptime(secs) {
    if (typeof secs !== 'number' || !Number.isFinite(secs) || secs < 0) return '—';
    const total = Math.floor(secs);
    const days = Math.floor(total / 86400);
    const hours = Math.floor((total % 86400) / 3600);
    const mins = Math.floor((total % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h ${mins}m`;
    if (hours > 0) return `${hours}h ${mins}m`;
    return `${mins}m`;
  }

  function formatIsoShort(iso) {
    if (!iso) return 'never';
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return String(iso);
    const pad = (n) => String(n).padStart(2, '0');
    return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())} ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())} UTC`;
  }

  function computePlot(points, xMin, xMax, yTop, yBottom) {
    if (!Array.isArray(points) || points.length === 0) return null;
    if (points.length === 1) {
      const midY = ((yTop + yBottom) * 0.5).toFixed(1);
      return {
        polyline: `${xMin.toFixed(1)},${midY} ${xMax.toFixed(1)},${midY}`,
        area: `M ${xMin.toFixed(1)} ${yBottom.toFixed(1)} L ${xMin.toFixed(1)} ${midY} L ${xMax.toFixed(1)} ${midY} L ${xMax.toFixed(1)} ${yBottom.toFixed(1)} Z`,
        lastX: xMax.toFixed(1),
        lastY: midY,
      };
    }

    let minV = Infinity;
    let maxV = -Infinity;
    for (const p of points) {
      if (p.value < minV) minV = p.value;
      if (p.value > maxV) maxV = p.value;
    }
    const span = Math.abs(maxV - minV);
    const vLo = span < 1e-6 ? minV - 1 : minV - span * 0.1;
    const vHi = span < 1e-6 ? maxV + 1 : maxV + span * 0.1;
    const vRange = Math.max(vHi - vLo, 1e-6);

    const firstTs = points[0].ts_ms;
    const lastTs = points[points.length - 1].ts_ms;
    const useIdx = lastTs <= firstTs;
    const w = xMax - xMin;
    const h = yBottom - yTop;

    const coords = points.map((p, idx) => {
      const nx = useIdx ? idx / (points.length - 1) : (p.ts_ms - firstTs) / (lastTs - firstTs);
      const ny = Math.min(1, Math.max(0, (p.value - vLo) / vRange));
      return [xMin + nx * w, yBottom - ny * h];
    });

    const polyline = coords.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(' ');
    const first = coords[0];
    const last = coords[coords.length - 1];
    let area = `M ${first[0].toFixed(1)} ${yBottom.toFixed(1)}`;
    for (const [x, y] of coords) {
      area += ` L ${x.toFixed(1)} ${y.toFixed(1)}`;
    }
    area += ` L ${last[0].toFixed(1)} ${yBottom.toFixed(1)} Z`;

    return {
      polyline,
      area,
      lastX: last[0].toFixed(1),
      lastY: last[1].toFixed(1),
    };
  }

  function updateSvgElement(svg, points, xMin, xMax, yTop, yBottom, emptyOverlay, queryError) {
    if (!svg) return;
    const line = svg.querySelector('[data-chart-line]');
    const area = svg.querySelector('[data-chart-area]');
    const dot = svg.querySelector('[data-chart-dot]');
    const plot = computePlot(points, xMin, xMax, yTop, yBottom);

    if (plot) {
      if (line) line.setAttribute('points', plot.polyline);
      if (area) area.setAttribute('d', plot.area);
      if (dot) {
        dot.setAttribute('cx', plot.lastX);
        dot.setAttribute('cy', plot.lastY);
        dot.classList.remove('hidden');
      }
      if (emptyOverlay) emptyOverlay.classList.add('hidden');
    } else {
      if (line) line.setAttribute('points', '');
      if (area) area.setAttribute('d', '');
      if (dot) dot.classList.add('hidden');
      if (emptyOverlay) {
        const msg = document.createElement('span');
        if (queryError) {
          msg.className = 'mono err-text';
          msg.textContent = `Query error: ${queryError}`;
        } else {
          msg.className = 'mono muted-text';
          msg.textContent = 'Awaiting Grafana samples';
        }
        emptyOverlay.replaceChildren(msg);
        emptyOverlay.classList.remove('hidden');
      }
    }
  }

  function swapStateClass(el, state) {
    if (!el) return;
    el.classList.remove('state-live', 'state-degraded', 'state-error', 'state-standby');
    el.classList.add(stateClass(state));
  }

  function applyTelemetrySnapshot(snap) {
    if (!snap || typeof snap !== 'object') return;

    // Header pill
    const pill = document.querySelector('[data-telemetry-pill]');
    const pillTxt = document.querySelector('[data-telemetry-pill-text]');
    if (pill) {
      swapStateClass(pill, snap.state);
      pill.dataset.telemetryState = snap.state;
    }
    if (pillTxt) {
      const cpuCur = snap.metrics && snap.metrics.cpu ? snap.metrics.cpu.current : null;
      if (snap.state === 'connected' && typeof cpuCur === 'number') {
        pillTxt.textContent = `Live · ${cpuCur.toFixed(0)}% CPU`;
      } else {
        pillTxt.textContent = `Grafana: ${stateLabel(snap.state)}`;
      }
    }

    // Footer & shared sync timestamps
    const footerState = document.querySelector('[data-footer-state]');
    if (footerState) {
      swapStateClass(footerState, snap.state);
      footerState.textContent = `Grafana: ${stateLabel(snap.state)}`;
    }
    document.querySelectorAll('[data-last-sync]').forEach((el) => {
      el.textContent = formatIsoShort(snap.last_synced_at);
    });

    // Home panel state
    const homeBadge = document.querySelector('[data-home-state-badge]');
    const homeLabel = document.querySelector('[data-home-state-label]');
    const homeDs = document.querySelector('[data-home-ds]');
    if (homeBadge) swapStateClass(homeBadge, snap.state);
    if (homeLabel) homeLabel.textContent = stateLabel(snap.state);
    if (homeDs) {
      homeDs.textContent = snap.active_datasource
        ? `Source: ${snap.active_datasource.name} (${snap.active_datasource.ds_type})`
        : 'Grafana API · awaiting key';
    }

    // Telemetry page header
    const bannerBadge = document.querySelector('[data-telemetry-banner-badge]');
    const bannerLabel = document.querySelector('[data-telemetry-banner-label]');
    if (bannerBadge) swapStateClass(bannerBadge, snap.state);
    if (bannerLabel) bannerLabel.textContent = stateLabel(snap.state);

    const verEl = document.querySelector('[data-grafana-version]');
    if (verEl) {
      verEl.textContent = snap.grafana_version
        ? `v${snap.grafana_version}${snap.org_name ? ` · ${snap.org_name}` : ''}`
        : 'Not connected';
    }

    const activeDsEl = document.querySelector('[data-active-datasource]');
    if (activeDsEl) {
      activeDsEl.textContent = snap.active_datasource
        ? `${snap.active_datasource.name} (${snap.active_datasource.ds_type})`
        : 'Auto-discovery';
    }

    const errBanner = document.querySelector('[data-telemetry-error]');
    if (errBanner) {
      if (snap.last_error) {
        errBanner.textContent = `Sync Error: ${snap.last_error}`;
        errBanner.classList.remove('hidden');
      } else {
        errBanner.textContent = '';
        errBanner.classList.add('hidden');
      }
    }

    // Uptime & counters
    const uptimeVal = snap.metrics ? snap.metrics.uptime_seconds : null;
    document.querySelectorAll('[data-uptime-val]').forEach((el) => {
      if (typeof uptimeVal === 'number') {
        el.textContent = formatUptime(uptimeVal);
      } else if (!el.textContent || el.textContent.trim() === '—') {
        el.textContent = '—';
      }
    });

    const firingAlerts = Array.isArray(snap.alerts)
      ? snap.alerts.filter((a) => String(a.state || '').toLowerCase() !== 'suppressed').length
      : 0;
    document.querySelectorAll('[data-alerts-count]').forEach((el) => {
      el.textContent = el.closest('.panel-footer') ? `${firingAlerts} active` : String(firingAlerts);
    });

    const dbEl = document.querySelector('[data-grafana-db]');
    if (dbEl) dbEl.textContent = snap.grafana_database || '—';
    const dsCountEl = document.querySelector('[data-ds-count]');
    if (dsCountEl && Array.isArray(snap.datasources)) dsCountEl.textContent = String(snap.datasources.length);
    const dashCountEl = document.querySelector('[data-dash-count]');
    if (dashCountEl && Array.isArray(snap.dashboards)) dashCountEl.textContent = String(snap.dashboards.length);

    // Homelab KPI
    const hlLoad = document.querySelector('[data-homelab-load]');
    const hlMem = document.querySelector('[data-homelab-mem]');
    if (hlLoad && snap.metrics && snap.metrics.cpu && typeof snap.metrics.cpu.current === 'number') {
      hlLoad.textContent = `${snap.metrics.cpu.current.toFixed(1)}% CPU`;
    }
    if (hlMem && snap.metrics && snap.metrics.memory && typeof snap.metrics.memory.current === 'number') {
      hlMem.textContent = `Memory ${snap.metrics.memory.current.toFixed(1)}% · Uptime ${formatUptime(uptimeVal)}`;
    }

    // Metric series cards & charts
    if (snap.metrics) {
      for (const key of ['cpu', 'memory', 'disk', 'load', 'network']) {
        const series = snap.metrics[key];
        if (!series) continue;
        document.querySelectorAll(`[data-metric-title="${key}"]`).forEach((el) => {
          if (series.label) el.textContent = series.label;
        });
        document.querySelectorAll(`[data-metric-short-label="${key}"]`).forEach((el) => {
          el.textContent = series.unit === 'guests' ? 'GUESTS' : key.toUpperCase();
        });
        document.querySelectorAll(`[data-metric-value="${key}"]`).forEach((el) => {
          el.textContent = formatMetricValue(series.current, series.unit);
        });
        document.querySelectorAll(`[data-metric-min="${key}"]`).forEach((el) => {
          el.textContent = formatMetricValue(series.min, series.unit);
        });
        document.querySelectorAll(`[data-metric-avg="${key}"]`).forEach((el) => {
          el.textContent = formatMetricValue(series.avg, series.unit);
        });
        document.querySelectorAll(`[data-metric-max="${key}"]`).forEach((el) => {
          el.textContent = formatMetricValue(series.max, series.unit);
        });
        document.querySelectorAll(`svg[data-sparkline="${key}"]`).forEach((svg) => {
          updateSvgElement(svg, series.points, 4, 276, 8, 56, null, null);
        });
        document.querySelectorAll(`svg[data-chart="${key}"]`).forEach((svg) => {
          const empty = document.querySelector(`[data-chart-empty="${key}"]`);
          updateSvgElement(svg, series.points, 8, 472, 14, 134, empty, series.query_error || null);
        });
      }
    }

    // Service probe badges across Home & Homelab
    if (Array.isArray(snap.service_probes)) {
      const known = snap.service_probes.filter((p) => typeof p.up === 'boolean');
      const upCount = known.filter((p) => p.up === true).length;
      document.querySelectorAll('[data-probes-summary]').forEach((el) => {
        el.textContent = known.length > 0 ? `${upCount}/${known.length} up` : `${snap.service_probes.length} services`;
      });

      for (const probe of snap.service_probes) {
        document.querySelectorAll(`[data-service-badge="${probe.service_id}"]`).forEach((badge) => {
          badge.classList.remove('probe-up', 'probe-down', 'probe-unknown');
          if (probe.up === true) {
            badge.classList.add('probe-up');
            badge.textContent = 'UP';
          } else if (probe.up === false) {
            badge.classList.add('probe-down');
            badge.textContent = 'DOWN';
          } else {
            badge.classList.add('probe-unknown');
            badge.textContent = 'UNMONITORED';
          }
        });
        document.querySelectorAll(`[data-service-target="${probe.service_id}"]`).forEach((t) => {
          if (probe.target) {
            t.textContent = `Probe target: ${probe.target}`;
            t.classList.remove('hidden');
          }
        });
      }

      const probesBody = document.querySelector('[data-probes-body]');
      if (probesBody) {
        probesBody.replaceChildren();
        for (const probe of snap.service_probes) {
          const tr = document.createElement('tr');
          const tdName = document.createElement('td');
          tdName.className = 'mono';
          tdName.textContent = probe.name;

          const tdSource = document.createElement('td');
          tdSource.className = 'mono muted-text';
          tdSource.textContent = probe.source;

          const tdTarget = document.createElement('td');
          tdTarget.className = 'mono muted-text';
          tdTarget.textContent = probe.target || '—';

          const tdStatus = document.createElement('td');
          const badge = document.createElement('span');
          const cls = probe.up === true ? 'probe-up' : probe.up === false ? 'probe-down' : 'probe-unknown';
          badge.className = `probe-badge mono ${cls}`;
          badge.textContent = probe.up === true ? 'UP' : probe.up === false ? 'DOWN' : 'UNMONITORED';
          tdStatus.appendChild(badge);

          tr.append(tdName, tdSource, tdTarget, tdStatus);
          probesBody.appendChild(tr);
        }
      }
    }

    // Alerts list on Telemetry console
    const alertsBody = document.querySelector('[data-alerts-body]');
    if (alertsBody && Array.isArray(snap.alerts)) {
      alertsBody.replaceChildren();
      if (snap.alerts.length === 0) {
        const wrap = document.createElement('div');
        wrap.className = 'empty-note';
        const p = document.createElement('p');
        p.className = 'card-desc';
        p.textContent = snap.configured
          ? 'No alerts reported by Grafana Alertmanager.'
          : 'Waiting for Grafana API connection.';
        wrap.appendChild(p);
        alertsBody.appendChild(wrap);
      } else {
        const ul = document.createElement('ul');
        ul.className = 'alert-list';
        for (const alert of snap.alerts) {
          const li = document.createElement('li');
          li.className = 'alert-row';
          const top = document.createElement('div');
          top.className = 'alert-top';
          const name = document.createElement('strong');
          name.className = 'mono';
          name.textContent = alert.name;
          const sev = document.createElement('span');
          sev.className = 'mono alert-sev';
          sev.textContent = `${alert.severity} · ${alert.state}`;
          top.append(name, sev);
          li.appendChild(top);
          if (alert.summary) {
            const sum = document.createElement('p');
            sum.className = 'card-desc';
            sum.textContent = alert.summary;
            li.appendChild(sum);
          }
          ul.appendChild(li);
        }
        alertsBody.appendChild(ul);
      }
    }

    // Dashboards list on Telemetry console
    const dashBody = document.querySelector('[data-dashboards-body]');
    if (dashBody && Array.isArray(snap.dashboards)) {
      dashBody.replaceChildren();
      if (snap.dashboards.length === 0) {
        const p = document.createElement('p');
        p.className = 'card-desc';
        p.textContent = 'No dashboards indexed yet.';
        dashBody.appendChild(p);
      } else {
        const ul = document.createElement('ul');
        ul.className = 'dash-list';
        for (const d of snap.dashboards.slice(0, 10)) {
          const li = document.createElement('li');
          li.className = 'dash-row';
          const rowTop = document.createElement('div');
          const safeUrl =
            typeof d.url === 'string' &&
            (d.url.startsWith('https://') || d.url.startsWith('http://'))
              ? d.url
              : null;
          if (safeUrl) {
            const a = document.createElement('a');
            a.href = safeUrl;
            a.target = '_blank';
            a.rel = 'noreferrer';
            a.className = 'mono table-title-link';
            a.textContent = d.title;
            rowTop.appendChild(a);
          } else {
            const span = document.createElement('span');
            span.className = 'mono';
            span.textContent = d.title;
            rowTop.appendChild(span);
          }
          if (d.folder_title) {
            const folder = document.createElement('span');
            folder.className = 'mono muted-text';
            folder.style.marginLeft = '0.5rem';
            folder.style.fontSize = '0.75rem';
            folder.textContent = `[${d.folder_title}]`;
            rowTop.appendChild(folder);
          }
          li.appendChild(rowTop);
          if (Array.isArray(d.tags) && d.tags.length > 0) {
            const tags = document.createElement('div');
            tags.className = 'tag-row';
            tags.style.marginTop = '0.35rem';
            for (const t of d.tags) {
              const tag = document.createElement('span');
              tag.className = 'tag';
              tag.textContent = t;
              tags.appendChild(tag);
            }
            li.appendChild(tags);
          }
          ul.appendChild(li);
        }
        dashBody.appendChild(ul);
      }
    }

    // Annotations list on Telemetry console
    const annBody = document.querySelector('[data-annotations-body]');
    if (annBody && Array.isArray(snap.annotations)) {
      annBody.replaceChildren();
      if (snap.annotations.length === 0) {
        const p = document.createElement('p');
        p.className = 'card-desc';
        p.textContent = 'No Grafana annotations available.';
        annBody.appendChild(p);
      } else {
        const ul = document.createElement('ul');
        ul.className = 'annotation-list';
        for (const ann of snap.annotations) {
          const li = document.createElement('li');
          li.className = 'annotation-item';
          const ts = document.createElement('div');
          ts.className = 'mono muted-text';
          ts.style.fontSize = '0.72rem';
          ts.textContent = formatIsoShort(ann.time_ms);
          const txt = document.createElement('p');
          txt.className = 'annotation-text';
          txt.textContent = ann.text;
          li.append(ts, txt);
          if (Array.isArray(ann.tags) && ann.tags.length > 0) {
            const tags = document.createElement('div');
            tags.className = 'tag-row';
            for (const t of ann.tags) {
              const tag = document.createElement('span');
              tag.className = 'tag';
              tag.textContent = t;
              tags.appendChild(tag);
            }
            li.appendChild(tags);
          }
          ul.appendChild(li);
        }
        annBody.appendChild(ul);
      }
    }

    // Diagnostics list on Telemetry console
    const diagList = document.querySelector('[data-diagnostics-list]');
    if (diagList && Array.isArray(snap.diagnostics)) {
      diagList.replaceChildren();
      for (const msg of snap.diagnostics) {
        const li = document.createElement('li');
        li.className = 'mono';
        li.style.fontSize = '0.78rem';
        li.textContent = msg;
        diagList.appendChild(li);
      }
    }
  }

  // Subscribe to SSE telemetry stream
  const sseIndicator = document.getElementById('sse-indicator');
  if (typeof EventSource !== 'undefined') {
    const es = new EventSource('/api/telemetry/stream');
    es.addEventListener('open', () => {
      if (sseIndicator) sseIndicator.textContent = 'SSE: live';
    });
    es.addEventListener('telemetry', (evt) => {
      if (sseIndicator) sseIndicator.textContent = 'SSE: live';
      try {
        applyTelemetrySnapshot(JSON.parse(evt.data));
      } catch {
        // Ignore malformed frame
      }
    });
    es.addEventListener('error', () => {
      if (sseIndicator) sseIndicator.textContent = 'SSE: reconnecting…';
    });
  }

  // Command Palette
  const palette = document.getElementById('cmd-palette');
  const openPaletteBtn = document.getElementById('open-cmd-palette');
  const closePaletteBtn = document.getElementById('cmd-close');
  const cmdInput = document.getElementById('cmd-input');
  const cmdResults = document.getElementById('cmd-results');

  let activeItems = [];
  let selectedIdx = 0;
  let searchTimer = null;

  function renderPaletteItems(items) {
    if (!cmdResults) return;
    cmdResults.replaceChildren();
    activeItems = items;
    selectedIdx = 0;

    if (items.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'card-desc';
      empty.style.padding = '1rem';
      empty.textContent = 'No matching pages, projects, services, or Grafana items.';
      cmdResults.appendChild(empty);
      return;
    }

    items.forEach((item, idx) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = idx === 0 ? 'cmd-item selected' : 'cmd-item';

      const left = document.createElement('div');
      const title = document.createElement('div');
      title.className = 'cmd-item-title';
      title.textContent = item.title;
      const sub = document.createElement('div');
      sub.className = 'cmd-item-sub';
      sub.textContent = item.subtitle;
      left.append(title, sub);

      const kind = document.createElement('span');
      kind.className = 'cmd-item-kind';
      kind.textContent = item.kind;

      btn.append(left, kind);
      btn.addEventListener('click', () => triggerItem(item));
      cmdResults.appendChild(btn);
    });
  }

  function updatePaletteSelection() {
    if (!cmdResults) return;
    const rows = cmdResults.querySelectorAll('.cmd-item');
    rows.forEach((row, idx) => {
      row.classList.toggle('selected', idx === selectedIdx);
    });
  }

  function triggerItem(item) {
    closePalette();
    if (typeof item.action === 'function') {
      item.action();
    } else if (item.href) {
      if (item.external) {
        window.open(item.href, '_blank', 'noopener,noreferrer');
      } else {
        window.location.href = item.href;
      }
    }
  }

  async function runPaletteSearch(query) {
    const localActions = [
      {
        title: 'Toggle Color Theme',
        subtitle: 'Switch between Warm Obsidian and Archival Paper',
        kind: 'action',
        action: toggleTheme,
      },
    ];

    try {
      const res = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
      const data = await res.json();
      const serverItems = Array.isArray(data.results) ? data.results : [];
      const qLower = query.trim().toLowerCase();
      const matchedActions = qLower
        ? localActions.filter((a) => a.title.toLowerCase().includes(qLower))
        : localActions;
      renderPaletteItems([...serverItems, ...matchedActions]);
    } catch {
      renderPaletteItems(localActions);
    }
  }

  function openPalette() {
    if (!palette || !cmdInput) return;
    palette.classList.remove('hidden');
    cmdInput.value = '';
    cmdInput.focus();
    runPaletteSearch('');
  }

  function closePalette() {
    if (!palette) return;
    palette.classList.add('hidden');
  }

  if (openPaletteBtn) openPaletteBtn.addEventListener('click', openPalette);
  if (closePaletteBtn) closePaletteBtn.addEventListener('click', closePalette);
  if (palette) {
    palette.addEventListener('click', (e) => {
      if (e.target === palette) closePalette();
    });
  }

  if (cmdInput) {
    cmdInput.addEventListener('input', () => {
      clearTimeout(searchTimer);
      searchTimer = setTimeout(() => runPaletteSearch(cmdInput.value), 120);
    });
    cmdInput.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowDown' && activeItems.length > 0) {
        e.preventDefault();
        selectedIdx = (selectedIdx + 1) % activeItems.length;
        updatePaletteSelection();
      } else if (e.key === 'ArrowUp' && activeItems.length > 0) {
        e.preventDefault();
        selectedIdx = (selectedIdx - 1 + activeItems.length) % activeItems.length;
        updatePaletteSelection();
      } else if (e.key === 'Enter' && activeItems[selectedIdx]) {
        e.preventDefault();
        triggerItem(activeItems[selectedIdx]);
      } else if (e.key === 'Escape') {
        closePalette();
      }
    });
  }

  document.addEventListener('keydown', (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      if (palette && palette.classList.contains('hidden')) {
        openPalette();
      } else {
        closePalette();
      }
    } else if (
      e.key === '/' &&
      palette &&
      palette.classList.contains('hidden') &&
      !['INPUT', 'TEXTAREA', 'SELECT'].includes(document.activeElement?.tagName || '')
    ) {
      e.preventDefault();
      openPalette();
    } else if (e.key === 'Escape' && palette && !palette.classList.contains('hidden')) {
      closePalette();
    }
  });
})();
