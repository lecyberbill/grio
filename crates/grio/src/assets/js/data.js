  /* ---------- data, plotting, metrics, code & file components ---------- */

  const LANGS = {
    rust: { kw: false, s: /"(?:[^"\\]|\\.)*"/g, c: /(?:\/\/.*$|\/\*[\s\S]*?\*\/)/gm, n: /\b\d[\d_]*(?:\.\d+)?(?:e[+-]?\d+)?\b/g },
    python: { kw: /\b(?:def|class|return|if|elif|else|for|while|import|from|as|with|try|except|finally|raise|lambda|yield|global|nonlocal|pass|break|continue|None|True|False|async|await|self|print|in|is|not|and|or)\b/g, s: /"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'/g, c: /#[^\n]*/g, n: /\b\d[\d_]*(?:\.\d+)?(?:e[+-]?\d+)?\b/g },
    javascript: { kw: /\b(?:function|return|const|let|var|if|else|for|while|class|new|import|export|from|async|await|try|catch|finally|throw|switch|case|default|break|continue|typeof|instanceof|this|super|null|undefined|true|false|of|in|do)\b/g, s: /"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`/g, c: /(?:\/\/.*$|\/\*[\s\S]*?\*\/)/gm, n: /\b\d[\d_]*(?:\.\d+)?(?:e[+-]?\d+)?\b/g },
    json: { kw: false, s: /"(?:[^"\\]|\\.)*"/g, c: false, n: /-?\b\d[\d_]*(?:\.\d+)?(?:e[+-]?\d+)?\b/g },
    markdown: { kw: /(?:^#{1,6}\s)|(?:^\s*[-*+]\s)|(?:^\s*\d+\.\s)|(?:\*\*[^*\n]+\*\*)|(?:`[^`\n]+`)|(?:\[[^\]]+\]\([^)]*\))/gm, s: false, c: false, n: false },
  };
  const HIGHLIGHT_RUST_KW = /\b(?:fn|let|mut|pub|struct|enum|impl|trait|match|if|else|for|while|loop|return|use|mod|const|static|async|await|move|ref|where|self|Self|Some|None|Ok|Err|crate|super|dyn|unsafe|type|as|in|break|continue|Box|String|Vec)\b/g;
  LANGS.rust.kw = HIGHLIGHT_RUST_KW;

  function highlight(src, lang) {
    const cfg = (lang && LANGS[lang]) || LANGS.rust;
    const alts = [];
    const kinds = [];
    if (cfg.c) { alts.push(cfg.c.source); kinds.push('c'); }
    if (cfg.s) { alts.push(cfg.s.source); kinds.push('s'); }
    if (cfg.n) { alts.push(cfg.n.source); kinds.push('n'); }
    if (cfg.kw) { alts.push(cfg.kw.source); kinds.push('k'); }
    const rx = new RegExp('(' + kinds.map((_, i) => '(' + alts[i] + ')').join('|') + ')', 'gm');
    const s = String(src == null ? '' : src);
    let out = '';
    let last = 0;
    let m;
    while ((m = rx.exec(s)) !== null) {
      out += esc(s.slice(last, m.index));
      let kind = 'k';
      for (let i = 0; i < kinds.length; i++) {
        if (m[2 + i] !== undefined) { kind = kinds[i]; break; }
      }
      out += '<span class="tok-' + kind + '">' + esc(m[0]) + '</span>';
      last = m.index + m[0].length;
      if (m[0].length === 0) rx.lastIndex++;
    }
    out += esc(s.slice(last));
    return out;
  }

  function fmtBytes(n) {
    if (n < 1024) return n + ' o';
    if (n < 1048576) return (n / 1024).toFixed(1) + ' Ko';
    return (n / 1048576).toFixed(1) + ' Mo';
  }

  function niceStep(rough) {
    const mag = Math.pow(10, Math.floor(Math.log10(rough || 1)));
    for (const m of [1, 2, 2.5, 5, 10]) { if (rough <= m * mag) return m * mag; }
    return 10 * mag;
  }

  function niceTicks(min, max, count) {
    const step = niceStep((max - min) / count);
    const out = [];
    for (let v = Math.ceil(min / step) * step; v <= max + step * 1e-6; v += step) out.push(v);
    return out;
  }

  function fmtTick(v) {
    const r = Math.round(v * 100) / 100;
    return Math.abs(r - Math.round(r)) < 1e-9 ? String(Math.round(r)) : String(r);
  }

  function drawPlot(spec, p) {
    const type = (spec && spec.variant) || p.variant || 'line';
    const W = p.width || 480;
    const H = p.height || 280;
    const pad = { l: 60, r: 14, t: (p.title ? 26 : 10), b: (p.xlabel ? 44 : 26) };
    const pw = W - pad.l - pad.r;
    const ph = H - pad.t - pad.b;
    const labels = (spec && Array.isArray(spec.labels)) ? spec.labels.map(String) : [];
    const series = (spec && Array.isArray(spec.series)) ? spec.series : [];
    const cols = (p.colors && p.colors.length) ? p.colors : ['#6366f1'];

    let minV = 0, maxV = 1, maxIdx = 0, hasV = false;
    series.forEach((s) => {
      if (type === 'scatter' && Array.isArray(s.points)) {
        s.points.forEach((pt) => {
          if (Array.isArray(pt) && pt.length >= 2) {
            hasV = true;
            minV = Math.min(minV, Number(pt[1]));
            maxV = Math.max(maxV, Number(pt[1]));
            maxIdx = Math.max(maxIdx, Number(pt[0]));
          }
        });
      } else if (Array.isArray(s.data)) {
        s.data.forEach((v) => {
          const f = Number(v);
          if (!isNaN(f)) { hasV = true; minV = Math.min(minV, f); maxV = Math.max(maxV, f); }
        });
        maxIdx = Math.max(maxIdx, (s.data.length || 1) - 1);
      }
    });
    if (!hasV) { minV = 0; maxV = 1; }
    if (minV === maxV) { minV -= 1; maxV += 1; }
    const yMin = minV >= 0 ? 0 : minV;
    let yMax = maxV + (maxV - yMin) * 0.08;
    if (yMax - yMin < 1e-9) yMax = yMin + 1;
    const span = (yMax - yMin) || 1;
    const ticks = niceTicks(yMin, yMax, 4);
    const nX = Math.max(labels.length, maxIdx + 1, 1);
    const slotW = pw / nX;
    const X = (i) => type === 'bar' ? (pad.l + (i + 0.5) * slotW) : (pad.l + (nX > 1 ? pw * (i / (nX - 1)) : pw / 2));
    const Y = (v) => pad.t + ph - ph * ((Number(v) - yMin) / span);

    let svg = '<svg class="mg-plot-svg" viewBox="0 0 ' + W + ' ' + H + '" xmlns="http://www.w3.org/2000/svg">';
    ticks.forEach((v) => {
      if (v < yMin - 1e-9 || v > yMax + 1e-9) return;
      const y = Y(v);
      svg += '<line x1="' + pad.l + '" y1="' + y + '" x2="' + (W - pad.r) + '" y2="' + y + '" class="mg-plot-grid"/>';
      svg += '<text x="' + (pad.l - 8) + '" y="' + (y + 4) + '" class="mg-plot-tick" text-anchor="end">' + fmtTick(v) + '</text>';
    });
    if (labels.length <= 12 || true) {
      const step = labels.length > 12 ? Math.ceil(labels.length / 12) : 1;
      labels.forEach((lab, i) => {
        if (i % step === 0) {
          svg += '<text x="' + X(i) + '" y="' + (pad.t + ph + 16) + '" class="mg-plot-tick mg-plot-x" text-anchor="middle">' + esc(lab) + '</text>';
        }
      });
    }
    svg += '<line x1="' + pad.l + '" y1="' + pad.t + '" x2="' + pad.l + '" y2="' + (pad.t + ph) + '" class="mg-plot-axis"/>';
    svg += '<line x1="' + pad.l + '" y1="' + (pad.t + ph) + '" x2="' + (W - pad.r) + '" y2="' + (pad.t + ph) + '" class="mg-plot-axis"/>';
    if (p.title) svg += '<text x="' + (W / 2) + '" y="16" class="mg-plot-title" text-anchor="middle">' + esc(p.title) + '</text>';

    const numSeries = series.length || 1;
    series.forEach((s, si) => {
      const color = s.color || cols[si % cols.length];
      if (type === 'bar') {
        const totalBarGroupWidth = slotW * 0.75;
        const bw = Math.max(2, totalBarGroupWidth / numSeries - 2);
        const groupStart = (i) => X(i) - totalBarGroupWidth / 2;
        (s.data || []).forEach((v, i) => {
          const f = Number(v);
          if (isNaN(f)) return;
          const barX = groupStart(i) + si * (bw + 2);
          const y = Y(f);
          const barH = Math.max(1, (pad.t + ph) - y);
          svg += '<rect x="' + barX + '" y="' + y + '" width="' + bw + '" height="' + barH + '" fill="' + color + '" rx="2" opacity="0.9"/>';
        });
      } else if (type === 'scatter') {
        (s.points || []).forEach((pt) => {
          if (!Array.isArray(pt) || pt.length < 2) return;
          svg += '<circle cx="' + X(Number(pt[0])) + '" cy="' + Y(Number(pt[1])) + '" r="4" fill="' + color + '"/>';
        });
      } else {
        let pts = '';
        (s.data || []).forEach((v, i) => {
          const f = Number(v);
          if (isNaN(f)) return;
          pts += (pts ? ' ' : '') + X(i) + ',' + Y(f);
        });
        if (pts) {
          svg += '<polyline points="' + pts + '" fill="none" stroke="' + color + '" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/>';
          (s.data || []).forEach((v, i) => {
            const f = Number(v);
            if (isNaN(f)) return;
            svg += '<circle cx="' + X(i) + '" cy="' + Y(f) + '" r="3" fill="' + color + '"/>';
          });
        }
      }
      if (s.name) {
        const ly = pad.t + 8 + si * 15;
        svg += '<rect x="' + (pad.l + 6) + '" y="' + ly + '" width="9" height="9" fill="' + color + '" rx="1.5"/>';
        svg += '<text x="' + (pad.l + 20) + '" y="' + (ly + 9) + '" class="mg-plot-tick">' + esc(s.name) + '</text>';
      }
    });
    if (p.xlabel) svg += '<text x="' + (pad.l + pw / 2) + '" y="' + (H - 4) + '" class="mg-plot-tick mg-plot-label-x" text-anchor="middle" font-weight="600">' + esc(p.xlabel) + '</text>';
    if (p.ylabel) svg += '<text x="16" y="' + (pad.t + ph / 2) + '" class="mg-plot-tick" text-anchor="middle" transform="rotate(-90 16 ' + (pad.t + ph / 2) + ')">' + esc(p.ylabel) + '</text>';
    return svg + '</svg>';
  }

  register('dataframe', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-df';
      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);
      const view = document.createElement('div');
      view.className = 'mg-df-view';
      wrap.appendChild(view);
      c.el.appendChild(wrap);

      let headers = Array.isArray(p.headers) && p.headers.length
        ? p.headers.map(String)
        : (Array.isArray(p.value) && p.value.length ? p.value[0].map((_, i) => 'col' + (i + 1)) : []);
      let rows = Array.isArray(p.value) ? p.value.map((r) => (Array.isArray(r) ? r.slice() : [])) : [];
      const editable = p.interactive !== false;
      const sortable = p.sortable !== false;
      let sort = null;

      const sortRows = () => {
        if (!sort) return;
        const { i, dir } = sort;
        rows.sort((a, b) => {
          const va = a[i] == null ? '' : a[i];
          const vb = b[i] == null ? '' : b[i];
          const na = Number(va), nb = Number(vb);
          const cmp = (va !== '' && vb !== '' && !isNaN(na) && !isNaN(nb))
            ? na - nb
            : String(va).localeCompare(String(vb), undefined, { numeric: true, sensitivity: 'base' });
          return cmp * dir;
        });
      };

      const build = () => {
        const cols = Math.max(1, headers.length, ...rows.map((r) => r.length));
        let html = '<table class="mg-table"><thead><tr>';
        for (let i = 0; i < cols; i++) {
          const act = sort && sort.i === i ? ' mg-df-sort-' + (sort.dir > 0 ? 'asc' : 'desc') : '';
          const mark = sort && sort.i === i ? (sort.dir > 0 ? ' ▲' : ' ▼') : '';
          html += '<th' + (sortable ? ' class="mg-df-sortable' + act + '" data-s="' + i + '"' : '') + '>' +
            esc(i < headers.length ? headers[i] : 'col' + (i + 1)) + mark + '</th>';
        }
        if (editable && p.addable) html += '<th class="mg-df-ops"></th>';
        html += '</tr></thead><tbody>';
        rows.forEach((r) => {
          html += '<tr>';
          for (let i = 0; i < cols; i++) {
            const v = r[i] == null ? '' : (Array.isArray(r[i]) || (r[i] && typeof r[i] === 'object') ? JSON.stringify(r[i]) : String(r[i]));
            html += editable
              ? '<td><input class="mg-cell" data-c="' + i + '" value="' + esc(v) + '"></td>'
              : '<td>' + esc(v) + '</td>';
          }
          if (editable && p.addable) html += '<td><button type="button" class="mg-btn mg-ico mg-btn-secondary" title="Supprimer">x</button></td>';
          html += '</tr>';
        });
        html += '</tbody></table>';
        view.innerHTML = html;
        if (sortable) {
          view.querySelectorAll('th[data-s]').forEach((th) => {
            th.addEventListener('click', () => {
              const i = +th.dataset.s;
              if (sort && sort.i === i) sort.dir = -sort.dir; else sort = { i, dir: 1 };
              sortRows();
              build();
              commit();
            });
          });
        }
        const redraw = () => { build(); commit(); };
        if (editable) {
          const add = document.createElement('button');
          add.type = 'button';
          add.className = 'mg-btn mg-btn-secondary';
          add.textContent = '+ ligne';
          add.addEventListener('click', () => { rows.push(Array(cols).fill('')); redraw(); });
          view.appendChild(add);
          view.querySelectorAll('input.mg-cell').forEach((inp) => {
            inp.addEventListener('change', () => {
              const r = +inp.closest('tr').rowIndex - 1;
              const i = +inp.dataset.c;
              while (rows.length <= r) rows.push([]);
              rows[r][i] = inp.value;
              commit();
            });
          });
          view.querySelectorAll('.mg-df-ops button').forEach((b) => {
            b.addEventListener('click', () => {
              rows.splice(+b.closest('tr').rowIndex - 1, 1);
              redraw();
            });
          });
        }
      };
      const commit = () => {
        if (p.interactive === false) return;
        emit(c, 'change', { headers, data: rows });
      };
      build();

      c.getValue = () => ({ headers, data: rows });
      c.apply = (patch) => {
        if (patch.value != null) {
          const v = patch.value;
          if (Array.isArray(v.headers)) {
            headers = v.headers.map(String);
          } else if (Array.isArray(v) ) {
            rows = v.map((r) => (Array.isArray(r) ? r.slice() : []));
            headers = rows[0] ? rows[0].map((_, i) => 'col' + (i + 1)) : headers;
          } else if (v.data != null) {
            headers = Array.isArray(v.headers) ? v.headers.map(String) : headers;
            rows = Array.isArray(v.data) ? v.data.map((r) => (Array.isArray(r) ? r.slice() : [])) : rows;
          }
          if (v.data != null) rows = Array.isArray(v.data) ? v.data.map((r) => (Array.isArray(r) ? r.slice() : [])) : rows;
          build();
        }
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('plot', {
    mount(c) {
      const p = c.props;
      const holder = document.createElement('div');
      holder.className = 'mg-plot';
      holder.innerHTML = '<div class="mg-label"><span>' + esc(p.label || c.id) + '</span></div>';
      const box = document.createElement('div');
      box.className = 'mg-plot-box';
      holder.appendChild(box);
      box.innerHTML = drawPlot(p.value || {}, p);
      c.el.appendChild(holder);
      c.apply = (patch) => {
        if (patch.value != null && typeof patch.value === 'object') box.innerHTML = drawPlot(patch.value, p);
        if (patch.label != null) { const s = holder.querySelector('.mg-label span'); if (s) s.textContent = patch.label; }
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('metric', {
    mount(c) {
      const p = c.props;
      const holder = document.createElement('div');
      holder.className = 'mg-metric';
      
      const render = (props) => {
        let deltaHtml = '';
        if (props.delta) {
          const cls = props.delta_color || (props.delta.startsWith('-') ? 'neg' : 'pos');
          deltaHtml = `<div class="mg-metric-delta ${cls}"><span>${esc(props.delta)}</span></div>`;
        }
        const unitHtml = props.unit ? `<span class="mg-metric-unit">${esc(props.unit)}</span>` : '';
        holder.innerHTML = `
          <div class="mg-metric-label">${esc(props.label || c.id)}</div>
          <div class="mg-metric-main">
            <div class="mg-metric-value">${esc(props.value ?? '')}</div>
            ${unitHtml}
          </div>
          ${deltaHtml}
        `;
      };

      render(p);
      c.el.appendChild(holder);

      c.apply = (patch) => {
        if (typeof patch.value === 'object' && patch.value !== null) {
          Object.assign(p, patch.value);
        } else if (patch.value != null) {
          p.value = patch.value;
        }
        if (patch.delta != null) p.delta = patch.delta;
        if (patch.delta_color != null) p.delta_color = patch.delta_color;
        if (patch.unit != null) p.unit = patch.unit;
        if (patch.label != null) p.label = patch.label;
        render(p);
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('label', {
    mount(c) {
      const p = c.props;
      const card = document.createElement('div');
      card.className = 'mg-label-card';
      card.innerHTML =
        '<div class="mg-card-label">' + esc(p.label || c.id) + '</div>' +
        '<div class="mg-label-value mg-label-var-' + esc(p.variant || 'normal') + '"></div>';
      c.el.appendChild(card);
      const val = card.querySelector('.mg-label-value');
      val.style.fontSize = (p.size || 26) + 'px';
      val.textContent = p.value ?? '';
      c.apply = (patch) => {
        if (patch.value != null) val.textContent = patch.value;
        if (patch.variant != null) val.className = 'mg-label-value mg-label-var-' + esc(patch.variant);
        if (patch.label != null) {
          const l = card.querySelector('.mg-card-label');
          if (l) l.textContent = patch.label;
        }
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('json', {
    mount(c) {
      const p = c.props;
      const interactive = p.interactive !== false;
      const stringify = (v) => (v == null ? '' : JSON.stringify(v, null, 2));
      const box = document.createElement('div');
      box.className = 'mg-json';
      box.innerHTML = makeLabel(p, c) +
        (interactive ? '<textarea class="mg-input mg-json-text" rows="10" spellcheck="false"></textarea>'
                     : '<pre class="mg-json-view"></pre>') +
        (interactive ? '<div class="mg-hint"></div>' : '');
      c.el.appendChild(box);
      const text = box.querySelector('.mg-json-text') || box.querySelector('.mg-json-view');
      const hint = box.querySelector('.mg-hint');
      const show = (v) => {
        if (interactive) { text.value = stringify(v); } else { text.textContent = stringify(v); }
      };
      const mark = (ok) => {
        lastOk = ok;
        text.classList.toggle('mg-json-ok', ok);
        text.classList.toggle('mg-json-bad', !ok);
        if (hint) hint.textContent = ok ? t('json_valid') : t('json_invalid');
      };
      let lastOk = true;
      onI18n(() => mark(lastOk));
      if (interactive) {
        show(p.value ?? null);
        mark(true);
        text.addEventListener('input', () => {
          const raw = text.value.trim();
          if (!raw) { mark(true); return; }
          try { const v = JSON.parse(raw); mark(true); emit(c, 'change', v); }
          catch { mark(false); }
        });
        c.getValue = () => { try { return JSON.parse(text.value.trim() || 'null'); } catch { return null; } };
        c.apply = (patch) => {
          if (patch.value != null) {
            let v = patch.value;
            if (typeof v === 'string' && v.trim()) { try { v = JSON.parse(v); } catch { v = patch.value; } }
            if (text.value.trim()) {
              try {
                const cur = JSON.parse(text.value.trim());
                if (JSON.stringify(cur) === JSON.stringify(v)) return;
              } catch { /* ignore */ }
            }
            show(v); mark(true);
          }
        };
      } else {
        show(p.value ?? null);
        c.apply = (patch) => {
          if (patch.value != null) {
            let v = patch.value;
            if (typeof v === 'string') { try { v = JSON.parse(v); } catch { /* ignore */ } }
            show(v);
          }
        };
      }
    }
  });

  register('file', {
    mount(c) {
      const p = c.props;
      const interactive = p.interactive !== false;
      const types = p.types || [];
      const maxSize = p.max_size || 0;
      const list = [];
      const box = document.createElement('div');
      box.className = 'mg-file' + (interactive ? ' mg-file-drop' : '');
      c.el.appendChild(box);
      box.insertAdjacentHTML('beforeend', makeLabel(p, c));
      const input = document.createElement('input');
      input.type = 'file';
      input.hidden = true;
      if (p.multiple !== false) input.multiple = true;
      if (types.length) input.accept = types.join(',');
      box.appendChild(input);
      const items = document.createElement('div');
      items.className = 'mg-file-list';
      box.appendChild(items);
      const render = () => {
        items.innerHTML = list.map((f, i) =>
          '<div class="mg-file-item">' +
            '<span class="mg-file-item-name">' + esc(f.name) + '</span>' +
            '<span class="mg-file-item-size">' + fmtBytes(f.size) + '</span>' +
            (interactive ? '<button type="button" class="mg-btn mg-btn-secondary mg-file-del" data-i="' + i + '" title="' + t('file_remove') + '">×</button>' : '') +
          '</div>').join('');
        items.querySelectorAll('.mg-file-del').forEach((b) => b.addEventListener('click', () => {
          list.splice(+(b.dataset.i), 1);
          render();
          emit(c, 'change', list.slice());
        }));
      };
      const emitList = () => emit(c, 'change', list.slice());
      if (interactive) {
        const dz = document.createElement('div');
        dz.className = 'mg-file-dropzone';
        dz.innerHTML = '<span class="mg-file-icon">📁</span><div class="mg-file-drop-info"><span class="mg-file-drop-text"></span><span class="mg-file-drop-sub"></span></div>';
        box.appendChild(dz);
        const dropText = dz.querySelector('.mg-file-drop-text');
        const dropSub = dz.querySelector('.mg-file-drop-sub');
        const setDropText = () => {
          dropText.textContent = t('file_drop', 'Click or drop files here to upload');
          dropSub.textContent = t('file_drop_sub', 'Drag & drop documents or browse');
        };
        setDropText();
        onI18n(setDropText);
        const prog = document.createElement('div');
        prog.className = 'mg-file-progress';
        prog.hidden = true;
        prog.innerHTML = '<div class="mg-file-progress-bar"><i></i></div><span class="mg-file-progress-txt">0%</span>';
        box.appendChild(prog);
        const bar = prog.querySelector('.mg-file-progress-bar');
        const pct = prog.querySelector('.mg-file-progress-txt');
        const addFiles = (files) => {
          const acc = [];
          for (const f of Array.from(files)) {
            if (types.some((t) => t.endsWith('/*') ? f.type.startsWith(t.slice(0, -1)) : f.type === t)) {}
            if (types.length && !types.some((tp) => tp.endsWith('/*') ? f.type.startsWith(tp.slice(0, -1)) : f.type === tp)) {
              toast(t('file_type_bad') + ' : ' + f.name, 'error'); continue;
            }
            if (maxSize && f.size > maxSize) {
              toast(t('file_too_big') + ' : ' + f.name, 'error'); continue;
            }
            acc.push(f);
          }
          if (!acc.length) return;
          const total = acc.length;
          let done = 0;
          prog.hidden = false;
          const next = () => {
            const f = acc.shift();
            if (!f) { prog.hidden = true; render(); emitList(); return; }
            const r = new FileReader();
            r.onprogress = (e) => {
              if (e.lengthComputable) {
                const v = Math.round(((done + e.loaded / e.total) / total) * 100);
                pct.textContent = v + '%';
                bar.style.width = v + '%';
              }
            };
            r.onload = () => {
              list.push({ name: f.name, size: f.size, mime: f.type || 'application/octet-stream', data_url: String(r.result) });
              done++;
              pct.textContent = Math.round((done / total) * 100) + '%';
              bar.style.width = Math.round((done / total) * 100) + '%';
              next();
            };
            r.readAsDataURL(f);
          };
          next();
        };
        dz.addEventListener('click', () => input.click());
        dz.addEventListener('dragover', (e) => { e.preventDefault(); dz.classList.add('hover'); });
        dz.addEventListener('dragleave', () => dz.classList.remove('hover'));
        dz.addEventListener('drop', (e) => {
          e.preventDefault();
          dz.classList.remove('hover');
          addFiles(e.dataTransfer.files);
        });
        input.addEventListener('change', () => {
          if (input.files && input.files.length) addFiles(input.files);
          input.value = '';
        });
      }
      c.getValue = () => list.slice();
      c.apply = (patch) => {
        if (patch.visible != null) box.hidden = !patch.visible;
      };
    }
  });

  register('download', {
    mount(c) {
      const p = c.props;
      let filename = p.filename || 'download.bin';
      let labelSrc = p.label || '';
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'mg-btn';
      const renderLabel = () => { b.innerHTML = '⤓ ' + esc(labelSrc || t('download_label')); };
      renderLabel();
      c.el.appendChild(b);
      const resolve = (v) => {
        if (!v) return '';
        if (typeof v === 'string' && v.startsWith('data:')) return v;
        if (typeof v === 'string') return 'data:application/octet-stream;base64,' + v;
        if (v && typeof v === 'object') return 'data:' + (v.mime || 'application/octet-stream') + ';base64,' + (v.b64 || '');
        return '';
      };
      let href = resolve(p.value);
      const refresh = () => { b.disabled = !href; };
      refresh();
      b.addEventListener('click', () => {
        emit(c, 'click', null);
        if (!href) return;
        const a = document.createElement('a');
        a.href = href;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        a.remove();
      });
      onI18n(renderLabel);
      c.apply = (patch) => {
        if (patch.value != null) { href = resolve(patch.value); refresh(); }
        if (patch.filename != null) filename = patch.filename;
        if (patch.label != null) { labelSrc = patch.label; renderLabel(); }
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('code', {
    mount(c) {
      const p = c.props;
      const editable = p.interactive !== false && !p.out;
      const holder = document.createElement('div');
      holder.className = 'mg-code mg-code-theme-' + (p.theme === 'auto' ? 'auto' : p.theme);
      const bar = document.createElement('div');
      bar.className = 'mg-code-bar';
      const t = document.createElement('span');
      t.textContent = p.label || c.id;
      bar.appendChild(t);
      if (p.language) {
        const tag = document.createElement('span');
        tag.className = 'mg-code-lang';
        tag.textContent = p.language;
        bar.appendChild(tag);
      }
      holder.appendChild(bar);
      const editor = document.createElement('div');
      editor.className = 'mg-code-editor';
      const ln = document.createElement('div');
      ln.className = 'mg-code-ln';
      const body = document.createElement('div');
      body.className = 'mg-code-body';
      const pre = document.createElement('pre');
      pre.className = 'mg-code-pre';
      const code = document.createElement('code');
      pre.appendChild(code);
      body.appendChild(pre);
      let ta = null;
      if (editable) {
        ta = document.createElement('textarea');
        ta.className = 'mg-code-ta';
        ta.spellcheck = false;
        ta.wrap = 'off';
        body.appendChild(ta);
      }
      editor.appendChild(ln);
      editor.appendChild(body);
      holder.appendChild(editor);
      c.el.appendChild(holder);

      let value = String(p.value != null ? p.value : '');
      const paint = (v) => {
        let lns = '';
        const n = v.split('\n').length;
        for (let i = 1; i <= n; i++) lns += i + '<br>';
        ln.innerHTML = lns;
        code.innerHTML = highlight(v, p.language || null) + '\n';
      };
      if (editable) {
        ta.value = value;
        const sync = () => {
          pre.scrollTop = ta.scrollTop;
          pre.scrollLeft = ta.scrollLeft;
          ln.style.transform = 'translateY(-' + (ta.scrollTop || 0) + 'px)';
        };
        ta.addEventListener('input', () => {
          value = ta.value;
          paint(value);
          sync();
          if (ta._t) clearTimeout(ta._t);
          ta._t = setTimeout(() => emit(c, 'change', value), 250);
        });
        ta.addEventListener('scroll', sync);
      }
      paint(value);
      c.getValue = () => value;
      c.apply = (patch) => {
        if (patch.value != null && String(patch.value) !== value) {
          value = String(patch.value);
          if (editable) ta.value = value;
          paint(value);
        }
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('explorer', {
    mount(c) {
      const p = c.props;
      const root = String(p.root || '.');
      const pattern = p.pattern ? String(p.pattern) : null;
      const holder = document.createElement('div');
      holder.className = 'mg-explorer';
      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      holder.appendChild(lab);
      const crumb = document.createElement('div');
      crumb.className = 'mg-explorer-crumb';
      holder.appendChild(crumb);
      const list = document.createElement('div');
      list.className = 'mg-explorer-list';
      holder.appendChild(list);
      c.el.appendChild(holder);

      let current = '';
      let selected = p.value ? String(p.value) : '';
      const go = (path) => {
        list.innerHTML = '<span class="mg-explorer-loading">chargement\u2026</span>';
        let url = '/api/explore?root=' + encodeURIComponent(root) + '&path=' + encodeURIComponent(path);
        if (pattern) url += '&pattern=' + encodeURIComponent(pattern);
        fetch(url)
          .then((r) => r.json())
          .then((m) => {
            if (m.t !== 'ok') { toast(m.m || 'erreur', 'error'); return; }
            current = path;
            const segs = path ? path.split('/').filter(Boolean) : [];
            crumb.innerHTML = '';
            const acc = [];
            const home = document.createElement('button');
            home.type = 'button';
            home.className = 'mg-explorer-crumb-btn';
            home.textContent = '\u2302';
            home.title = 'racine';
            home.addEventListener('click', () => go(''));
            crumb.appendChild(home);
            segs.forEach((seg) => {
              acc.push(seg);
              const b = document.createElement('button');
              b.type = 'button';
              b.className = 'mg-explorer-crumb-btn';
              b.textContent = seg;
              b.addEventListener('click', () => go(acc.join('/')));
              crumb.appendChild(b);
            });
            list.innerHTML = '';
            m.dirs.forEach((d) => {
              const row = document.createElement('button');
              row.type = 'button';
              row.className = 'mg-explorer-row mg-explorer-dir';
              row.textContent = '\u25b8 ' + d;
              const rel = (path ? path + '/' : '') + d;
              row.addEventListener('click', () => go(rel));
              list.appendChild(row);
            });
            m.files.forEach((f) => {
              const rel = (path ? path + '/' : '') + f;
              const row = document.createElement('button');
              row.type = 'button';
              row.className = 'mg-explorer-row mg-explorer-file' + (selected === rel ? ' sel' : '');
              row.textContent = f;
              row.addEventListener('click', () => {
                selected = rel;
                list.querySelectorAll('.sel').forEach((el) => el.classList.remove('sel'));
                row.classList.add('sel');
                emit(c, 'change', rel);
              });
              list.appendChild(row);
            });
            if (!m.dirs.length && !m.files.length) {
              list.innerHTML = '<span class="mg-explorer-empty">dossier vide</span>';
            }
          })
          .catch(() => toast('explorateur injoignable', 'error'));
      };
      go(current);
      c.getValue = () => selected;
      c.apply = (patch) => {
        if (patch.value != null) selected = String(patch.value);
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('dataeditor', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-dataeditor-wrap';

      if (p.label) {
        const lab = document.createElement('div');
        lab.className = 'mg-label';
        lab.innerHTML = '<span>' + esc(p.label) + '</span>';
        wrap.appendChild(lab);
      }

      const toolbar = document.createElement('div');
      toolbar.className = 'mg-dataeditor-toolbar';

      const tableContainer = document.createElement('div');
      tableContainer.className = 'mg-dataeditor-container';
      if (p.max_height) {
        tableContainer.style.maxHeight = p.max_height + 'px';
        tableContainer.style.overflowY = 'auto';
      }

      wrap.appendChild(toolbar);
      wrap.appendChild(tableContainer);
      c.el.appendChild(wrap);

      let columns = Array.isArray(p.columns) ? p.columns : [];
      let rows = Array.isArray(p.data) ? p.data.map((r) => (Array.isArray(r) ? r.slice() : [])) : [];
      let sortCol = null;
      let sortDir = 1;
      let searchQuery = '';
      const ROW_HEIGHT = 38;
      const BUFFER_COUNT = 15;

      const getColDef = (idx) => columns[idx] || { id: 'col_' + idx, label: 'Col ' + (idx + 1), type: 'text', editable: true };

      const getFilteredRows = () => {
        if (!searchQuery) return rows;
        const q = searchQuery.toLowerCase();
        return rows.filter((row) => {
          return row.some((val) => val != null && String(val).toLowerCase().includes(q));
        });
      };

      const sortRows = () => {
        if (sortCol === null) return;
        const colDef = getColDef(sortCol);
        rows.sort((a, b) => {
          const va = a[sortCol] != null ? a[sortCol] : '';
          const vb = b[sortCol] != null ? b[sortCol] : '';
          if (colDef.type === 'number') {
            return (Number(va) - Number(vb)) * sortDir;
          }
          if (colDef.type === 'boolean') {
            return ((va === true ? 1 : 0) - (vb === true ? 1 : 0)) * sortDir;
          }
          return String(va).localeCompare(String(vb), undefined, { numeric: true }) * sortDir;
        });
      };

      const commit = () => {
        if (p.interactive === false) return;
        emit(c, 'change', {
          columns,
          data: rows
        });
      };

      const parseClipboard = (text) => {
        const lines = text.trim().split(/\r?\n/);
        return lines.map((line) => {
          if (line.includes('\t')) return line.split('\t');
          if (line.includes(';')) return line.split(';');
          if (line.includes(',')) return line.split(',');
          return [line];
        });
      };

      // Table & Structure DOM
      const table = document.createElement('table');
      table.className = 'mg-table mg-dataeditor-table';

      const thead = document.createElement('thead');
      const headerRow = document.createElement('tr');
      const tbody = document.createElement('tbody');

      const topSpacer = document.createElement('tr');
      topSpacer.className = 'mg-virtual-spacer';
      const topSpacerCell = document.createElement('td');
      topSpacer.appendChild(topSpacerCell);

      const bottomSpacer = document.createElement('tr');
      bottomSpacer.className = 'mg-virtual-spacer';
      const bottomSpacerCell = document.createElement('td');
      bottomSpacer.appendChild(bottomSpacerCell);

      table.appendChild(thead);
      table.appendChild(tbody);
      tableContainer.appendChild(table);

      const updateHeader = () => {
        headerRow.innerHTML = '';
        columns.forEach((col, cIdx) => {
          const th = document.createElement('th');
          th.className = 'mg-dataeditor-th' + (p.sortable !== false ? ' sortable' : '');
          if (col.width) th.style.width = col.width + 'px';

          const titleSpan = document.createElement('span');
          titleSpan.textContent = col.label || col.id;
          th.appendChild(titleSpan);

          if (sortCol === cIdx) {
            const arrow = document.createElement('span');
            arrow.className = 'mg-dataeditor-sort-arrow';
            arrow.textContent = sortDir > 0 ? ' ▲' : ' ▼';
            th.appendChild(arrow);
          }

          if (p.sortable !== false) {
            th.addEventListener('click', () => {
              if (sortCol === cIdx) {
                sortDir = -sortDir;
              } else {
                sortCol = cIdx;
                sortDir = 1;
              }
              sortRows();
              render();
              commit();
            });
          }
          headerRow.appendChild(th);
        });

        if (p.allow_delete !== false && p.interactive !== false) {
          const thOps = document.createElement('th');
          thOps.className = 'mg-dataeditor-th-ops';
          thOps.style.width = '48px';
          headerRow.appendChild(thOps);
        }
        thead.innerHTML = '';
        thead.appendChild(headerRow);
      };

      // Moteur de rendu virtuel (Windowing)
      let ticking = false;
      const render = () => {
        updateHeader();
        const filtered = getFilteredRows();
        const totalRows = filtered.length;
        const totalCols = columns.length + (p.allow_delete !== false && p.interactive !== false ? 1 : 0);
        topSpacerCell.colSpan = totalCols;
        bottomSpacerCell.colSpan = totalCols;

        // Si moins de 50 lignes, rendu direct sans virtualisation
        const isVirtual = totalRows > 50 && p.max_height;
        const scrollTop = tableContainer.scrollTop || 0;
        const viewportHeight = tableContainer.clientHeight || 400;

        let startIdx = 0;
        let endIdx = totalRows;

        if (isVirtual) {
          startIdx = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER_COUNT);
          endIdx = Math.min(totalRows, Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + BUFFER_COUNT);
        }

        const topHeight = startIdx * ROW_HEIGHT;
        const bottomHeight = Math.max(0, (totalRows - endIdx) * ROW_HEIGHT);

        topSpacer.style.height = topHeight + 'px';
        bottomSpacer.style.height = bottomHeight + 'px';
        topSpacer.style.display = topHeight > 0 ? '' : 'none';
        bottomSpacer.style.display = bottomHeight > 0 ? '' : 'none';

        tbody.innerHTML = '';
        if (topHeight > 0) tbody.appendChild(topSpacer);

        for (let rIdx = startIdx; rIdx < endIdx; rIdx++) {
          const row = filtered[rIdx];
          const tr = document.createElement('tr');
          tr.style.height = ROW_HEIGHT + 'px';

          columns.forEach((col, cIdx) => {
            const td = document.createElement('td');
            const val = row[cIdx] !== undefined ? row[cIdx] : '';
            const isEditable = p.interactive !== false && col.editable !== false;

            if (col.type === 'boolean') {
              const cb = document.createElement('input');
              cb.type = 'checkbox';
              cb.className = 'mg-dataeditor-checkbox';
              cb.checked = Boolean(val);
              if (!isEditable) cb.disabled = true;
              cb.addEventListener('change', () => {
                row[cIdx] = cb.checked;
                commit();
              });
              td.appendChild(cb);
            } else if (col.type === 'dropdown' && Array.isArray(col.choices)) {
              const select = document.createElement('select');
              select.className = 'mg-select mg-dataeditor-select';
              if (!isEditable) select.disabled = true;
              col.choices.forEach((choice) => {
                const opt = document.createElement('option');
                opt.value = choice;
                opt.textContent = choice;
                if (String(val) === String(choice)) opt.selected = true;
                select.appendChild(opt);
              });
              select.addEventListener('change', () => {
                row[cIdx] = select.value;
                commit();
              });
              td.appendChild(select);
            } else if (col.type === 'number') {
              if (isEditable) {
                const inp = document.createElement('input');
                inp.type = 'number';
                inp.className = 'mg-cell mg-dataeditor-input';
                inp.value = val != null ? val : '';
                inp.addEventListener('change', () => {
                  row[cIdx] = inp.value === '' ? null : Number(inp.value);
                  commit();
                });
                td.appendChild(inp);
              } else {
                td.textContent = val != null ? String(val) : '';
              }
            } else {
              if (isEditable) {
                const inp = document.createElement('input');
                inp.type = 'text';
                inp.className = 'mg-cell mg-dataeditor-input';
                inp.value = val != null ? String(val) : '';
                inp.addEventListener('change', () => {
                  row[cIdx] = inp.value;
                  commit();
                });
                td.appendChild(inp);
              } else {
                td.textContent = val != null ? String(val) : '';
              }
            }
            tr.appendChild(td);
          });

          if (p.allow_delete !== false && p.interactive !== false) {
            const tdOps = document.createElement('td');
            tdOps.className = 'mg-dataeditor-cell-ops';
            const delBtn = document.createElement('button');
            delBtn.type = 'button';
            delBtn.className = 'mg-btn mg-btn-secondary mg-ico';
            delBtn.textContent = '✕';
            delBtn.title = 'Supprimer cette ligne';
            delBtn.addEventListener('click', () => {
              const originalIdx = rows.indexOf(row);
              if (originalIdx >= 0) {
                rows.splice(originalIdx, 1);
                render();
                commit();
              }
            });
            tdOps.appendChild(delBtn);
            tr.appendChild(tdOps);
          }
          tbody.appendChild(tr);
        }

        if (bottomHeight > 0) tbody.appendChild(bottomSpacer);
      };

      // Écouteur de scroll optimisé par requestAnimationFrame
      tableContainer.addEventListener('scroll', () => {
        if (!ticking) {
          window.requestAnimationFrame(() => {
            render();
            ticking = false;
          });
          ticking = true;
        }
      });

      // Construction de la barre d'outils analytique
      const updateToolbar = () => {
        toolbar.innerHTML = '';

        if (p.interactive !== false && p.allow_add !== false) {
          const addBtn = document.createElement('button');
          addBtn.type = 'button';
          addBtn.className = 'mg-btn mg-btn-secondary';
          addBtn.textContent = '+ Ajouter une ligne';
          addBtn.addEventListener('click', () => {
            const newRow = columns.map((col) => (col.type === 'boolean' ? false : (col.type === 'number' ? 0 : '')));
            rows.push(newRow);
            render();
            commit();
          });
          toolbar.appendChild(addBtn);
        }

        // Champ de recherche / filtrage en direct
        const searchInput = document.createElement('input');
        searchInput.type = 'search';
        searchInput.className = 'mg-input mg-dataeditor-search';
        searchInput.placeholder = '🔍 Filtrer les données...';
        searchInput.value = searchQuery;
        searchInput.addEventListener('input', (e) => {
          searchQuery = e.target.value.trim();
          render();
        });
        toolbar.appendChild(searchInput);

        // Compteur de lignes Big Data
        const countBadge = document.createElement('span');
        countBadge.className = 'mg-dataeditor-count';
        countBadge.textContent = `${rows.length.toLocaleString()} lignes`;
        toolbar.appendChild(countBadge);

        // Export CSV rapide
        const exportBtn = document.createElement('button');
        exportBtn.type = 'button';
        exportBtn.className = 'mg-btn mg-btn-secondary mg-dataeditor-export';
        exportBtn.textContent = '⬇ CSV';
        exportBtn.title = 'Exporter les données au format CSV';
        exportBtn.addEventListener('click', () => {
          let csv = columns.map((col) => `"${(col.label || col.id).replace(/"/g, '""')}"`).join(',') + '\n';
          rows.forEach((r) => {
            csv += r.map((cell) => `"${String(cell ?? '').replace(/"/g, '""')}"`).join(',') + '\n';
          });
          const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = 'export_data.csv';
          a.click();
          URL.revokeObjectURL(url);
        });
        toolbar.appendChild(exportBtn);
      };

      if (p.allow_paste !== false && p.interactive !== false) {
        wrap.addEventListener('paste', (e) => {
          const text = (e.clipboardData || window.clipboardData).getData('text');
          if (text) {
            const pasted = parseClipboard(text);
            if (pasted.length > 0) {
              e.preventDefault();
              pasted.forEach((pRow) => {
                const newRow = columns.map((col, idx) => {
                  const raw = pRow[idx] !== undefined ? pRow[idx] : '';
                  if (col.type === 'number') return isNaN(Number(raw)) ? 0 : Number(raw);
                  if (col.type === 'boolean') return raw === 'true' || raw === '1' || raw === 'oui';
                  return raw;
                });
                rows.push(newRow);
              });
              updateToolbar();
              render();
              commit();
              toast(`${pasted.length} lignes collées`, 'success');
            }
          }
        });
      }

      updateToolbar();
      render();

      c.getValue = () => ({ columns, data: rows });
      c.apply = (patch) => {
        if (Array.isArray(patch.columns)) columns = patch.columns;
        if (Array.isArray(patch.data)) rows = patch.data.map((r) => (Array.isArray(r) ? r.slice() : []));
        if (Array.isArray(patch.append_rows)) {
          patch.append_rows.forEach((r) => {
            if (Array.isArray(r)) rows.push(r.slice());
          });
        }
        if (patch.visible != null) c.el.hidden = !patch.visible;
        updateToolbar();
        render();
      };
    }
  });

