/* grio — moteur frontend (vanilla JS, zéro dépendance) */
(function () {
  'use strict';

  /* ---------- core ---------- */

  const registry = {};
  const byId = {};

  let ws = null;
  let ready = false;
  let pending = [];
  let retry = 1;
  let runButton = null;

  function esc(s) {
    return String(s ?? '').replace(/[&<>"']/g, (c) => (
      { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]
    ));
  }

  function inputsSnapshot() {
    const v = {};
    for (const k in byId) {
      const c = byId[k];
      if (c.input && c.getValue) v[c.id] = c.getValue();
    }
    return v;
  }

  function emit(c, name, data) {
    send({ t: 'event', c: c.id, e: name, d: data ?? null, v: inputsSnapshot() });
  }

  // Bus d'écouteurs client pour les scripts personnalisés (window.grio.on)
  const clientSubscribers = {};

  // API publique robuste exposée sur window pour scripts utilisateur et composants HTML
  window.grio = {
    emit(id, eventName, data) {
      const c = byId[id] || { id };
      emit(c, eventName || 'change', data);
    },
    get(id) {
      const c = byId[id];
      return c && c.getValue ? c.getValue() : undefined;
    },
    snapshot() {
      return inputsSnapshot();
    },
    on(id, callback) {
      if (!clientSubscribers[id]) clientSubscribers[id] = [];
      clientSubscribers[id].push(callback);
      return () => {
        clientSubscribers[id] = (clientSubscribers[id] || []).filter((cb) => cb !== callback);
      };
    },
    toast(msg, level) {
      toast(msg, level);
    }
  };

  function send(payload) {
    const raw = JSON.stringify(payload);
    if (ready) ws.send(raw);
    else pending.push(raw);
  }

  function flash(el) {
    el.classList.remove('mg-flash');
    void el.offsetWidth;
    el.classList.add('mg-flash');
  }

  function toast(msg, level) {
    const t = document.createElement('div');
    t.className = 'mg-toast' + (level ? ' mg-toast-' + level : '');
    t.textContent = msg;
    t.dataset.level = level || 'error';
    document.body.appendChild(t);
    setTimeout(() => {
      t.classList.add('out');
      setTimeout(() => t.remove(), 360);
    }, 3200);
  }

  function connect() {
    const proto = location.protocol === 'https:' ? 'wss://' : 'ws://';
    ws = new WebSocket(proto + location.host + '/ws');

    ws.onopen = () => {
      ready = true;
      retry = 1;
      const p = pending;
      pending = [];
      p.forEach((s) => ws.send(s));
      send({ t: 'event', c: '', e: 'load', d: null, v: {} });
    };

    ws.onmessage = (ev) => {
      let m;
      try { m = JSON.parse(ev.data); } catch { return; }
      if (m.t === 'update') {
        (m.u || []).forEach((u) => {
          const c = byId[u.id];
          if (c && c.apply) { c.apply(u.p || {}); flash(c.el); }
          // Notification des abonnés client (window.grio.on)
          if (clientSubscribers[u.id]) {
            clientSubscribers[u.id].forEach((cb) => {
              try { cb(u.p || {}); } catch (err) { console.error('[grio.on error]', err); }
            });
          }
        });
      } else if (m.t === 'alert') {
        toast(m.m || '—', m.level || 'info');
      } else if (m.t === 'error') {
        toast(m.m || 'Erreur', 'error');
      }
    };

    ws.onclose = () => {
      ready = false;
      setTimeout(connect, Math.min(1000 * retry++, 15000));
    };

    ws.onerror = () => ws.close();
  }

  function register(kind, impl) { registry[kind] = impl; }

  function applyLayout(el, props) {
    const L = props.layout;
    if (!L) return;
    const st = el.style;
    if (L.scale) { st.flexGrow = L.scale; st.flexBasis = '0%'; }
    if (L.width) { st.width = L.width + 'px'; st.flexGrow = '0'; }
    if (L.height) st.height = L.height + 'px';
    if (L.max_width) { st.maxWidth = L.max_width + 'px'; }
    if (L.max_height) st.maxHeight = L.max_height + 'px';
    if (L.min_width) st.minWidth = L.min_width + 'px';
  }

  function mount(el) {
    const kind = el.dataset.kind;
    const props = JSON.parse(el.dataset.props || '{}');
    if (kind === 'row' || kind === 'column' || kind === 'grid' || kind === 'panel' || kind === 'accordion') {
      if (typeof props.gap === 'number') el.style.setProperty('--mg-gap', props.gap + 'px');
      if (typeof props.gap_x === 'number') el.style.setProperty('--mg-gap-x', props.gap_x + 'px');
      if (typeof props.gap_y === 'number') el.style.setProperty('--mg-gap-y', props.gap_y + 'px');
      if (props.columns) el.style.setProperty('--mg-grid-cols', props.columns);
      if (props.wrap === false) el.style.setProperty('--mg-wrap', 'nowrap');
      if (props.align) el.style.setProperty('--mg-align', props.align);
      if (props.justify) el.style.setProperty('--mg-justify', props.justify);
      applyLayout(el, props);
      return;
    }
    const impl = registry[kind];
    if (!impl) {
      el.innerHTML = '<span class="mg-unknown">unknown component: ' + esc(kind) + '</span>';
      return;
    }
    const c = { kind, id: el.dataset.id, props, el, input: el.dataset.role === 'input' };
    byId[c.id] = c;
    impl.mount(c);
    applyLayout(el, props);
  }

  /* ---------- markdown mini ---------- */

  function inline(s) {
    let t = esc(s);
    t = t.replace(/`([^`]+)`/g, (_, x) => '<code class="mg-code-inline">' + x + '</code>');
    t = t.replace(/\*\*([^*\n]+)\*\*/g, '<strong>$1</strong>');
    t = t.replace(/\*([^*\n]+)\*/g, '<em>$1</em>');
    t = t.replace(/~~([^~\n]+)~~/g, '<del>$1</del>');
    t = t.replace(/\[([^\]]+)\]\((https?:\/\/[^)\s]+)\)/g,
      '<a href="$2" target="_blank" rel="noopener">$1</a>');
    return t;
  }

  function markdown(src) {
    const codes = [];
    let s = String(src || '');
    s = s.replace(/```([\s\S]*?)```/g, (_, b) => {
      const token = '__MGCODE_' + codes.length + '__';
      codes.push('<pre class="mg-code"><code>' + esc(b) + '</code></pre>');
      return token;
    });
    const lines = s.split('\n').map((line) => {
      line = line.trim();
      if (/^#{1,6}\s/.test(line)) {
        const n = line.match(/^#+/)[0].length;
        return '<h' + n + '>' + inline(line.replace(/^#+\s/, '')) + '</h' + n + '>';
      }
      if (/^>\s?/.test(line)) return '<blockquote>' + inline(line.replace(/^>\s?/, '')) + '</blockquote>';
      if (/^[-*]\s/.test(line)) return '<li>' + inline(line.replace(/^[-*]\s/, '')) + '</li>';
      if (/^\d+\.\s/.test(line)) return '<li>' + inline(line.replace(/^\d+\.\s/, '')) + '</li>';
      if (/^\s*-{3,}\s*$/.test(line)) return '<hr>';
      return inline(line);
    }).join('\n');
    return lines.replace(/__MGCODE_\d+__/g, () => codes.shift());
  }

  /* ---------- composants ---------- */

  register('text', {
    mount(c) {
      const p = c.props;
      const id = 'f_' + c.id;
      const ph = p.placeholder ? 'placeholder="' + esc(p.placeholder) + '"' : '';
      const isMulti = p.lines && p.lines > 1;
      const inputHtml = isMulti
        ? '<textarea id="' + id + '" class="mg-input" rows="' + p.lines + '" ' + ph + '>' + esc(p.value ?? '') + '</textarea>'
        : '<input id="' + id + '" class="mg-input" type="text" value="' + esc(p.value ?? '') + '" ' + ph + ' autocomplete="off">';
      c.el.innerHTML =
        '<label class="mg-label" for="' + id + '"><span>' + esc(p.label || c.id) + '</span></label>' + inputHtml;
      const input = c.el.querySelector('.mg-input');
      if (p.interactive === false) { input.disabled = true; c.el.classList.add('mg-disabled'); }
      input.addEventListener('input', () => emit(c, 'change', input.value));
      c.getValue = () => input.value;
      c.apply = (patch) => {
        if (patch.value != null && String(input.value) !== String(patch.value)) input.value = patch.value;
      };
    }
  });

  register('slider', {
    mount(c) {
      const p = c.props;
      const id = 'f_' + c.id;
      c.el.innerHTML =
        '<div class="mg-label"><span>' + esc(p.label || c.id) + '</span>' +
        '<span class="mg-slider-value"></span></div>' +
        '<input id="' + id + '" class="mg-range" type="range" min="' + p.min + '" max="' + p.max +
        '" step="' + p.step + '" value="' + p.value + '">';
      const range = c.el.querySelector('input');
      const val = c.el.querySelector('.mg-slider-value');
      val.textContent = range.value;
      if (p.interactive === false) { range.disabled = true; c.el.classList.add('mg-disabled'); }
      range.addEventListener('input', () => {
        val.textContent = range.value;
        emit(c, 'change', parseFloat(range.value));
      });
      c.getValue = () => parseFloat(range.value);
      c.apply = (patch) => {
        if (patch.value != null) { range.value = patch.value; val.textContent = range.value; }
      };
    }
  });

  register('output', {
    mount(c) {
      const p = c.props;
      c.el.innerHTML =
        '<div class="mg-card mg-output">' +
        '<div class="mg-card-label">' + esc(p.label || c.id) + '</div>' +
        '<div class="mg-output-text"></div></div>';
      const out = c.el.querySelector('.mg-output-text');
      out.textContent = p.value ?? '';
      c.apply = (patch) => {
        if (patch.value != null) out.textContent = patch.value;
        if (patch.append != null) out.textContent = String(out.textContent) + String(patch.append);
        if (patch.label != null) {
          const lbl = c.el.querySelector('.mg-card-label');
          if (lbl) lbl.textContent = patch.label;
        }
        if (patch.visible != null) c.el.hidden = !patch.visible;
        if (patch.disabled != null) c.el.classList.toggle('mg-disabled', !!patch.disabled);
      };
    }
  });

  register('progress', {
    mount(c) {
      const p = c.props;
      const variant = p.variant || 'bar';
      const size = p.size || (variant === 'bar' ? null : 84);

      if (variant === 'circle') {
        const dim = size;
        const strokeW = 8;
        const radius = 42;
        const circ = 2 * Math.PI * radius;
        c.el.innerHTML =
          '<div class="mg-progress-circle-wrap" style="width:' + dim + 'px">' +
          '<div class="mg-progress-circle-box" style="width:' + dim + 'px;height:' + dim + 'px">' +
          '<svg class="mg-progress-circle-svg" viewBox="0 0 100 100">' +
          '<circle class="mg-progress-circle-bg" cx="50" cy="50" r="' + radius + '" stroke-width="' + strokeW + '"/>' +
          '<circle class="mg-progress-circle-fill" cx="50" cy="50" r="' + radius + '" stroke-width="' + strokeW + '" ' +
          'stroke-dasharray="' + circ + '" stroke-dashoffset="' + circ + '"/>' +
          '</svg>' +
          '<div class="mg-progress-circle-center">' +
          '<span class="mg-progress-value">0%</span>' +
          '</div>' +
          '</div>' +
          '<div class="mg-progress-info">' +
          '<div class="mg-label-text">' + esc(p.label || c.id) + '</div>' +
          '<div class="mg-progress-label"></div>' +
          '</div>' +
          '</div>';

        const fill = c.el.querySelector('.mg-progress-circle-fill');
        const val = c.el.querySelector('.mg-progress-value');
        const lab = c.el.querySelector('.mg-progress-label');

        c.apply = (patch) => {
          if (patch.value == null) return;
          const raw = patch.value;
          const f = typeof raw === 'number' ? raw : (raw.progress ?? 0);
          const label = typeof raw === 'object' && raw != null ? (raw.label ?? '') : '';
          const pct = Math.max(0, Math.min(100, Math.round(f * 100)));
          const offset = circ - (pct / 100) * circ;
          fill.style.strokeDashoffset = offset;
          fill.classList.toggle('done', pct >= 100);
          val.textContent = pct + '%';
          lab.textContent = label != null ? String(label) : '';
        };
      } else if (variant === 'pie') {
        const dim = size;
        c.el.innerHTML =
          '<div class="mg-progress-pie-wrap">' +
          '<div class="mg-progress-pie-disk" style="width:' + dim + 'px;height:' + dim + 'px;--mg-pct:0%">' +
          '<div class="mg-progress-pie-badge">0%</div>' +
          '</div>' +
          '<div class="mg-progress-info">' +
          '<div class="mg-label-text">' + esc(p.label || c.id) + '</div>' +
          '<div class="mg-progress-label"></div>' +
          '</div>' +
          '</div>';

        const disk = c.el.querySelector('.mg-progress-pie-disk');
        const badge = c.el.querySelector('.mg-progress-pie-badge');
        const lab = c.el.querySelector('.mg-progress-label');

        c.apply = (patch) => {
          if (patch.value == null) return;
          const raw = patch.value;
          const f = typeof raw === 'number' ? raw : (raw.progress ?? 0);
          const label = typeof raw === 'object' && raw != null ? (raw.label ?? '') : '';
          const pct = Math.max(0, Math.min(100, Math.round(f * 100)));
          disk.style.setProperty('--mg-pct', pct + '%');
          disk.classList.toggle('done', pct >= 100);
          badge.textContent = pct + '%';
          lab.textContent = label != null ? String(label) : '';
        };
      } else {
        // Mode bar par défaut
        c.el.innerHTML =
          '<div class="mg-label"><span>' + esc(p.label || c.id) + '</span>' +
          '<span class="mg-progress-value">0%</span></div>' +
          '<div class="mg-progress-track"><div class="mg-progress-bar" style="width:0%"></div></div>' +
          '<div class="mg-progress-label"></div>';
        const bar = c.el.querySelector('.mg-progress-bar');
        const val = c.el.querySelector('.mg-progress-value');
        const lab = c.el.querySelector('.mg-progress-label');

        c.apply = (patch) => {
          if (patch.value == null) return;
          const raw = patch.value;
          const f = typeof raw === 'number' ? raw : (raw.progress ?? 0);
          const label = typeof raw === 'object' && raw != null ? (raw.label ?? '') : '';
          const pct = Math.max(0, Math.min(100, Math.round(f * 100)));
          bar.style.width = pct + '%';
          bar.classList.toggle('done', pct >= 100);
          val.textContent = pct + '%';
          lab.textContent = label != null ? String(label) : '';
        };
      }
    }
  });

  register('markdown', {
    mount(c) {
      const div = document.createElement('div');
      div.className = 'mg-markdown';
      div.innerHTML = markdown(c.props.text);
      c.el.appendChild(div);
      c.apply = (patch) => {
        if (patch.value != null) div.innerHTML = markdown(patch.value);
      };
    }
  });

  /* ---------- Phase 7: Number / Label / JSON / Timer / File / Download ---------- */

  register('number', {
    mount(c) {
      const p = c.props;
      const id = 'f_' + c.id;
      const step = p.step || 1;
      const min = p.min != null ? p.min : 0;
      const max = p.max != null ? p.max : 1e6;
      const clamp = (v) => Math.min(max, Math.max(min, Number.isFinite(v) ? v : min));
      const snap = (v) => {
        const r = Math.round((clamp(v) - min) / step) * step + min;
        return Number(r.toFixed(10));
      };
      c.el.innerHTML = makeLabel(p, c) +
        '<div class="mg-num" data-id="' + id + '">' +
          '<button type="button" class="mg-btn mg-btn-secondary mg-num-btn" data-d="-1" title="' + t('num_step_down') + '" aria-label="' + t('num_step_down') + '">−</button>' +
          '<input class="mg-input mg-num-input" id="' + id + '" type="number" step="' + step + '" min="' + min + '" max="' + max + '" value="' + p.value + '" autocomplete="off">' +
          '<button type="button" class="mg-btn mg-btn-secondary mg-num-btn" data-d="1" title="' + t('num_step_up') + '" aria-label="' + t('num_step_up') + '">+</button>' +
          (p.unit ? '<span class="mg-num-unit">' + esc(p.unit) + '</span>' : '') +
        '</div>';
      const decBtn = c.el.querySelector('[data-d="-1"]');
      const incBtn = c.el.querySelector('[data-d="1"]');
      const setStepTitles = () => {
        decBtn.title = t('num_step_down'); decBtn.setAttribute('aria-label', t('num_step_down'));
        incBtn.title = t('num_step_up'); incBtn.setAttribute('aria-label', t('num_step_up'));
      };
      setStepTitles();
      onI18n(setStepTitles);
      const input = c.el.querySelector('.mg-num-input');
      const setVal = (v) => { input.value = snap(v); };
      const fire = () => { const v = snap(parseFloat(input.value)); input.value = v; emit(c, 'change', v); };
      if (p.interactive === false) { input.disabled = true; c.el.classList.add('mg-disabled'); }
      input.addEventListener('input', fire);
      c.el.querySelectorAll('.mg-num-btn').forEach((b) => b.addEventListener('click', () => {
        const v = snap(parseFloat(input.value) + step * +(b.dataset.d));
        input.value = v; emit(c, 'change', v);
      }));
      c.getValue = () => snap(parseFloat(input.value));
      c.apply = (patch) => { if (patch.value != null) setVal(Number(patch.value)); };
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
              } catch { /* reformate quand même */ }
            }
            show(v); mark(true);
          }
        };
      } else {
        show(p.value ?? null);
        c.apply = (patch) => {
          if (patch.value != null) {
            let v = patch.value;
            if (typeof v === 'string') { try { v = JSON.parse(v); } catch { /* garde la chaîne */ } }
            show(v);
          }
        };
      }
    }
  });

  register('timer', {
    mount(c) {
      const p = c.props;
      const iv = Math.max(50, (p.interval || 1) * 1000);
      const box = document.createElement('div');
      box.className = 'mg-timer';
      box.innerHTML = makeLabel(p, c) + '<div class="mg-timer-value">0.0 s</div>';
      c.el.appendChild(box);
      const out = box.querySelector('.mg-timer-value');
      let running = p.running !== false;
      let t0 = performance.now();
      const elapsed = () => (performance.now() - t0) / 1000;
      const render = () => { out.textContent = elapsed().toFixed(1) + ' s'; };
      const tick = () => {
        render();
        emit(c, 'change', elapsed());
      };
      render();
      let handle = null;
      if (running) handle = setInterval(tick, iv);
      c.apply = (patch) => {
        if (patch.running != null && !!patch.running !== running) {
          running = !!patch.running;
          if (running) {
            t0 = performance.now() - elapsed() * 1000;
            if (!handle) handle = setInterval(tick, iv);
          } else if (handle) {
            clearInterval(handle); handle = null;
          }
        }
      };
    }
  });

  function fmtBytes(n) {
    if (n < 1024) return n + ' o';
    if (n < 1048576) return (n / 1024).toFixed(1) + ' Ko';
    return (n / 1048576).toFixed(1) + ' Mo';
  }

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
      b.addEventListener('click', (ev) => {
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

  /* ---------- média ---------- */

  function dataUrl(v) { return typeof v === 'string' && v.length ? v : ''; }

  function makeLabel(p, c) {
    return '<div class="mg-label"><span>' + esc(p.label || c.id) + '</span></div>';
  }

  function readFile(file, done) {
    const r = new FileReader();
    r.onload = () => done(String(r.result));
    r.readAsDataURL(file);
  }

  function sendStream(c, blob) {
    const r = new FileReader();
    r.onload = () => {
      const b64 = String(r.result).split(',')[1] || '';
      send({ t: 'stream', c: c.id, p: { mime: blob.type || 'application/octet-stream', b64 } });
    };
    r.readAsDataURL(blob);
  }

  function wireUpload(box, onData) {
    box.addEventListener('click', () => {
      const f = box.querySelector('input[type=file]');
      if (f) f.click();
    });
    const input = box.querySelector('input[type=file]');
    input.addEventListener('change', (e) => {
      const file = e.target.files && e.target.files[0];
      if (!file) return;
      readFile(file, onData);
    });
  }

  function playerButtons(c) {
    const wrap = document.createElement('div');
    wrap.className = 'mg-media-controls';
    ['play', 'pause', 'stop'].forEach((name) => {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'mg-btn mg-btn-secondary mg-media-btn';
      b.textContent = name;
      b.addEventListener('click', () => emit(c, name, null));
      wrap.appendChild(b);
    });
    return wrap;
  }

  register('image', {
    mount(c) {
      const p = c.props;
      const interactive = p.interactive !== false;
      const box = document.createElement('div');
      box.className = 'mg-media-box' + (interactive ? ' mg-media-drop' : '');
      const img = document.createElement('img');
      img.className = 'mg-media-img';
      img.alt = '';
      const src0 = dataUrl(p.value);
      img.src = src0 || 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
      if (!src0) img.classList.add('mg-media-empty');
      const file = interactive ? '<input type="file" accept="image/*" hidden>' : '';
      box.innerHTML = file;
      box.appendChild(img);
      const holder = document.createElement('div');
      holder.className = 'mg-media';
      holder.innerHTML = makeLabel(p, c) + (c.input ? '<div class="mg-hint">clic ou glisse une image</div>' : '');
      holder.appendChild(box);
      c.el.appendChild(holder);

      c.getValue = () => (img.classList.contains('mg-media-empty') ? '' : img.src);
      c.apply = (patch) => {
        if (patch.value != null && dataUrl(patch.value)) {
          img.src = patch.value;
          img.classList.remove('mg-media-empty');
        }
      };

      if (!interactive) return;
      wireUpload(box, (url) => {
        img.src = url;
        img.classList.remove('mg-media-empty');
        emit(c, 'change', url);
      });
      box.addEventListener('dragover', (e) => { e.preventDefault(); box.classList.add('hover'); });
      box.addEventListener('dragleave', () => box.classList.remove('hover'));
      box.addEventListener('drop', (e) => {
        e.preventDefault();
        box.classList.remove('hover');
        const f = e.dataTransfer.files && e.dataTransfer.files[0];
        if (f) readFile(f, (url) => { img.src = url; emit(c, 'change', url); });
      });
    }
  });

  register('imageeditor', {
    mount(c) {
      const p = c.props;
      const interactive = p.interactive !== false;
      const nLayers = Math.max(1, Math.min(4, p.layers || 2));
      const dab = (s, v) => { const b = document.createElement('button'); b.type = 'button'; b.className = 'mg-btn mg-btn-secondary mg-ie-tool'; b.textContent = s; if (v) b.dataset.v = v; return b; };

      const wrap = document.createElement('div');
      wrap.className = 'mg-ie';
      wrap.innerHTML = makeLabel(p, c);
      c.el.appendChild(wrap);

      let bg = null, BGW = 900, BGH = 600;
      const layers = []; // { cv, ctx, visible, opacity }
      const mkLayer = (w, h) => {
        const cv = document.createElement('canvas');
        cv.width = w; cv.height = h;
        return { cv, ctx: cv.getContext('2d'), visible: true, opacity: 1 };
      };
      const visibleComp = () => {
        const cv = document.createElement('canvas');
        cv.width = BGW; cv.height = BGH;
        const g = cv.getContext('2d');
        if (bg) g.drawImage(bg, 0, 0);
        layers.forEach((l) => { if (l.visible) { g.globalAlpha = l.opacity; g.drawImage(l.cv, 0, 0); g.globalAlpha = 1; } });
        return cv;
      };
      const mask = () => {
        const cv = document.createElement('canvas');
        cv.width = BGW; cv.height = BGH;
        const g = cv.getContext('2d');
        g.fillStyle = '#000'; g.fillRect(0, 0, BGW, BGH);
        layers.forEach((l) => {
          g.drawImage(l.cv, 0, 0);
          g.globalCompositeOperation = 'source-in';
          g.fillStyle = '#fff'; g.fillRect(0, 0, BGW, BGH);
          g.globalCompositeOperation = 'source-over';
        });
        return cv;
      };
      const snapshot = () => ({ bg: bg ? bg.getContext('2d').getImageData(0, 0, BGW, BGH) : null, layers: layers.map((l) => l.ctx.getImageData(0, 0, BGW, BGH)), w: BGW, h: BGH });
      const history = []; let hIndex = -1;
      const pushHistory = () => { history.splice(hIndex + 1); history.push(snapshot()); if (history.length > 20) history.shift(); hIndex = history.length - 1; };
      const restore = (s) => {
        BGW = s.w; BGH = s.h;
        bg = document.createElement('canvas'); bg.width = BGW; bg.height = BGH;
        if (s.bg) bg.getContext('2d').putImageData(s.bg, 0, 0);
        layers.splice(0, layers.length);
        for (let i = 0; i < s.layers.length; i++) {
          layers.push(mkLayer(BGW, BGH));
          layers[i].ctx.putImageData(s.layers[i], 0, 0);
        }
      };
      const redo = (ev) => { if (hIndex + 1 >= history.length) return; restore(history[++hIndex]); draw(); commit(); };
      const undo = (ev) => { if (hIndex < 0) return; hIndex--; restore(hIndex >= 0 ? history[hIndex] : snapshotBlank()); draw(); commit(); };
      const snapshotBlank = () => { const c0 = document.createElement('canvas'); c0.width = BGW; c0.height = BGH; return { bg: null, layers: layers.map(() => { const c1 = document.createElement('canvas'); c1.width = BGW; c1.height = BGH; return c1.getContext('2d').createImageData(BGW, BGH); }), w: BGW, h: BGH }; };

      const view = document.createElement('canvas');
      view.className = 'mg-ie-canvas';
      const vctx = view.getContext('2d');
      const ovl = document.createElement('canvas');
      ovl.className = 'mg-ie-canvas mg-ie-ovl';
      const octx = ovl.getContext('2d');
      const stage = document.createElement('div');
      stage.className = 'mg-ie-stage';
      stage.appendChild(view); stage.appendChild(ovl);

      let zoom = 1, tx = 0, ty = 0, tool = 'brush', color = '#e11d48', size = 8;
      let drawing = null;
      const scaleCv = (aw, ah) => {
        const w = Math.max(1, Math.round(aw)), h = Math.max(1, Math.round(ah));
        view.width = w; view.height = h; view.style.width = w + 'px'; view.style.height = h + 'px';
        ovl.width = w; ovl.height = h; ovl.style.width = w + 'px'; ovl.style.height = h + 'px';
      };
      const fit = () => {
        const availW = Math.max(80, stage.clientWidth - 4), availH = Math.max(120, stage.clientHeight - 4);
        zoom = Math.min(availW / BGW, availH / BGH, 4);
        zoom = Math.max(zoom, 0.05);
        tx = (availW - BGW * zoom) / 2; ty = (availH - BGH * zoom) / 2;
        scaleCv(availW, availH);
        draw();
        clearOvl();
      };
      const draw = () => {
        vctx.setTransform(1, 0, 0, 1, 0, 0);
        vctx.clearRect(0, 0, view.width, view.height);
        const g = vctx;
        if (bg) { g.drawImage(bg, tx, ty, BGW * zoom, BGH * zoom); }
        layers.forEach((l) => {
          if (!l.visible) return;
          g.globalAlpha = l.opacity;
          g.drawImage(l.cv, tx, ty, BGW * zoom, BGH * zoom);
          g.globalAlpha = 1;
        });
      };
      const clearOvl = () => { octx.setTransform(1, 0, 0, 1, 0, 0); octx.clearRect(0, 0, ovl.width, ovl.height); };
      const toImg = (ev) => {
        const r = ovl.getBoundingClientRect();
        return { x: (ev.clientX - r.left - tx) / zoom, y: (ev.clientY - r.top - ty) / zoom };
      };

      const commit = () => {
        if (!interactive) return;
        const img = visibleComp(), mk = mask();
        emit(c, 'change', {
          image: img.toDataURL('image/png'),
          layers: layers.map((l) => l.cv.toDataURL('image/png')),
          mask: mk.toDataURL('image/png'),
        });
      };
      c.getValue = () => ({ image: visibleComp().toDataURL('image/png'), layers: layers.map((l) => l.cv.toDataURL('image/png')), mask: mask().toDataURL('image/png') });
      c.apply = (patch) => {
        if (patch.visible != null) c.el.hidden = !patch.visible;
        if (patch.value != null && dataUrl(patch.value) && dataUrl(patch.value) !== bgSrc) {
          loadBg(dataUrl(patch.value));
        }
      };
      let bgSrc = '';

      const px = (fn) => {
        if (!bg) return;
        const g = bg.getContext('2d');
        const d = g.getImageData(0, 0, BGW, BGH);
        for (let i = 0; i < d.data.length; i += 4) fn(i, d.data);
        g.putImageData(d, 0, 0);
      };
      const gray = () => { px((i, d) => { const v = 0.299 * d[i] + 0.587 * d[i + 1] + 0.114 * d[i + 2]; d[i] = d[i + 1] = d[i + 2] = v; }); };
      const invert = () => { px((i, d) => { d[i] = 255 - d[i]; d[i + 1] = 255 - d[i + 1]; d[i + 2] = 255 - d[i + 2]; }); };
      const level = (k) => { px((i, d) => { d[i] = Math.min(255, d[i] * k); d[i + 1] = Math.min(255, d[i + 1] * k); d[i + 2] = Math.min(255, d[i + 2] * k); }); };
      const blur4 = () => {
        if (!bg) return;
        const tmp = document.createElement('canvas');
        tmp.width = BGW; tmp.height = BGH;
        const g = tmp.getContext('2d');
        g.filter = 'blur(5px)'; g.drawImage(bg, 0, 0); g.filter = 'none';
        bg.getContext('2d').clearRect(0, 0, BGW, BGH);
        bg.getContext('2d').drawImage(tmp, 0, 0);
      };

      // --- barre d'outils ---
      const bar = document.createElement('div');
      bar.className = 'mg-ie-bar';
      if (interactive) {
        const loadUserImage = (url) => { history.length = 0; hIndex = -1; loadBg(url, null, () => commit()); };
        const fileIn = document.createElement('input');
        fileIn.type = 'file'; fileIn.accept = 'image/*'; fileIn.hidden = true;
        fileIn.addEventListener('change', () => {
          const f = fileIn.files && fileIn.files[0];
          if (f) readFile(f, loadUserImage);
          fileIn.value = '';
        });
        const open = dab('', '-');
        open.addEventListener('click', () => fileIn.click());
        bar.appendChild(open);
        bar.appendChild(fileIn);
        bar.appendChild(document.createElement('span')).className = 'mg-ie-sep';
        stage.addEventListener('dragover', (e) => { e.preventDefault(); stage.classList.add('hover'); });
        stage.addEventListener('dragleave', () => stage.classList.remove('hover'));
        stage.addEventListener('drop', (e) => {
          e.preventDefault();
          stage.classList.remove('hover');
          const f = e.dataTransfer.files && e.dataTransfer.files[0];
          if (f && f.type.startsWith('image/')) readFile(f, loadUserImage);
        });
        const tools = [];
        if (p.brush !== false) tools.push(['brush', 'ie_brush', '🖌️'], ['eraser', 'ie_eraser', '🧹']);
        if (p.shapes !== false) tools.push(['rect', 'ie_rect', '▭'], ['line', 'ie_line', '╱'], ['arrow', 'ie_arrow', '➜']);
        if (p.crop !== false) tools.push(['crop', 'ie_crop', '✂️']);
        tools.push(['pan', 'ie_pan', '✋']);
        const toolBtns = [];
        tools.forEach(([id, key, ic], idx) => {
          const b = dab('', '-'); b.dataset.t = id; b.dataset.key = key; b.dataset.icon = ic;
          if (idx === 0 && tool === id) b.classList.add('mg-ie-on');
          b.addEventListener('click', () => {
            tool = id;
            ovl.dataset.t = id;
            bar.querySelectorAll('.mg-ie-tool').forEach((x) => x.classList.remove('mg-ie-on'));
            b.classList.add('mg-ie-on');
          });
          bar.appendChild(b);
          toolBtns.push(b);
        });
        bar.appendChild(document.createElement('span')).className = 'mg-ie-sep';
        const col = document.createElement('input');
        col.type = 'color'; col.value = color;
        col.addEventListener('input', () => { color = col.value; });
        bar.appendChild(col);
        const sz = document.createElement('input');
        sz.type = 'range'; sz.min = 2; sz.max = 80; sz.value = size;
        sz.addEventListener('input', () => { size = +sz.value; });
        bar.appendChild(sz);
        bar.appendChild(document.createElement('span')).className = 'mg-ie-sep';
        const rotBtn = dab('', '-');
        rotBtn.addEventListener('click', () => { if (!interactive) return; pushHistory(); rotateBoth(1); draw(); commit(); });
        bar.appendChild(rotBtn);

        const filt = document.createElement('div');
        filt.className = 'mg-btn mg-btn-secondary mg-ie-tool mg-ie-menuwrap';
        filt.innerHTML = '<span>✨ <span class="mg-filt-label"></span></span>';
        const fmenu = document.createElement('div');
        fmenu.className = 'mg-ie-menu';
        fmenu.hidden = true;
        const filterDefs = [
          ['ie_f_gray', gray],
          ['ie_f_invert', invert],
          ['ie_f_bright', () => level(1.35)],
          ['ie_f_dark', () => level(0.6)],
          ['ie_f_blur', blur4]
        ];
        const filterBtns = [];
        filterDefs.forEach(([key, fn]) => {
          const b = document.createElement('button'); b.type = 'button'; b.dataset.key = key;
          b.addEventListener('click', (ev) => {
            ev.stopPropagation();
            fmenu.hidden = true;
            if (!interactive) return;
            pushHistory(); fn(); draw(); commit();
          });
          fmenu.appendChild(b);
          filterBtns.push(b);
        });
        filt.appendChild(fmenu);
        filt.addEventListener('click', () => { fmenu.hidden = !fmenu.hidden; });
        bar.appendChild(filt);
        bar.appendChild(document.createElement('span')).className = 'mg-ie-sep';
        const undoBtn = dab('', '-'); undoBtn.addEventListener('click', undo);
        const redoBtn = dab('', '-'); redoBtn.addEventListener('click', redo);
        const resetBtn = dab('', '-'); resetBtn.addEventListener('click', () => { if (!interactive) return; pushHistory(); loadBg(bgInit || null, true); draw(); commit(); });
        bar.appendChild(undoBtn);
        bar.appendChild(redoBtn);
        bar.appendChild(resetBtn);

        const updateIeToolbarTexts = () => {
          open.textContent = `📁 ${t('ie_open', 'Ouvrir')}`;
          open.title = t('ie_open_title', 'Charger une image');
          toolBtns.forEach((b) => {
            const label = t(b.dataset.key);
            b.textContent = `${b.dataset.icon} ${label}`;
            b.title = label;
          });
          col.title = t('ie_color', 'Couleur');
          sz.title = t('ie_size', 'Épaisseur');
          rotBtn.textContent = `🔄 ${t('ie_rotate', 'Rotation')}`;
          rotBtn.title = t('ie_rotate', 'Rotation');
          const fl = filt.querySelector('.mg-filt-label');
          if (fl) fl.textContent = t('ie_filters', 'Filtres');
          filterBtns.forEach((b) => { b.textContent = t(b.dataset.key); });
          undoBtn.textContent = `↩️ ${t('ie_undo', 'Annuler')}`;
          undoBtn.title = t('ie_undo', 'Annuler');
          redoBtn.textContent = `↪️ ${t('ie_redo', 'Rétablir')}`;
          redoBtn.title = t('ie_redo', 'Rétablir');
          resetBtn.textContent = `🗑️ ${t('ie_reset', 'Reset')}`;
          resetBtn.title = t('ie_reset', 'Reset');
        };
        updateIeToolbarTexts();
        onI18n(updateIeToolbarTexts);
      }
      wrap.appendChild(bar);
      wrap.appendChild(stage);

      // --- panneau calques ---
      const lp = document.createElement('div');
      lp.className = 'mg-ie-layers';
      const lpTitle = document.createElement('div');
      lpTitle.className = 'mg-label';
      lpTitle.innerHTML = '<span class="mg-lp-title">Calques</span>';
      lp.appendChild(lpTitle);
      let sel = 0;
      const buildLayersUI = () => {
        lp.querySelectorAll('.mg-ie-layer').forEach((e) => e.remove());
        layers.forEach((l, i) => {
          const row = document.createElement('div');
          row.className = 'mg-ie-layer' + (i === sel ? ' sel' : '');
          const eye = document.createElement('button');
          eye.type = 'button'; eye.className = 'mg-btn mg-btn-secondary mg-ico';
          eye.textContent = l.visible ? '👁' : '—';
          eye.title = t('ie_visibility', 'Visibilité');
          eye.addEventListener('click', () => { l.visible = !l.visible; eye.textContent = l.visible ? '👁' : '—'; draw(); commit(); });
          const name = document.createElement('span');
          name.className = 'mg-layer-name';
          name.textContent = `${t('ie_layer', 'Calque')} ${i + 1}`;
          const op = document.createElement('input');
          op.type = 'range'; op.min = 0; op.max = 100; op.value = Math.round(l.opacity * 100);
          op.addEventListener('input', () => { l.opacity = op.value / 100; draw(); });
          op.addEventListener('change', () => draw());
          row.append(eye, name, op);
          row.addEventListener('click', (ev) => { if (ev.target === op) return; sel = i; buildLayersUI(); });
          lp.appendChild(row);
        });
      };
      const updateLayersTitle = () => {
        const tSpan = lpTitle.querySelector('.mg-lp-title');
        if (tSpan) tSpan.textContent = t('ie_layers', 'Calques');
        lp.querySelectorAll('.mg-ie-layer').forEach((row, idx) => {
          const n = row.querySelector('.mg-layer-name');
          if (n) n.textContent = `${t('ie_layer', 'Calque')} ${idx + 1}`;
          const eye = row.querySelector('.mg-ico');
          if (eye) eye.title = t('ie_visibility', 'Visibilité');
        });
      };
      onI18n(updateLayersTitle);
      updateLayersTitle();
      wrap.appendChild(lp);
      if (!interactive) lp.hidden = true;
      buildLayersUI();

      // --- pointer ---
      const pt = (ev) => {
        const s = toImg(ev);
        const bounds = { x: 0, y: 0, w: BGW, h: BGH };
        const inside = s.x >= 0 && s.y >= 0 && s.x < BGW && s.y < BGH;
        if (tool === 'brush' || tool === 'eraser') {
          if (ev.type === 'pointerdown') {
            pushHistory();
            drawing = { l: layers[sel], pts: [s], mode: tool };
            beginStroke(drawing, s);
          } else if (ev.type === 'pointermove' && drawing) {
            drawing.pts.push(s); drawStroke(drawing, s); draw();
          } else if (ev.type === 'pointerup' && drawing) {
            drawing = null; commit();
          }
        } else if (tool === 'rect' || tool === 'line' || tool === 'arrow') {
          if (ev.type === 'pointerdown') { pushHistory(); drawing = { s0: s, tool }; }
          else if (ev.type === 'pointermove' && drawing) { drawShapeOvl(drawing, s); }
          else if (ev.type === 'pointerup' && drawing) {
            clearOvl();
            drawShapeImage(layers[sel].ctx, drawing.s0, s, drawing.tool, false);
            drawing = null; draw(); commit();
          }
        } else if (tool === 'crop') {
          if (ev.type === 'pointerdown') { pushHistory(); drawing = { s0: s }; }
          else if (ev.type === 'pointermove' && drawing) { clearOvl(); drawCrop(drawing, s); }
          else if (ev.type === 'pointerup' && drawing) {
            clearOvl(); applyCrop(drawing.s0, s); drawing = null; draw(); commit();
          }
        } else if (tool === 'pan') {
          if (ev.type === 'pointerdown') { drawing = { x: ev.clientX, y: ev.clientY, tx, ty }; }
          else if (ev.type === 'pointermove' && drawing) { tx = drawing.tx + (ev.clientX - drawing.x); ty = drawing.ty + (ev.clientY - drawing.y); draw(); }
          else if (ev.type === 'pointerup') drawing = null;
        }
      };
      const beginStroke = (dr, s) => { const g = dr.l.ctx; g.beginPath(); g.moveTo(s.x, s.y); };
      const drawStroke = (dr, s) => {
        const g = dr.l.ctx;
        g.strokeStyle = color; g.lineWidth = size; g.lineCap = 'round'; g.lineJoin = 'round';
        if (dr.mode === 'eraser') { g.globalCompositeOperation = 'destination-out'; g.strokeStyle = 'rgba(0,0,0,1)'; }
        g.lineTo(s.x, s.y); g.stroke();
        if (dr.mode === 'eraser') g.globalCompositeOperation = 'source-over';
      };
      const drawShapeImage = (g, s0, s, st, fill) => {
        g.strokeStyle = color; g.fillStyle = color; g.lineWidth = size; g.lineCap = 'round'; g.lineJoin = 'round';
        if (st === 'rect') {
          const x = Math.min(s0.x, s.x), y = Math.min(s0.y, s.y), w = Math.abs(s.x - s0.x), h = Math.abs(s.y - s0.y);
          if (fill) { g.globalAlpha = 0.35; g.fillRect(x, y, w, h); g.globalAlpha = 1; }
          g.strokeRect(x, y, w, h);
        } else {
          g.beginPath();
          g.moveTo(s0.x, s0.y);
          g.lineTo(s.x, s.y);
          g.stroke();
          if (st === 'arrow') {
            const ang = Math.atan2(s.y - s0.y, s.x - s0.x);
            const hx = Math.max(10, size * 1.6);
            g.beginPath();
            g.moveTo(s.x, s.y);
            g.lineTo(s.x - hx * Math.cos(ang - 0.4), s.y - hx * Math.sin(ang - 0.4));
            g.moveTo(s.x, s.y);
            g.lineTo(s.x - hx * Math.cos(ang + 0.4), s.y - hx * Math.sin(ang + 0.4));
            g.stroke();
          }
        }
      };
      const drawShapeOvl = (dr, s) => {
        clearOvl();
        octx.setTransform(zoom, 0, 0, zoom, tx, ty);
        octx.globalAlpha = 0.5;
        drawShapeImage(octx, dr.s0, s, dr.tool, true);
        octx.globalAlpha = 1;
        octx.setTransform(1, 0, 0, 1, 0, 0);
      };
      const drawCrop = (dr, s) => {
        clearOvl();
        octx.setTransform(1, 0, 0, 1, 0, 0);
        octx.fillStyle = 'rgba(0,0,0,0.45)';
        octx.fillRect(0, 0, ovl.width, ovl.height);
        const a = { x: tx + Math.min(dr.s0.x, s.x) * zoom, y: ty + Math.min(dr.s0.y, s.y) * zoom, w: Math.abs(s.x - dr.s0.x) * zoom, h: Math.abs(s.y - dr.s0.y) * zoom };
        octx.clearRect(a.x, a.y, a.w, a.h);
        octx.strokeStyle = '#fff'; octx.lineWidth = Math.max(1, zoom);
        octx.strokeRect(a.x, a.y, a.w, a.h);
      };
      const applyCrop = (a, b) => {
        const x = Math.max(0, Math.min(a.x, b.x)), y = Math.max(0, Math.min(a.y, b.y));
        const w = Math.max(1, Math.abs(b.x - a.x)), h = Math.max(1, Math.abs(b.y - a.y));
        const cw = Math.min(w, BGW - x), ch = Math.min(h, BGH - y);
        const nb = document.createElement('canvas'); nb.width = cw; nb.height = ch;
        if (bg) nb.getContext('2d').drawImage(bg, -x, -y);
        const old = layers.slice();
        layers.splice(0, layers.length);
        old.forEach((l, i) => {
          const c2 = document.createElement('canvas'); c2.width = cw; c2.height = ch;
          c2.getContext('2d').drawImage(l.cv, -x, -y);
          layers.push({ cv: c2, ctx: c2.getContext('2d'), visible: l.visible, opacity: l.opacity });
        });
        bg = nb; BGW = cw; BGH = ch;
        fit();
      };
      const rotateBoth = (dir) => {
        const rot = (cv) => {
          const out = document.createElement('canvas');
          out.width = cv.height; out.height = cv.width;
          const g = out.getContext('2d');
          g.translate(out.width, 0); g.rotate(Math.PI / 2);
          g.drawImage(cv, 0, 0);
          return out;
        };
        if (bg) bg = rot(bg);
        layers.forEach((l) => { const n = rot(l.cv); l.cv = n; l.ctx = n.getContext('2d'); });
        const t = BGW; BGW = BGH; BGH = t;
        fit();
      };

      ovl.style.pointerEvents = interactive ? 'auto' : 'none';
      ovl.addEventListener('pointerdown', (ev) => { ovl.setPointerCapture(ev.pointerId); pt(ev); });
      ovl.addEventListener('pointermove', pt);
      ovl.addEventListener('pointerup', pt);
      ovl.addEventListener('wheel', (ev) => {
        ev.preventDefault();
        const r = ovl.getBoundingClientRect();
        const mx = ev.clientX - r.left - tx, my = ev.clientY - r.top - ty;
        const k = ev.deltaY < 0 ? 1.15 : 1 / 1.15;
        const nz = Math.min(8, Math.max(0.05, zoom * k));
        tx = ev.clientX - r.left - mx * (nz / zoom);
        ty = ev.clientY - r.top - my * (nz / zoom);
        zoom = nz;
        scaleCv(r.width, r.height); draw(); clearOvl();
      });

      let bgInit = dataUrl(p.value) || null;
      const loadBg = (src, force, done) => {
        const fin = () => { fit(); draw(); buildLayersUI(); if (done) done(); };
        if (!src) {
          const c0 = document.createElement('canvas');
          c0.width = BGW; c0.height = BGH;
          bg = c0; bgSrc = '';
          const g = bg.getContext('2d');
          const grad = g.createLinearGradient(0, 0, BGW, BGH);
          grad.addColorStop(0, '#eef2ff'); grad.addColorStop(1, '#fdf2f8');
          g.fillStyle = grad; g.fillRect(0, 0, BGW, BGH);
          g.fillStyle = 'rgba(99,102,241,0.14)';
          g.beginPath(); g.arc(BGW * 0.3, BGH * 0.4, BGH * 0.25, 0, Math.PI * 2); g.fill();
          g.fillStyle = 'rgba(245,158,11,0.16)';
          g.beginPath(); g.arc(BGW * 0.72, BGH * 0.62, BGH * 0.3, 0, Math.PI * 2); g.fill();
          layers.splice(0, layers.length);
          for (let i = 0; i < nLayers; i++) layers.push(mkLayer(BGW, BGH));
          fin();
          return;
        }
        const im = new Image();
        im.onload = () => {
          let w = im.naturalWidth || 700, h = im.naturalHeight || 400;
          const s = Math.min(1, 4096 / w, 4096 / h, Math.sqrt(1.6e7 / (w * h)));
          w = Math.max(1, Math.round(w * s)); h = Math.max(1, Math.round(h * s));
          BGW = w; BGH = h;
          bgSrc = src;
          bg = document.createElement('canvas');
          bg.width = BGW; bg.height = BGH;
          bg.getContext('2d').drawImage(im, 0, 0, BGW, BGH);
          layers.splice(0, layers.length);
          for (let i = 0; i < nLayers; i++) layers.push(mkLayer(BGW, BGH));
          fin();
        };
        im.onerror = () => toast('Impossible de charger cette image', 'error');
        im.src = src;
      };

      loadBg(bgInit);
      requestAnimationFrame(() => fit());
      window.addEventListener('resize', fit);
      if (window.ResizeObserver) {
        const ro = new ResizeObserver(() => {
          if (stage.clientWidth > 0 && stage.clientHeight > 0) fit();
        });
        ro.observe(stage);
      }

      if (!interactive && bgInit) {
        // sortie seule : on ne monte que l'image, pas la barre d'outils
        bar.hidden = true;
      }
    }
  });

  register('audio', {
    mount(c) {
      const p = c.props;
      const holder = document.createElement('div');
      holder.className = 'mg-media';
      holder.innerHTML = makeLabel(p, c);
      c.el.appendChild(holder);

      const audio = document.createElement('audio');
      audio.className = 'mg-media-player';
      audio.controls = true;
      const src0 = dataUrl(p.value);
      if (src0) audio.src = src0;
      audio.volume = 0.8;
      c.getValue = () => audio.src;
      c.apply = (patch) => {
        if (patch.value != null && dataUrl(patch.value) && audio.src !== patch.value) {
          audio.src = patch.value;
        }
        if (patch.visible != null) holder.hidden = !patch.visible;
      };

      if (c.input) {
        const file = document.createElement('input');
        file.type = 'file';
        file.accept = 'audio/*';
        file.hidden = true;
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'mg-btn mg-btn-secondary';
        const setAudioBtnText = () => { btn.textContent = t('media_choose_audio', 'Choisir un audio'); };
        setAudioBtnText();
        onI18n(setAudioBtnText);
        btn.addEventListener('click', () => file.click());
        file.addEventListener('change', (e) => {
          const f = e.target.files && e.target.files[0];
          if (!f) return;
          readFile(f, (url) => {
            audio.src = url;
            emit(c, 'change', url);
          });
        });
        const row = document.createElement('div');
        row.className = 'mg-media-actions';
        row.appendChild(btn);
        row.appendChild(file);
        holder.appendChild(row);
      } else {
        holder.appendChild(audio);
        const controls = playerButtons(c);
        holder.appendChild(controls);
        if (p.live === true) {
          let rec = null;
          const recBtn = document.createElement('button');
          recBtn.type = 'button';
          recBtn.className = 'mg-btn mg-btn-primary mg-live-btn';
          recBtn.textContent = 'Record';
          recBtn.addEventListener('click', async () => {
            if (rec) {
              rec.stop();
              rec = null;
              return;
            }
            let stream;
            try { stream = await navigator.mediaDevices.getUserMedia({ audio: true }); }
            catch { toast('micro inaccessible', 'error'); return; }
            audio.srcObject = stream;
            audio.play();
            rec = new MediaRecorder(stream);
            rec.onstart = () => emit(c, 'play', null);
            rec.ondataavailable = (e) => { if (e.data && e.data.size) sendStream(c, e.data); };
            rec.onstop = () => {
              audio.srcObject = null;
              emit(c, 'stop', null);
            };
            rec.start(500);
            recBtn.textContent = 'Stop';
            rec.onstop = () => { recBtn.textContent = 'Record'; };
          });
          holder.appendChild(recBtn);
        }
      }
    }
  });

  register('video', {
    mount(c) {
      const p = c.props;
      const holder = document.createElement('div');
      holder.className = 'mg-media';
      holder.innerHTML = makeLabel(p, c);
      c.el.appendChild(holder);

      const video = document.createElement('video');
      video.className = 'mg-media-player';
      video.controls = true;
      const src0 = dataUrl(p.value);
      if (src0) video.src = src0;
      c.getValue = () => video.src;
      c.apply = (patch) => {
        if (patch.value != null && dataUrl(patch.value) && video.src !== patch.value) {
          video.src = patch.value;
        }
        if (patch.visible != null) holder.hidden = !patch.visible;
      };

      if (c.input) {
        const file = document.createElement('input');
        file.type = 'file';
        file.accept = 'video/*';
        file.hidden = true;
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'mg-btn mg-btn-secondary';
        const setVideoBtnText = () => { btn.textContent = t('media_choose_video', 'Choisir une vidéo'); };
        setVideoBtnText();
        onI18n(setVideoBtnText);
        btn.addEventListener('click', () => file.click());
        file.addEventListener('change', (e) => {
          const f = e.target.files && e.target.files[0];
          if (!f) return;
          readFile(f, (url) => {
            video.src = url;
            emit(c, 'change', url);
          });
        });
        const row = document.createElement('div');
        row.className = 'mg-media-actions';
        row.appendChild(btn);
        row.appendChild(file);
        holder.appendChild(row);
      } else {
        holder.appendChild(video);
        holder.appendChild(playerButtons(c));
        if (p.live === true) {
          let camRec = null;
          const camBtn = document.createElement('button');
          camBtn.type = 'button';
          camBtn.className = 'mg-btn mg-btn-primary mg-live-btn';
          camBtn.textContent = 'Camera';
          camBtn.addEventListener('click', async () => {
            if (camRec) {
              camRec.stop();
              camRec = null;
              return;
            }
            let stream;
            try { stream = await navigator.mediaDevices.getUserMedia({ video: true }); }
            catch { toast('caméra inaccessible', 'error'); return; }
            video.srcObject = stream;
            video.play();
            camRec = new MediaRecorder(stream);
            camRec.onstart = () => emit(c, 'play', null);
            camRec.ondataavailable = (e) => { if (e.data && e.data.size) sendStream(c, e.data); };
            camRec.onstop = () => {
              video.srcObject = null;
              emit(c, 'stop', null);
            };
            camRec.start(1000);
            camBtn.textContent = 'Stop caméra';
          });
          holder.appendChild(camBtn);
        }
      }
    }
  });

  /* ---------- widgets avancés ---------- */

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

  function fmtV(v) {
    const a = Math.abs(v);
    if (a >= 1000) return (v / 1000).toFixed(1) + 'k';
    return (Math.round(v * 100) / 100).toString();
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
    // En mode bar, chaque catégorie i dispose d'un slot [pad.l + i*slot, pad.l + (i+1)*slot]
    // Le centre du slot est à pad.l + (i + 0.5) * slot.
    // En mode line/scatter classique, le point va de pad.l à pad.l + pw.
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

  register('checkbox', {
    mount(c) {
      const p = c.props;
      const box = document.createElement('label');
      box.className = 'mg-checkbox';
      const cb = document.createElement('input');
      cb.type = 'checkbox';
      cb.checked = !!p.value;
      if (p.interactive === false) { cb.disabled = true; box.classList.add('mg-disabled'); }
      const span = document.createElement('span');
      span.textContent = p.label || c.id;
      box.append(cb, span);
      cb.addEventListener('change', () => emit(c, 'change', cb.checked));
      c.el.appendChild(box);
      c.getValue = () => cb.checked;
      c.apply = (patch) => {
        if (patch.value != null && !!patch.value !== cb.checked) cb.checked = !!patch.value;
        if (patch.label != null) span.textContent = patch.label;
        if (patch.visible != null) c.el.hidden = !patch.visible;
        if (patch.disabled != null) { cb.disabled = !!patch.disabled; box.classList.toggle('mg-disabled', !!patch.disabled); }
      };
    }
  });

  register('dropdown', {
    mount(c) {
      const p = c.props;
      const choices = Array.isArray(p.choices) ? p.choices : [];
      const isMulti = !!p.multiple;
      const wrap = document.createElement('div');
      wrap.className = 'mg-field-inner';
      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);
      const sel = document.createElement('select');
      sel.className = 'mg-select' + (isMulti ? ' mg-select-multi' : '');
      if (isMulti) sel.multiple = true;
      choices.forEach((ch) => {
        const o = document.createElement('option');
        o.value = ch.value != null ? String(ch.value) : String(ch.label);
        o.textContent = ch.label != null ? String(ch.label) : String(ch.value);
        sel.appendChild(o);
      });
      if (!isMulti) {
        const e = document.createElement('option');
        e.value = '';
        e.textContent = '\u2014';
        sel.prepend(e);
      }
      const setValue = (v) => {
        const vals = isMulti ? (Array.isArray(v) ? v.map(String) : []) : [v == null ? '' : String(v)];
        Array.from(sel.options).forEach((o) => { o.selected = vals.indexOf(o.value) >= 0; });
      };
      setValue(p.value);
      if (p.interactive === false) sel.disabled = true;
      const read = () => (isMulti ? Array.from(sel.selectedOptions).map((o) => o.value) : sel.value);
      sel.addEventListener('change', () => emit(c, 'change', read()));
      wrap.appendChild(sel);
      if (p.allow_custom) {
        const custom = document.createElement('input');
        custom.className = 'mg-input';
        custom.placeholder = 'valeur libre\u2026';
        custom.addEventListener('input', () => { if (custom.value) emit(c, 'change', custom.value); });
        const row = document.createElement('div');
        row.className = 'mg-media-actions';
        row.appendChild(custom);
        wrap.appendChild(row);
      }
      c.el.appendChild(wrap);
      c.getValue = () => read();
      c.apply = (patch) => {
        if (patch.value != null) setValue(patch.value);
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
        if (patch.disabled != null) sel.disabled = !!patch.disabled;
      };
    }
  });

  function dateTimeRegister(kind, pickerType) {
    register(kind, {
      mount(c) {
        const p = c.props;
        const wrap = document.createElement('div');
        wrap.className = 'mg-field-inner';
        wrap.innerHTML = '<div class="mg-label"><span>' + esc(p.label || c.id) + '</span></div>';
        const inp = document.createElement('input');
        inp.className = 'mg-input';
        inp.type = pickerType;
        if (p.min) inp.min = p.min;
        if (p.max) inp.max = p.max;
        if (p.value) inp.value = p.value;
        if (p.interactive === false) inp.disabled = true;
        inp.addEventListener('change', () => emit(c, 'change', inp.value));
        wrap.appendChild(inp);
        c.el.appendChild(wrap);
        c.getValue = () => inp.value;
        c.apply = (patch) => {
          if (patch.value != null && String(patch.value) !== inp.value) inp.value = patch.value;
          if (patch.visible != null) c.el.hidden = !patch.visible;
        };
      }
    });
  }
  dateTimeRegister('date', 'date');
  dateTimeRegister('time', 'time');

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
      let sort = null; // { i, dir } — colonne triée et direction (+1 asc / -1 desc)

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

  register('gallery', {
    mount(c) {
      const p = c.props;
      const holder = document.createElement('div');
      holder.className = 'mg-gallery';
      
      // Styling custom dimensions
      if (p.height) holder.style.height = typeof p.height === 'number' ? p.height + 'px' : p.height;
      if (p.min_height) holder.style.minHeight = typeof p.min_height === 'number' ? p.min_height + 'px' : p.min_height;
      if (p.max_height) holder.style.maxHeight = typeof p.max_height === 'number' ? p.max_height + 'px' : p.max_height;
      
      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      if (p.label === '' || p.label === false) lab.style.display = 'none';
      holder.appendChild(lab);

      const scrollBox = document.createElement('div');
      scrollBox.className = 'mg-gallery-scroll';
      if (p.height || p.max_height || p.rows) {
        scrollBox.classList.add('mg-scrollable');
      }

      const grid = document.createElement('div');
      grid.className = 'mg-gallery-grid';
      const cols = p.columns || 3;
      grid.style.gridTemplateColumns = 'repeat(' + cols + ', minmax(0, 1fr))';
      scrollBox.appendChild(grid);
      holder.appendChild(scrollBox);

      const file = document.createElement('input');
      file.type = 'file';
      file.accept = 'image/*';
      file.multiple = true;
      file.hidden = true;
      holder.appendChild(file);

      let items = Array.isArray(p.value) ? p.value.slice() : [];
      const itemObj = (it) => (it && typeof it === 'object') ? it : { image: it };
      const images = () => items.map((it) => itemObj(it).image);

      // Lightbox preview modal
      const openLightbox = (url, caption) => {
        if (!p.allow_preview) return;
        const modal = document.createElement('div');
        modal.className = 'mg-gallery-lightbox';
        modal.innerHTML = `
          <div class="mg-lightbox-backdrop"></div>
          <div class="mg-lightbox-content">
            <button class="mg-lightbox-close" title="Fermer (Échap)">✕</button>
            <img src="${url}" alt="${esc(caption || '')}" class="mg-lightbox-img">
            ${caption ? `<div class="mg-lightbox-caption">${esc(caption)}</div>` : ''}
          </div>
        `;
        const close = () => {
          modal.classList.add('closing');
          setTimeout(() => modal.remove(), 200);
          document.removeEventListener('keydown', onKey);
        };
        const onKey = (e) => { if (e.key === 'Escape') close(); };
        modal.querySelector('.mg-lightbox-backdrop').addEventListener('click', close);
        modal.querySelector('.mg-lightbox-close').addEventListener('click', close);
        document.addEventListener('keydown', onKey);
        document.body.appendChild(modal);
        requestAnimationFrame(() => modal.classList.add('active'));
      };

      const rebuild = () => {
        grid.innerHTML = '';
        let displayList = items;
        if (p.limit && typeof p.limit === 'number' && p.limit > 0) {
          displayList = items.slice(0, p.limit);
        }

        displayList.forEach((it, i) => {
          const o = itemObj(it);
          const cell = document.createElement('figure');
          cell.className = 'mg-gallery-item';
          
          const img = document.createElement('img');
          img.src = o.image || '';
          img.loading = 'lazy';
          img.alt = o.caption || '';
          if (p.object_fit) img.style.objectFit = p.object_fit;
          if (p.item_height) {
            img.style.height = typeof p.item_height === 'number' ? p.item_height + 'px' : p.item_height;
          }
          if (p.item_width) {
            img.style.width = typeof p.item_width === 'number' ? p.item_width + 'px' : p.item_width;
          }
          if (p.aspect_ratio) {
            img.style.aspectRatio = p.aspect_ratio;
          }
          cell.appendChild(img);

          if (o.caption) {
            const cap = document.createElement('figcaption');
            cap.textContent = o.caption;
            cap.title = o.caption;
            cell.appendChild(cap);
          }

          if (p.interactive) {
            cell.classList.add('del');
            const del = document.createElement('button');
            del.type = 'button';
            del.className = 'mg-ico';
            del.textContent = '✕';
            del.title = 'Retirer';
            del.addEventListener('click', (e) => {
              e.stopPropagation();
              items.splice(i, 1);
              rebuild();
              emit(c, 'change', images());
            });
            cell.appendChild(del);
          }

          cell.addEventListener('click', () => {
            emit(c, 'click', i);
            openLightbox(o.image, o.caption);
          });
          grid.appendChild(cell);
        });
      };

      file.addEventListener('change', (e) => {
        const fs = e.target.files ? Array.from(e.target.files) : [];
        let rem = fs.length;
        if (!rem) return;
        fs.forEach((f) => readFile(f, (url) => {
          items.unshift({ image: url, caption: f.name });
          if (--rem === 0) { rebuild(); emit(c, 'change', items); }
        }));
        file.value = '';
      });

      if (p.interactive && p.upload) {
        const addBtn = document.createElement('button');
        addBtn.type = 'button';
        addBtn.className = 'mg-btn mg-btn-secondary mg-gallery-upload-btn';
        addBtn.textContent = '+ Ajouter une image';
        addBtn.addEventListener('click', () => file.click());
        holder.appendChild(addBtn);

        const gridDrop = (e) => {
          e.preventDefault();
          grid.classList.remove('hover');
          const fs = e.dataTransfer.files ? Array.from(e.dataTransfer.files) : [];
          let rem = fs.length;
          if (!rem) return;
          fs.forEach((f) => readFile(f, (url) => {
            items.unshift({ image: url, caption: f.name });
            if (--rem === 0) { rebuild(); emit(c, 'change', items); }
          }));
        };
        grid.addEventListener('dragover', (e) => { e.preventDefault(); grid.classList.add('hover'); });
        grid.addEventListener('dragleave', () => grid.classList.remove('hover'));
        grid.addEventListener('drop', gridDrop);
      }

      rebuild();
      c.el.appendChild(holder);
      c.getValue = () => items.slice();
      c.apply = (patch) => {
        if (patch.value != null && Array.isArray(patch.value)) { items = patch.value.slice(); rebuild(); }
        if (patch.label != null) { const s = holder.querySelector('.mg-label span'); if (s) s.textContent = patch.label; }
        if (patch.columns != null) { grid.style.gridTemplateColumns = 'repeat(' + patch.columns + ', minmax(0, 1fr))'; }
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('list', {
    mount(c) {
      const p = c.props;
      const holder = document.createElement('div');
      holder.className = 'mg-sortable';
      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      holder.appendChild(lab);
      const ul = document.createElement('ul');
      ul.className = 'mg-sortable-list';
      holder.appendChild(ul);
      c.el.appendChild(holder);

      const items = Array.isArray(p.items) ? p.items : [];
      const byVal = {};
      items.forEach((it) => { byVal[String(it.value)] = it.label != null ? String(it.label) : String(it.value); });
      let order = Array.isArray(p.value) ? p.value.map(String) : items.map((it) => String(it.value));
      items.forEach((it) => { if (order.indexOf(String(it.value)) < 0) order.push(String(it.value)); });
      const disabled = p.interactive === false;
      let dragIdx = -1;

      const render = () => {
        ul.innerHTML = '';
        order.forEach((v, i) => {
          const li = document.createElement('li');
          li.className = 'mg-sortable-item';
          li.draggable = !disabled;
          li.dataset.idx = i;
          li.innerHTML = '<span class="mg-grip">\u22ef\u22ee</span><span class="mg-sortable-label">' + esc(byVal[v] || v) + '</span>';
          if (disabled) { ul.appendChild(li); return; }
          li.addEventListener('dragstart', (e) => {
            dragIdx = i;
            e.dataTransfer.effectAllowed = 'move';
            e.dataTransfer.setData('text/plain', String(v));
            li.classList.add('dragging');
          });
          li.addEventListener('dragend', () => { li.classList.remove('dragging'); dragIdx = -1; });
          li.addEventListener('dragover', (e) => { e.preventDefault(); e.dataTransfer.dropEffect = 'move'; });
          li.addEventListener('drop', (e) => {
            e.preventDefault();
            const to = +li.dataset.idx;
            if (dragIdx < 0 || dragIdx === to) return;
            const moved = order.splice(dragIdx, 1)[0];
            order.splice(to, 0, moved);
            dragIdx = -1;
            render();
            emit(c, 'change', order.slice());
          });
          ul.appendChild(li);
        });
      };
      render();
      c.getValue = () => order.slice();
      c.apply = (patch) => {
        if (patch.value != null && Array.isArray(patch.value)) { order = patch.value.map(String); render(); }
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

  register('button', {
    mount(c) {
      const p = c.props;
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'mg-btn ' + (p.variant === 'secondary' ? 'mg-btn-secondary' : 'mg-btn-primary');
      btn.textContent = p.label || c.id;
      c.el.appendChild(btn);
      if (p.primary) runButton = btn;
      btn.addEventListener('click', () => emit(c, 'click', null));
      c.apply = (patch) => {
        if (patch.label != null) btn.textContent = patch.label;
        if (patch.visible != null) c.el.hidden = !patch.visible;
        if (patch.disabled != null) { btn.disabled = !!patch.disabled; c.el.classList.toggle('mg-disabled', !!patch.disabled); }
      };
    }
  });

  register('chatbot', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-chatbot-wrap';
      if (p.label) {
        const lbl = document.createElement('div');
        lbl.className = 'mg-label';
        lbl.style.padding = '10px 14px';
        lbl.style.borderBottom = '1px solid var(--mg-border)';
        lbl.style.marginBottom = '0';
        lbl.innerHTML = '<span>' + esc(p.label) + '</span>';
        wrap.appendChild(lbl);
      }
      const hist = document.createElement('div');
      hist.className = 'mg-chatbot-history';
      if (p.height) hist.style.maxHeight = p.height + 'px';
      wrap.appendChild(hist);

      let messages = Array.isArray(p.value) ? p.value.slice() : [];

      function renderMsg(m) {
        const row = document.createElement('div');
        const role = m.role || 'assistant';
        row.className = 'mg-chat-row mg-' + (role === 'user' ? 'user' : 'assistant');
        const bubble = document.createElement('div');
        bubble.className = 'mg-chat-bubble';
        bubble.innerHTML = markdown(m.content || '');
        row.appendChild(bubble);
        return row;
      }

      function rebuild() {
        hist.innerHTML = '';
        if (messages.length === 0) {
          const empty = document.createElement('div');
          empty.className = 'mg-chatbot-empty';
          empty.textContent = p.placeholder || 'Commencez la conversation...';
          hist.appendChild(empty);
          return;
        }
        messages.forEach((m) => hist.appendChild(renderMsg(m)));
        hist.scrollTop = hist.scrollHeight;
      }

      rebuild();
      c.el.appendChild(wrap);

      c.apply = (patch) => {
        if (patch.value != null) {
          if (Array.isArray(patch.value)) {
            messages = patch.value.slice();
            rebuild();
          }
        }
        if (patch.append != null && typeof patch.append === 'string') {
          // Streaming de token vers le dernier message assistant (ou création si absent)
          if (messages.length === 0 || messages[messages.length - 1].role !== 'assistant') {
            messages.push({ role: 'assistant', content: patch.append });
            rebuild();
          } else {
            messages[messages.length - 1].content += patch.append;
            const rows = hist.querySelectorAll('.mg-chat-row');
            if (rows.length > 0) {
              const lastBubble = rows[rows.length - 1].querySelector('.mg-chat-bubble');
              if (lastBubble) {
                lastBubble.innerHTML = markdown(messages[messages.length - 1].content);
                hist.scrollTop = hist.scrollHeight;
              }
            }
          }
        }
        if (patch.label != null) {
          const lbl = wrap.querySelector('.mg-label span');
          if (lbl) lbl.textContent = patch.label;
        }
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

  register('tabs', {
    mount(c) {
      const p = c.props;
      const headers = p.labels || p.tabs || [];
      let selected = p.selected || 0;

      const bar = document.createElement('nav');
      bar.className = 'mg-tabs-bar';

      const panels = Array.from(c.el.querySelectorAll('.mg-tab-pane'));

      headers.forEach((h, idx) => {
        const labelText = typeof h === 'string' ? h : (h.label || `Tab ${idx + 1}`);
        const iconText = typeof h === 'object' && h.icon ? `<span>${esc(h.icon)}</span> ` : '';
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'mg-tab-btn' + (idx === selected ? ' mg-active' : '');
        btn.dataset.rawLabel = labelText;
        btn.innerHTML = iconText + '<span>' + esc(t(labelText, labelText)) + '</span>';
        btn.addEventListener('click', () => {
          selected = idx;
          bar.querySelectorAll('.mg-tab-btn').forEach((b, i) => b.classList.toggle('mg-active', i === selected));
          panels.forEach((pane, i) => pane.classList.toggle('mg-active', i === selected));
        });
        bar.appendChild(btn);
      });

      onI18n(() => {
        bar.querySelectorAll('.mg-tab-btn').forEach((btn) => {
          const raw = btn.dataset.rawLabel;
          if (raw) {
            const span = btn.querySelector('span:last-child') || btn;
            span.textContent = t(raw, raw);
          }
        });
      });

      c.el.insertBefore(bar, c.el.firstChild);
      panels.forEach((pane, i) => pane.classList.toggle('mg-active', i === selected));

      c.apply = (patch) => {
        if (patch.selected != null && typeof patch.selected === 'number') {
          selected = patch.selected;
          bar.querySelectorAll('.mg-tab-btn').forEach((b, i) => b.classList.toggle('mg-active', i === selected));
          panels.forEach((pane, i) => pane.classList.toggle('mg-active', i === selected));
        }
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('radio', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-radio-group mg-radio-' + (p.direction || 'horizontal') + (p.style === 'pills' ? ' mg-radio-pills' : '');
      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);

      const items = document.createElement('div');
      items.className = 'mg-radio-items';
      wrap.appendChild(items);

      let current = p.value != null ? String(p.value) : '';
      const choices = Array.isArray(p.choices) ? p.choices : [];

      const render = () => {
        items.innerHTML = '';
        choices.forEach((choice, idx) => {
          const item = document.createElement('label');
          item.className = 'mg-radio-item' + (current === choice ? ' sel' : '');
          const isRadio = p.style === 'radio';
          const input = document.createElement('input');
          input.type = 'radio';
          input.name = 'mg_r_' + c.id;
          input.value = choice;
          input.checked = current === choice;
          if (p.interactive === false) input.disabled = true;

          input.addEventListener('change', () => {
            current = choice;
            items.querySelectorAll('.mg-radio-item').forEach((it) => it.classList.remove('sel'));
            item.classList.add('sel');
            emit(c, 'change', current);
          });

          if (!isRadio) {
            input.style.display = 'none';
          }
          item.appendChild(input);
          const txt = document.createElement('span');
          txt.textContent = choice;
          item.appendChild(txt);
          items.appendChild(item);
        });
      };

      render();
      c.el.appendChild(wrap);
      if (p.interactive === false) c.el.classList.add('mg-disabled');

      c.getValue = () => current;
      c.apply = (patch) => {
        if (patch.value != null) {
          current = String(patch.value);
          render();
        }
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
        if (patch.disabled != null) {
          p.interactive = !patch.disabled;
          c.el.classList.toggle('mg-disabled', !!patch.disabled);
          render();
        }
      };
    }
  });

  register('sliderrange', {
    mount(c) {
      const p = c.props;
      const min = typeof p.min === 'number' ? p.min : 0;
      const max = typeof p.max === 'number' ? p.max : 100;
      const step = typeof p.step === 'number' ? p.step : 1;
      const unit = p.unit ? String(p.unit) : '';

      let low = Array.isArray(p.value) && p.value.length > 0 ? Number(p.value[0]) : min;
      let high = Array.isArray(p.value) && p.value.length > 1 ? Number(p.value[1]) : max;

      const wrap = document.createElement('div');
      wrap.className = 'mg-slider-range-wrap';

      const header = document.createElement('div');
      header.className = 'mg-label';
      const titleSpan = document.createElement('span');
      titleSpan.textContent = p.label || c.id;
      const valBadge = document.createElement('span');
      valBadge.className = 'mg-range-badge';
      header.appendChild(titleSpan);
      header.appendChild(valBadge);
      wrap.appendChild(header);

      const trackWrap = document.createElement('div');
      trackWrap.className = 'mg-slider-range-track';
      const activeBar = document.createElement('div');
      activeBar.className = 'mg-slider-range-highlight';
      trackWrap.appendChild(activeBar);

      const inputLow = document.createElement('input');
      inputLow.type = 'range';
      inputLow.className = 'mg-range-thumb mg-range-low';
      inputLow.min = min; inputLow.max = max; inputLow.step = step; inputLow.value = low;

      const inputHigh = document.createElement('input');
      inputHigh.type = 'range';
      inputHigh.className = 'mg-range-thumb mg-range-high';
      inputHigh.min = min; inputHigh.max = max; inputHigh.step = step; inputHigh.value = high;

      if (p.interactive === false) {
        inputLow.disabled = true;
        inputHigh.disabled = true;
        c.el.classList.add('mg-disabled');
      }

      trackWrap.appendChild(inputLow);
      trackWrap.appendChild(inputHigh);
      wrap.appendChild(trackWrap);
      c.el.appendChild(wrap);

      const updateUI = () => {
        const pLow = ((low - min) / (max - min)) * 100;
        const pHigh = ((high - min) / (max - min)) * 100;
        activeBar.style.left = Math.min(pLow, pHigh) + '%';
        activeBar.style.width = Math.abs(pHigh - pLow) + '%';
        valBadge.textContent = `[${low}${unit}, ${high}${unit}]`;
      };

      const onInput = (which) => {
        let v1 = Number(inputLow.value);
        let v2 = Number(inputHigh.value);
        if (which === 'low' && v1 > v2) {
          v1 = v2;
          inputLow.value = v1;
        } else if (which === 'high' && v2 < v1) {
          v2 = v1;
          inputHigh.value = v2;
        }
        low = v1;
        high = v2;
        updateUI();
        emit(c, 'change', [low, high]);
      };

      inputLow.addEventListener('input', () => onInput('low'));
      inputHigh.addEventListener('input', () => onInput('high'));
      updateUI();

      c.getValue = () => [low, high];
      c.apply = (patch) => {
        if (Array.isArray(patch.value)) {
          if (patch.value.length > 0) low = Number(patch.value[0]);
          if (patch.value.length > 1) high = Number(patch.value[1]);
          inputLow.value = low;
          inputHigh.value = high;
          updateUI();
        }
        if (patch.label != null) titleSpan.textContent = patch.label;
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('colorpicker', {
    mount(c) {
      const p = c.props;
      let current = p.value || '#6366f1';
      const presets = Array.isArray(p.presets) ? p.presets : [];

      const wrap = document.createElement('div');
      wrap.className = 'mg-color-picker';

      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);

      const mainRow = document.createElement('div');
      mainRow.className = 'mg-color-main-row';

      const colorInput = document.createElement('input');
      colorInput.type = 'color';
      colorInput.className = 'mg-color-input';
      colorInput.value = current;

      const hexInput = document.createElement('input');
      hexInput.type = 'text';
      hexInput.className = 'mg-input mg-color-hex';
      hexInput.value = current;
      hexInput.spellcheck = false;

      if (p.interactive === false) {
        colorInput.disabled = true;
        hexInput.disabled = true;
        c.el.classList.add('mg-disabled');
      }

      mainRow.appendChild(colorInput);
      mainRow.appendChild(hexInput);
      wrap.appendChild(mainRow);

      const swatches = document.createElement('div');
      swatches.className = 'mg-color-swatches';

      const selectColor = (hex) => {
        current = hex;
        colorInput.value = current;
        hexInput.value = current;
        swatches.querySelectorAll('.mg-color-swatch').forEach((s) => {
          s.classList.toggle('sel', s.dataset.color.toLowerCase() === current.toLowerCase());
        });
        emit(c, 'change', current);
      };

      presets.forEach((hex) => {
        const swatch = document.createElement('button');
        swatch.type = 'button';
        swatch.className = 'mg-color-swatch' + (hex.toLowerCase() === current.toLowerCase() ? ' sel' : '');
        swatch.dataset.color = hex;
        swatch.style.backgroundColor = hex;
        swatch.title = hex;
        if (p.interactive === false) swatch.disabled = true;
        swatch.addEventListener('click', () => selectColor(hex));
        swatches.appendChild(swatch);
      });
      wrap.appendChild(swatches);
      c.el.appendChild(wrap);

      colorInput.addEventListener('input', () => {
        current = colorInput.value;
        hexInput.value = current;
        swatches.querySelectorAll('.mg-color-swatch').forEach((s) => {
          s.classList.toggle('sel', s.dataset.color.toLowerCase() === current.toLowerCase());
        });
        emit(c, 'change', current);
      });

      hexInput.addEventListener('change', () => {
        let v = hexInput.value.trim();
        if (!v.startsWith('#')) v = '#' + v;
        if (/^#[0-9a-fA-F]{6}$/.test(v)) {
          current = v;
          colorInput.value = current;
          swatches.querySelectorAll('.mg-color-swatch').forEach((s) => {
            s.classList.toggle('sel', s.dataset.color.toLowerCase() === current.toLowerCase());
          });
          emit(c, 'change', current);
        } else {
          hexInput.value = current;
        }
      });

      c.getValue = () => current;
      c.apply = (patch) => {
        if (patch.value != null) selectColor(String(patch.value));
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('annotatedimage', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-annotated-wrap';

      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);

      const stage = document.createElement('div');
      stage.className = 'mg-annotated-stage';

      const img = document.createElement('img');
      img.className = 'mg-annotated-img';
      img.alt = p.label || 'annotated';
      stage.appendChild(img);

      const svgOverlay = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
      svgOverlay.setAttribute('class', 'mg-annotated-svg');
      stage.appendChild(svgOverlay);

      wrap.appendChild(stage);
      c.el.appendChild(wrap);

      let currentImg = p.image || '';
      let currentBoxes = Array.isArray(p.boxes) ? p.boxes : [];

      const render = () => {
        if (currentImg) {
          img.src = currentImg;
          img.style.display = 'block';
        } else {
          img.style.display = 'none';
        }

        svgOverlay.innerHTML = '';
        currentBoxes.forEach((b) => {
          if (!Array.isArray(b.box_coords) || b.box_coords.length < 4) return;
          const [ymin, xmin, ymax, xmax] = b.box_coords;
          const color = b.color || '#6366f1';

          const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
          rect.setAttribute('x', `${xmin * 100}%`);
          rect.setAttribute('y', `${ymin * 100}%`);
          rect.setAttribute('width', `${Math.max(0, xmax - xmin) * 100}%`);
          rect.setAttribute('height', `${Math.max(0, ymax - ymin) * 100}%`);
          rect.setAttribute('stroke', color);
          rect.setAttribute('stroke-width', '2.5');
          rect.setAttribute('fill', color + '22');
          svgOverlay.appendChild(rect);

          if (p.show_labels !== false && b.label) {
            const txt = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            txt.setAttribute('x', `${xmin * 100}%`);
            txt.setAttribute('y', `${Math.max(4, ymin * 100 - 1)}%`);
            txt.setAttribute('fill', color);
            txt.setAttribute('font-size', '12');
            txt.setAttribute('font-weight', 'bold');
            txt.setAttribute('class', 'mg-bbox-tag');

            let labelContent = b.label;
            if (p.show_scores !== false && typeof b.score === 'number') {
              labelContent += ` ${Math.round(b.score * 100)}%`;
            }
            txt.textContent = labelContent;
            svgOverlay.appendChild(txt);
          }
        });
      };

      render();

      c.apply = (patch) => {
        if (patch.image != null) currentImg = String(patch.image);
        if (Array.isArray(patch.boxes)) currentBoxes = patch.boxes;
        if (patch.value && typeof patch.value === 'object') {
          if (patch.value.image != null) currentImg = String(patch.value.image);
          if (Array.isArray(patch.value.boxes)) currentBoxes = patch.value.boxes;
        }
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
        render();
      };
    }
  });

  register('imagecomparison', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-imgcomp-wrap';

      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);

      const frame = document.createElement('div');
      frame.className = 'mg-imgcomp-frame';

      const imgAfter = document.createElement('img');
      imgAfter.className = 'mg-imgcomp-img mg-imgcomp-after';
      imgAfter.src = p.after || '';

      const imgBefore = document.createElement('img');
      imgBefore.className = 'mg-imgcomp-img mg-imgcomp-before';
      imgBefore.src = p.before || '';

      const sliderLine = document.createElement('div');
      sliderLine.className = 'mg-imgcomp-divider';
      const handle = document.createElement('div');
      handle.className = 'mg-imgcomp-handle';
      handle.innerHTML = '<span>⇆</span>';
      sliderLine.appendChild(handle);

      const tagBefore = document.createElement('span');
      tagBefore.className = 'mg-imgcomp-badge mg-badge-before';
      tagBefore.textContent = p.before_label || 'Before';

      const tagAfter = document.createElement('span');
      tagAfter.className = 'mg-imgcomp-badge mg-badge-after';
      tagAfter.textContent = p.after_label || 'After';

      frame.appendChild(imgAfter);
      frame.appendChild(imgBefore);
      frame.appendChild(sliderLine);
      frame.appendChild(tagBefore);
      frame.appendChild(tagAfter);
      wrap.appendChild(frame);
      c.el.appendChild(wrap);

      let pos = typeof p.position === 'number' ? p.position : 50;

      // Utilisation d'un clip-path inset : l'image originale reste à 100% de la largeur
      // et ne s'étire ni ne se déforme jamais !
      const setPos = (pct) => {
        pos = Math.max(0, Math.min(100, pct));
        imgBefore.style.clipPath = `inset(0 ${100 - pos}% 0 0)`;
        sliderLine.style.left = pos + '%';
      };

      setPos(pos);

      let isDragging = false;
      const onMove = (e) => {
        if (!isDragging) return;
        const rect = frame.getBoundingClientRect();
        const clientX = e.touches ? e.touches[0].clientX : e.clientX;
        const pct = ((clientX - rect.left) / rect.width) * 100;
        setPos(pct);
      };

      const startDrag = (e) => {
        isDragging = true;
        onMove(e);
      };
      const stopDrag = () => { isDragging = false; };

      frame.addEventListener('mousedown', startDrag);
      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', stopDrag);

      frame.addEventListener('touchstart', startDrag, { passive: true });
      window.addEventListener('touchmove', onMove, { passive: true });
      window.addEventListener('touchend', stopDrag);

      c.apply = (patch) => {
        if (patch.before != null) imgBefore.src = patch.before;
        if (patch.after != null) imgAfter.src = patch.after;
        if (patch.position != null) setPos(Number(patch.position));
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('audiorecorder', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-recorder-wrap';

      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);

      const panel = document.createElement('div');
      panel.className = 'mg-recorder-panel';

      const recBtn = document.createElement('button');
      recBtn.type = 'button';
      recBtn.className = 'mg-btn mg-rec-btn';
      recBtn.innerHTML = '<span class="mg-rec-dot"></span> <span class="mg-rec-text">REC</span>';

      const recText = recBtn.querySelector('.mg-rec-text');
      const setRecBtnText = () => {
        if (recBtn.classList.contains('recording')) {
          recText.textContent = t('rec_stop', 'STOP');
        } else {
          recText.textContent = t('rec_rec', 'REC');
        }
      };
      setRecBtnText();
      onI18n(setRecBtnText);

      const timerSpan = document.createElement('span');
      timerSpan.className = 'mg-rec-timer';
      timerSpan.textContent = '00:00';

      const player = document.createElement('audio');
      player.controls = true;
      player.className = 'mg-rec-player';
      player.style.display = 'none';

      panel.appendChild(recBtn);
      panel.appendChild(timerSpan);
      panel.appendChild(player);
      wrap.appendChild(panel);
      c.el.appendChild(wrap);

      let mediaRecorder = null;
      let chunks = [];
      let timerId = null;
      let seconds = 0;
      let audioDataUrl = '';

      const updateTimer = () => {
        seconds++;
        const mins = String(Math.floor(seconds / 60)).padStart(2, '0');
        const secs = String(seconds % 60).padStart(2, '0');
        timerSpan.textContent = `${mins}:${secs}`;
        if (p.max_duration && seconds >= p.max_duration) {
          stopRecording();
        }
      };

      const startRecording = async () => {
        if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
          toast('Microphone non disponible dans ce navigateur', 'error');
          return;
        }
        try {
          const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
          chunks = [];
          mediaRecorder = new MediaRecorder(stream);
          mediaRecorder.ondataavailable = (e) => {
            if (e.data && e.data.size > 0) chunks.push(e.data);
          };
          mediaRecorder.onstop = () => {
            const blob = new Blob(chunks, { type: 'audio/webm' });
            const reader = new FileReader();
            reader.onloadend = () => {
              audioDataUrl = reader.result;
              player.src = audioDataUrl;
              player.style.display = 'block';
              emit(c, 'change', audioDataUrl);
            };
            reader.readAsDataURL(blob);
            stream.getTracks().forEach((track) => track.stop());
          };
          mediaRecorder.start();
          seconds = 0;
          timerSpan.textContent = '00:00';
          timerId = setInterval(updateTimer, 1000);
          recBtn.classList.add('recording');
          recText.textContent = t('rec_stop', 'STOP');
        } catch (err) {
          toast('Accès micro refusé ou impossible', 'error');
        }
      };

      const stopRecording = () => {
        if (mediaRecorder && mediaRecorder.state !== 'inactive') {
          mediaRecorder.stop();
        }
        clearInterval(timerId);
        recBtn.classList.remove('recording');
        recText.textContent = t('rec_rec', 'REC');
      };

      recBtn.addEventListener('click', () => {
        if (recBtn.classList.contains('recording')) {
          stopRecording();
        } else {
          startRecording();
        }
      });

      c.getValue = () => audioDataUrl;
      c.apply = (patch) => {
        if (patch.value != null) {
          audioDataUrl = String(patch.value);
          player.src = audioDataUrl;
          player.style.display = 'block';
        }
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  /* ---------- Phase 8 Lot 4: HighlightedText / CodeDiff / Model3D ---------- */

  register('highlightedtext', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-highlighted-wrap';

      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);

      const legend = document.createElement('div');
      legend.className = 'mg-highlighted-legend';
      wrap.appendChild(legend);

      const content = document.createElement('div');
      content.className = 'mg-highlighted-content';
      wrap.appendChild(content);

      c.el.appendChild(wrap);

      let currentSegs = Array.isArray(p.segments) ? p.segments : [];
      let colorMap = p.color_map && typeof p.color_map === 'object' ? { ...p.color_map } : {};

      const defaultPalette = ['#6366f1', '#10b981', '#f59e0b', '#ec4899', '#06b6d4', '#8b5cf6', '#ef4444', '#14b8a6'];
      let colorIdx = 0;
      const getColor = (tag) => {
        if (!colorMap[tag]) {
          colorMap[tag] = defaultPalette[colorIdx % defaultPalette.length];
          colorIdx++;
        }
        return colorMap[tag];
      };

      const render = () => {
        content.innerHTML = '';
        legend.innerHTML = '';

        const activeTags = new Set();
        currentSegs.forEach((seg) => {
          if (!seg || typeof seg.text !== 'string') return;
          if (seg.label) {
            activeTags.add(seg.label);
            const color = getColor(seg.label);
            const span = document.createElement('mark');
            span.className = 'mg-highlighted-mark';
            span.style.backgroundColor = color + '22';
            span.style.borderBottomColor = color;

            const textNode = document.createTextNode(seg.text);
            const tagBadge = document.createElement('span');
            tagBadge.className = 'mg-highlighted-tag';
            tagBadge.style.backgroundColor = color;
            tagBadge.textContent = seg.label;

            span.appendChild(textNode);
            span.appendChild(tagBadge);
            content.appendChild(span);
          } else {
            content.appendChild(document.createTextNode(seg.text));
          }
        });

        if (p.show_legend !== false && activeTags.size > 0) {
          legend.style.display = 'flex';
          activeTags.forEach((tag) => {
            const item = document.createElement('div');
            item.className = 'mg-highlighted-legend-item';
            const dot = document.createElement('span');
            dot.className = 'mg-highlighted-legend-dot';
            dot.style.backgroundColor = getColor(tag);
            const lbl = document.createElement('span');
            lbl.textContent = tag;
            item.appendChild(dot);
            item.appendChild(lbl);
            legend.appendChild(item);
          });
        } else {
          legend.style.display = 'none';
        }
      };

      render();

      c.apply = (patch) => {
        if (Array.isArray(patch.segments)) currentSegs = patch.segments;
        if (Array.isArray(patch.value)) {
          currentSegs = patch.value.map((x) => (Array.isArray(x) ? { text: x[0], label: x[1] } : x));
        }
        if (patch.color_map && typeof patch.color_map === 'object') {
          colorMap = { ...colorMap, ...patch.color_map };
        }
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
        render();
      };
    }
  });

  register('codediff', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-diff-wrap';

      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);

      const pre = document.createElement('div');
      pre.className = 'mg-diff-body';
      wrap.appendChild(pre);

      c.el.appendChild(wrap);

      let oldCode = p.old_code || '';
      let newCode = p.new_code || '';

      const computeDiff = (oldS, newS) => {
        const oldLines = oldS.split('\n');
        const newLines = newS.split('\n');
        const diff = [];
        let i = 0, j = 0;
        let lineOld = 1, lineNew = 1;

        while (i < oldLines.length || j < newLines.length) {
          if (i < oldLines.length && j < newLines.length && oldLines[i] === newLines[j]) {
            diff.push({ type: 'same', text: oldLines[i], oldNo: lineOld++, newNo: lineNew++ });
            i++; j++;
          } else if (j < newLines.length && (!oldLines.slice(i).includes(newLines[j]) || (oldLines[i] && newLines.slice(j).includes(oldLines[i])))) {
            diff.push({ type: 'add', text: newLines[j], oldNo: '', newNo: lineNew++ });
            j++;
          } else if (i < oldLines.length) {
            diff.push({ type: 'del', text: oldLines[i], oldNo: lineOld++, newNo: '' });
            i++;
          } else {
            diff.push({ type: 'add', text: newLines[j], oldNo: '', newNo: lineNew++ });
            j++;
          }
        }
        return diff;
      };

      const render = () => {
        pre.innerHTML = '';
        const lines = computeDiff(oldCode, newCode);
        lines.forEach((l) => {
          const row = document.createElement('div');
          row.className = 'mg-diff-line mg-diff-' + l.type;

          const numOld = document.createElement('span');
          numOld.className = 'mg-diff-no';
          numOld.textContent = l.oldNo;

          const numNew = document.createElement('span');
          numNew.className = 'mg-diff-no';
          numNew.textContent = l.newNo;

          const marker = document.createElement('span');
          marker.className = 'mg-diff-marker';
          marker.textContent = l.type === 'add' ? '+' : l.type === 'del' ? '-' : ' ';

          const code = document.createElement('span');
          code.className = 'mg-diff-text';
          code.textContent = l.text;

          row.appendChild(numOld);
          row.appendChild(numNew);
          row.appendChild(marker);
          row.appendChild(code);
          pre.appendChild(row);
        });
      };

      render();

      c.apply = (patch) => {
        if (patch.old_code != null) oldCode = String(patch.old_code);
        if (patch.new_code != null) newCode = String(patch.new_code);
        if (patch.value && typeof patch.value === 'object') {
          if (patch.value.old_code != null) oldCode = String(patch.value.old_code);
          if (patch.value.new_code != null) newCode = String(patch.value.new_code);
        }
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
        render();
      };
    }
  });

  register('model3d', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-model3d-wrap';

      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);

      const container = document.createElement('div');
      container.className = 'mg-model3d-box';

      const canvas = document.createElement('canvas');
      canvas.className = 'mg-model3d-canvas';
      canvas.width = 400;
      canvas.height = 300;
      container.appendChild(canvas);

      const hint = document.createElement('div');
      hint.className = 'mg-model3d-hint';
      const setHintText = () => { hint.textContent = t('m3d_hint', '🖱️ Glisser pour tourner · Molette pour zoomer'); };
      setHintText();
      onI18n(setHintText);
      container.appendChild(hint);

      wrap.appendChild(container);
      c.el.appendChild(wrap);

      // WebGL Renderer 3D natif (Cube / OBJ simple)
      const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
      let rotX = 0.4;
      let rotY = 0.6;
      let zoom = 2.8;

      const vsSource = `
        attribute vec3 aPos;
        attribute vec3 aNormal;
        uniform mat4 uMVP;
        uniform mat4 uModel;
        varying vec3 vNormal;
        void main() {
          gl_Position = uMVP * vec4(aPos, 1.0);
          vNormal = mat3(uModel) * aNormal;
        }
      `;
      const fsSource = `
        precision mediump float;
        varying vec3 vNormal;
        void main() {
          vec3 light = normalize(vec3(0.5, 1.0, 0.8));
          float diff = max(dot(normalize(vNormal), light), 0.2);
          vec3 baseCol = vec3(0.39, 0.40, 0.95);
          gl_FragColor = vec4(baseCol * diff, 1.0);
        }
      `;

      function createShader(gl, type, src) {
        const s = gl.createShader(type);
        gl.shaderSource(s, src);
        gl.compileShader(s);
        return s;
      }

      let prog = null;
      let posBuf = null;
      let normBuf = null;
      let idxBuf = null;
      let indexCount = 0;

      if (gl) {
        prog = gl.createProgram();
        gl.attachShader(prog, createShader(gl, gl.VERTEX_SHADER, vsSource));
        gl.attachShader(prog, createShader(gl, gl.FRAGMENT_SHADER, fsSource));
        gl.linkProgram(prog);

        // Cube par défaut
        const vertices = new Float32Array([
          -0.7,-0.7, 0.7,  0.7,-0.7, 0.7,  0.7, 0.7, 0.7, -0.7, 0.7, 0.7,
          -0.7,-0.7,-0.7, -0.7, 0.7,-0.7,  0.7, 0.7,-0.7,  0.7,-0.7,-0.7,
          -0.7, 0.7,-0.7, -0.7, 0.7, 0.7,  0.7, 0.7, 0.7,  0.7, 0.7,-0.7,
          -0.7,-0.7,-0.7,  0.7,-0.7,-0.7,  0.7,-0.7, 0.7, -0.7,-0.7, 0.7,
           0.7,-0.7,-0.7,  0.7, 0.7,-0.7,  0.7, 0.7, 0.7,  0.7,-0.7, 0.7,
          -0.7,-0.7,-0.7, -0.7,-0.7, 0.7, -0.7, 0.7, 0.7, -0.7, 0.7,-0.7
        ]);
        const normals = new Float32Array([
           0, 0, 1,   0, 0, 1,   0, 0, 1,   0, 0, 1,
           0, 0,-1,   0, 0,-1,   0, 0,-1,   0, 0,-1,
           0, 1, 0,   0, 1, 0,   0, 1, 0,   0, 1, 0,
           0,-1, 0,   0,-1, 0,   0,-1, 0,   0,-1, 0,
           1, 0, 0,   1, 0, 0,   1, 0, 0,   1, 0, 0,
          -1, 0, 0,  -1, 0, 0,  -1, 0, 0,  -1, 0, 0
        ]);
        const indices = new Uint16Array([
           0, 1, 2,   0, 2, 3,    4, 5, 6,   4, 6, 7,
           8, 9,10,   8,10,11,   12,13,14,  12,14,15,
          16,17,18,  16,18,19,   20,21,22,  20,22,23
        ]);
        indexCount = indices.length;

        posBuf = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, posBuf);
        gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

        normBuf = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, normBuf);
        gl.bufferData(gl.ARRAY_BUFFER, normals, gl.STATIC_DRAW);

        idxBuf = gl.createBuffer();
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, idxBuf);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);
      }

      function draw() {
        if (!gl || !prog) return;
        gl.viewport(0, 0, canvas.width, canvas.height);
        gl.clearColor(0.12, 0.16, 0.23, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        gl.enable(gl.DEPTH_TEST);

        gl.useProgram(prog);

        const aPos = gl.getAttribLocation(prog, 'aPos');
        gl.bindBuffer(gl.ARRAY_BUFFER, posBuf);
        gl.enableVertexAttribArray(aPos);
        gl.vertexAttribPointer(aPos, 3, gl.FLOAT, false, 0, 0);

        const aNorm = gl.getAttribLocation(prog, 'aNormal');
        gl.bindBuffer(gl.ARRAY_BUFFER, normBuf);
        gl.enableVertexAttribArray(aNorm);
        gl.vertexAttribPointer(aNorm, 3, gl.FLOAT, false, 0, 0);

        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, idxBuf);

        // Matrices MVP simplifiées
        const aspect = canvas.width / canvas.height;
        const fov = 45 * Math.PI / 180;
        const f = 1.0 / Math.tan(fov / 2);
        const near = 0.1, far = 100.0;
        const proj = [
          f / aspect, 0, 0, 0,
          0, f, 0, 0,
          0, 0, (far + near) / (near - far), -1,
          0, 0, (2 * far * near) / (near - far), 0
        ];

        // Rotation & translation modèle
        const cx = Math.cos(rotX), sx = Math.sin(rotX);
        const cy = Math.cos(rotY), sy = Math.sin(rotY);
        const model = [
          cy, sx * sy, -cx * sy, 0,
          0, cx, sx, 0,
          sy, -sx * cy, cx * cy, 0,
          0, 0, -zoom, 1
        ];

        // Multiplication simple Proj * Model
        const mvp = new Float32Array(16);
        for (let r = 0; r < 4; r++) {
          for (let c = 0; c < 4; c++) {
            let sum = 0;
            for (let k = 0; k < 4; k++) sum += proj[r + k * 4] * model[k + c * 4];
            mvp[r + c * 4] = sum;
          }
        }

        gl.uniformMatrix4fv(gl.getUniformLocation(prog, 'uMVP'), false, mvp);
        gl.uniformMatrix4fv(gl.getUniformLocation(prog, 'uModel'), false, new Float32Array(model));

        gl.drawElements(gl.TRIANGLES, indexCount, gl.UNSIGNED_SHORT, 0);
      }

      draw();

      let dragging = false;
      let lastX = 0, lastY = 0;
      canvas.addEventListener('mousedown', (e) => {
        dragging = true;
        lastX = e.clientX;
        lastY = e.clientY;
      });
      window.addEventListener('mousemove', (e) => {
        if (!dragging) return;
        const dx = e.clientX - lastX;
        const dy = e.clientY - lastY;
        lastX = e.clientX;
        lastY = e.clientY;
        rotY += dx * 0.01;
        rotX += dy * 0.01;
        draw();
      });
      window.addEventListener('mouseup', () => { dragging = false; });
      canvas.addEventListener('wheel', (e) => {
        e.preventDefault();
        zoom = Math.max(1.2, Math.min(8.0, zoom + e.deltaY * 0.003));
        draw();
      }, { passive: false });

      c.apply = (patch) => {
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  register('html', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-html-wrap';

      if (p.label) {
        const lab = document.createElement('div');
        lab.className = 'mg-label';
        lab.innerHTML = '<span>' + esc(p.label) + '</span>';
        wrap.appendChild(lab);
      }

      const container = document.createElement('div');
      container.className = 'mg-html-container';
      wrap.appendChild(container);
      c.el.appendChild(wrap);

      let currentValue = p.value || '';
      let cleanups = [];

      // 1. Évaluation et injection propre du HTML avec exécution scopée des <script>
      const setContent = (rawHtml) => {
        // Exécution des nettoyages précédents
        cleanups.forEach((fn) => { try { fn(); } catch (_) {} });
        cleanups = [];

        container.innerHTML = rawHtml;

        // Extraction et exécution scopée des balises <script>
        const scripts = container.querySelectorAll('script');
        scripts.forEach((oldScript) => {
          const newScript = document.createElement('script');
          Array.from(oldScript.attributes).forEach((attr) => {
            newScript.setAttribute(attr.name, attr.value);
          });
          if (oldScript.src) {
            newScript.src = oldScript.src;
          } else {
            // Scope sécurisé injectant 'element' et 'grio'
            newScript.textContent = `(function(element, grio) {\n${oldScript.textContent}\n})(document.querySelector('[data-id="${c.id}"] .mg-html-container'), window.grio);`;
          }
          oldScript.parentNode.replaceChild(newScript, oldScript);
        });
      };

      // 2. Délégation d'événements robuste (Event Delegation)
      // Capture les interactions même si le HTML interne est reconstruit dynamiquement
      container.addEventListener('click', (e) => {
        const target = e.target.closest('[data-grio-action], [data-grio-click], button, a');
        if (!target || !container.contains(target)) return;

        const action = target.dataset.grioAction || target.dataset.grioClick || 'click';
        let payload = target.dataset.grioPayload;
        if (payload) {
          try { payload = JSON.parse(payload); } catch (_) {}
        } else {
          payload = target.value || target.getAttribute('href') || null;
        }

        emit(c, action, payload);
      });

      container.addEventListener('change', (e) => {
        const target = e.target.closest('input, select, textarea, [data-grio-change]');
        if (!target || !container.contains(target)) return;

        let val;
        if (target.type === 'checkbox') {
          val = target.checked;
        } else {
          val = target.value;
        }
        currentValue = val;
        emit(c, 'change', val);
      });

      container.addEventListener('input', (e) => {
        const target = e.target.closest('[data-grio-input]');
        if (!target || !container.contains(target)) return;
        emit(c, 'change', target.value);
      });

      setContent(currentValue);

      c.getValue = () => currentValue;
      c.apply = (patch) => {
        if (patch.value != null) {
          currentValue = String(patch.value);
          setContent(currentValue);
        }
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

  /* ---------- boot ---------- */

  function initTheme() {
    const toggleBtn = document.getElementById('mg-theme-toggle');
    const saved = localStorage.getItem('mg-theme');
    if (saved === 'dark' || saved === 'light') {
      document.documentElement.setAttribute('data-theme', saved);
    }

    if (toggleBtn) {
      const updateIcon = () => {
        const isDark = document.documentElement.getAttribute('data-theme') === 'dark'
          || (!document.documentElement.getAttribute('data-theme') && window.matchMedia('(prefers-color-scheme: dark)').matches);
        toggleBtn.textContent = isDark ? '☀️' : '🌙';
      };
      updateIcon();

      toggleBtn.addEventListener('click', () => {
        const current = document.documentElement.getAttribute('data-theme');
        let next;
        if (current === 'dark') {
          next = 'light';
        } else if (current === 'light') {
          next = 'dark';
        } else {
          next = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'light' : 'dark';
        }
        document.documentElement.setAttribute('data-theme', next);
        localStorage.setItem('mg-theme', next);
        updateIcon();
      });
    }
  }

  // ---------------------------------------------------------------------------
  // i18n Internationalization Engine
  // ---------------------------------------------------------------------------
  const I18N = {
    en: {
      use_api: "Use via API",
      api_docs: "API Docs",
      schema: "Schema",
      settings: "Settings",
      settings_title: "Application Settings",
      language: "Language",
      theme: "Theme Customization",
      api_title: "API Documentation & Client Code",
      api_intro: "Interact with this grio AI application programmatically via Python, JavaScript, cURL or Model Context Protocol (MCP).",
      copy_code: "Copy Snippet",
      copied: "Copied!",
      no_images: "No images yet",
      drop_images: "Drop images here or click to browse",
      add_image: "+ Add image",
      add_row: "+ Add Row",
      empty_chat: "Start the conversation...",
      model_weights_loaded: "Model weights loaded successfully",
      file_drop: "Click or drop files here to upload",
      file_browse: "Browse...",
      file_empty: "No files uploaded",
      file_remove: "remove",
      file_type_bad: "type not allowed",
      file_too_big: "file too large",
      json_valid: "Valid JSON",
      json_invalid: "Invalid JSON",
      download_label: "Download",
      num_step_down: "decrease",
      num_step_up: "increase",
      // ImageEditor & Canvas tools
      ie_open: "Open",
      ie_open_title: "Upload an image",
      ie_brush: "Brush",
      ie_eraser: "Eraser",
      ie_rect: "Rectangle",
      ie_line: "Line",
      ie_arrow: "Arrow",
      ie_crop: "Crop",
      ie_pan: "Move",
      ie_color: "Color",
      ie_size: "Thickness",
      ie_rotate: "Rotate",
      ie_filters: "Filters",
      ie_f_gray: "Grayscale (B&W)",
      ie_f_invert: "Invert",
      ie_f_bright: "Brighten",
      ie_f_dark: "Darken",
      ie_f_blur: "Gaussian Blur",
      ie_undo: "Undo",
      ie_redo: "Redo",
      ie_reset: "Reset",
      ie_layers: "Layers",
      ie_layer: "Layer",
      ie_visibility: "Visibility",
      media_choose_audio: "Choose Audio",
      media_choose_video: "Choose Video",
      media_record: "Record",
      media_stop: "Stop",
      media_camera: "Camera",
      media_stop_camera: "Stop Camera",
      rec_rec: "REC",
      rec_stop: "STOP",
      m3d_hint: "🖱️ Drag to rotate · Wheel to zoom",
      // Showcase tabs & elements
      tab_controls: "🎛️ Forms & Controls",
      tab_media: "🖼️ Media & Vision",
      tab_data: "📊 Data, Files & Code",
      tab_chat: "🤖 Chatbot & Observability",
      tab_system: "⚙️ System, Gauges & Docs",
      run_label: "Test Submission (Run)",
    },
    fr: {
      use_api: "Utiliser via API",
      api_docs: "Doc API",
      schema: "Schéma",
      settings: "Paramètres",
      settings_title: "Paramètres de l'Application",
      language: "Langue",
      theme: "Personnalisation du Thème",
      api_title: "Documentation API & Code Client",
      api_intro: "Interagissez avec cette application grio par programmation via Python, JavaScript, cURL ou Model Context Protocol (MCP).",
      copy_code: "Copier l'extrait",
      copied: "Copié !",
      no_images: "Aucune image pour le moment",
      drop_images: "Glissez vos images ici ou cliquez pour parcourir",
      add_image: "+ Ajouter une image",
      add_row: "+ Ajouter une ligne",
      empty_chat: "Commencez la conversation...",
      model_weights_loaded: "Poids du modèle chargés avec succès",
      file_drop: "Cliquez ou glissez des fichiers ici pour téléverser",
      file_browse: "Parcourir vos fichiers...",
      file_drop_sub: "Glissez-déposez des documents ou parcourez",
      file_remove: "retirer",
      file_type_bad: "type non autorisé",
      file_too_big: "fichier trop volumineux",
      json_valid: "JSON valide",
      json_invalid: "JSON invalide",
      download_label: "Télécharger",
      num_step_down: "diminuer",
      num_step_up: "augmenter",
      // ImageEditor & Canvas tools
      ie_open: "Ouvrir",
      ie_open_title: "Charger une image",
      ie_brush: "Pinceau",
      ie_eraser: "Gomme",
      ie_rect: "Rectangle",
      ie_line: "Ligne",
      ie_arrow: "Flèche",
      ie_crop: "Rogner",
      ie_pan: "Déplacer",
      ie_color: "Couleur",
      ie_size: "Épaisseur",
      ie_rotate: "Rotation",
      ie_filters: "Filtres",
      ie_f_gray: "Gris (N&B)",
      ie_f_invert: "Négatif",
      ie_f_bright: "Éclaircir",
      ie_f_dark: "Assombrir",
      ie_f_blur: "Flou gaussien",
      ie_undo: "Annuler",
      ie_redo: "Rétablir",
      ie_reset: "Reset",
      ie_layers: "Calques",
      ie_layer: "Calque",
      ie_visibility: "Visibilité",
      media_choose_audio: "Choisir un audio",
      media_choose_video: "Choisir une vidéo",
      media_record: "Enregistrer",
      media_stop: "Arrêter",
      media_camera: "Caméra",
      media_stop_camera: "Stop caméra",
      rec_rec: "REC",
      rec_stop: "STOP",
      m3d_hint: "🖱️ Glisser pour tourner · Molette pour zoomer",
      // Showcase tabs & elements
      tab_controls: "🎛️ Formulaires & Contrôles",
      tab_media: "🖼️ Médias & Vision",
      tab_data: "📊 Données, Fichiers & Code",
      tab_chat: "🤖 Chatbot & Observabilité",
      tab_system: "⚙️ Système, Jauges & Doc",
      run_label: "Tester la soumission (Run)",
    },
    es: {
      use_api: "Usar vía API",
      api_docs: "Doc API",
      schema: "Esquema",
      settings: "Ajustes",
      settings_title: "Ajustes de la Aplicación",
      language: "Idioma",
      theme: "Personalización del Tema",
      api_title: "Documentación API y Código Cliente",
      api_intro: "Interactúe con esta aplicación grio programáticamente mediante Python, JavaScript, cURL o MCP.",
      copy_code: "Copiar fragmento",
      copied: "¡Copiado!",
      no_images: "No hay imágenes aún",
      drop_images: "Arrastre imágenes aquí o haga clic para explorar",
      add_image: "+ Añadir imagen",
      add_row: "+ Añadir fila",
      empty_chat: "Comience la conversación...",
      model_weights_loaded: "Pesos del modelo cargados con éxito",
      file_drop: "Haga clic o arrastre archivos aquí",
      file_browse: "Examinar archivos...",
      file_drop_sub: "Arrastre documentos o examine",
      file_remove: "quitar",
      file_type_bad: "tipo no permitido",
      file_too_big: "archivo demasiado grande",
      json_valid: "JSON válido",
      json_invalid: "JSON no válido",
      download_label: "Descargar",
      num_step_down: "disminuir",
      num_step_up: "aumentar",
      // ImageEditor & Canvas tools
      ie_open: "Abrir",
      ie_open_title: "Cargar una imagen",
      ie_brush: "Pincel",
      ie_eraser: "Borrador",
      ie_rect: "Rectángulo",
      ie_line: "Línea",
      ie_arrow: "Flecha",
      ie_crop: "Recortar",
      ie_pan: "Mover",
      ie_color: "Color",
      ie_size: "Grosor",
      ie_rotate: "Rotar",
      ie_filters: "Filtros",
      ie_f_gray: "Gris (B/N)",
      ie_f_invert: "Invertir",
      ie_f_bright: "Aclarar",
      ie_f_dark: "Oscurecer",
      ie_f_blur: "Desenfoque",
      ie_undo: "Deshacer",
      ie_redo: "Rehacer",
      ie_reset: "Restablecer",
      ie_layers: "Capas",
      ie_layer: "Capa",
      ie_visibility: "Visibilidad",
      media_choose_audio: "Elegir audio",
      media_choose_video: "Elegir vídeo",
      media_record: "Grabar",
      media_stop: "Parar",
      media_camera: "Cámara",
      media_stop_camera: "Detener cámara",
      rec_rec: "REC",
      rec_stop: "STOP",
      m3d_hint: "🖱️ Arrastrar para rotar · Rueda para zoom",
      // Showcase tabs & elements
      tab_controls: "🎛️ Formularios y Controles",
      tab_media: "🖼️ Medios y Visión",
      tab_data: "📊 Datos, Archivos y Código",
      tab_chat: "🤖 Chatbot y Observabilidad",
      tab_system: "⚙️ Sistema, Indicadores y Docs",
      run_label: "Probar Envío (Run)",
    },
    de: {
      use_api: "Über API nutzen",
      api_docs: "API-Doku",
      schema: "Schema",
      settings: "Einstellungen",
      settings_title: "Anwendungseinstellungen",
      language: "Sprache",
      theme: "Design-Anpassung",
      api_title: "API-Dokumentation & Client-Code",
      api_intro: "Interagieren Sie programmgesteuert mit dieser grio-App über Python, JavaScript, cURL oder MCP.",
      copy_code: "Code kopieren",
      copied: "Kopiert!",
      no_images: "Noch keine Bilder",
      drop_images: "Bilder hierher ziehen oder klicken",
      add_image: "+ Bild hinzufügen",
      add_row: "+ Zeile hinzufügen",
      empty_chat: "Beginnen Sie das Gespräch...",
      model_weights_loaded: "Modellgewichte erfolgreich geladen",
      file_drop: "Klicken oder Dateien hierher ziehen",
      file_browse: "Dateien durchsuchen...",
      file_drop_sub: "Dokumente ablegen oder durchsuchen",
      file_remove: "entfernen",
      file_type_bad: "Typ nicht erlaubt",
      file_too_big: "Datei zu groß",
      json_valid: "gültiges JSON",
      json_invalid: "ungültiges JSON",
      download_label: "Herunterladen",
      num_step_down: "verringern",
      num_step_up: "erhöhen",
      // ImageEditor & Canvas tools
      ie_open: "Öffnen",
      ie_open_title: "Bild laden",
      ie_brush: "Pinsel",
      ie_eraser: "Radierer",
      ie_rect: "Rechteck",
      ie_line: "Linie",
      ie_arrow: "Pfeil",
      ie_crop: "Zuschneiden",
      ie_pan: "Verschieben",
      ie_color: "Farbe",
      ie_size: "Stärke",
      ie_rotate: "Drehen",
      ie_filters: "Filter",
      ie_f_gray: "Graustufen (S/W)",
      ie_f_invert: "Invertieren",
      ie_f_bright: "Aufhellen",
      ie_f_dark: "Abdunkeln",
      ie_f_blur: "Weichzeichnen",
      ie_undo: "Rückgängig",
      ie_redo: "Wiederholen",
      ie_reset: "Zurücksetzen",
      ie_layers: "Ebenen",
      ie_layer: "Ebene",
      ie_visibility: "Sichtbarkeit",
      media_choose_audio: "Audio wählen",
      media_choose_video: "Video wählen",
      media_record: "Aufnehmen",
      media_stop: "Stopp",
      media_camera: "Kamera",
      media_stop_camera: "Kamera stoppen",
      rec_rec: "AUFNAHME",
      rec_stop: "STOPP",
      m3d_hint: "🖱️ Ziehen zum Drehen · Mausrad für Zoom",
      // Showcase tabs & elements
      tab_controls: "🎛️ Formulare & Steuerelemente",
      tab_media: "🖼️ Medien & Bilderkennung",
      tab_data: "📊 Daten, Dateien & Code",
      tab_chat: "🤖 Chatbot & Beobachtbarkeit",
      tab_system: "⚙️ System, Anzeigen & Dokumentation",
      run_label: "Absenden testen (Run)",
    }
  };

  let currentLang = localStorage.getItem('mg-lang') || 'en';

  function t(key, fallback) {
    const dict = I18N[currentLang] || I18N.en;
    return dict[key] || (I18N.en && I18N.en[key]) || fallback || key;
  }

  const i18nHooks = [];
  function onI18n(fn) { i18nHooks.push(fn); }

  function applyTranslations() {
    document.querySelectorAll('[data-i18n]').forEach((el) => {
      const key = el.dataset.i18n;
      const text = t(key);
      if (text) el.textContent = text;
    });
    i18nHooks.forEach((fn) => { try { fn(); } catch { /* garde-fou */ } });
  }

  function setLanguage(lang) {
    if (!I18N[lang]) lang = 'en';
    currentLang = lang;
    localStorage.setItem('mg-lang', lang);
    document.documentElement.setAttribute('lang', lang);
    applyTranslations();
    document.querySelectorAll('.mg-lang-btn').forEach((b) => {
      b.classList.toggle('active', b.dataset.setLang === lang);
    });
  }

  // ---------------------------------------------------------------------------
  // API Snippets Generator (Python, JS, cURL, MCP)
  // ---------------------------------------------------------------------------
  function generateApiSnippets() {
    const host = window.location.origin;
    const predictUrl = host + '/api/predict';

    // Collect current input names and sample values
    const inputs = {};
    Object.keys(byId).forEach((id) => {
      const comp = byId[id];
      if (comp && comp.role === 'input' && comp.getValue) {
        let val = comp.getValue();
        if (typeof val === 'string' && val.length > 50) val = val.substring(0, 47) + '...';
        inputs[id] = val;
      }
    });

    const inputsJson = JSON.stringify(inputs, null, 2);

    return {
      python: `import requests\n\n# 1. API Endpoint URL\nurl = "${predictUrl}"\n\n# 2. Request Payload (Inputs)\npayload = {\n    "inputs": ${inputsJson.replace(/\n/g, '\n    ')}\n}\n\n# 3. Perform Prediction Call\nresponse = requests.post(url, json=payload)\nresult = response.json()\n\nprint("Status:", result.get("ok"))\nprint("Outputs:", result.get("outputs"))`,
      
      js: `// 1. Prediction Payload\nconst payload = {\n  inputs: ${inputsJson.replace(/\n/g, '\n  ')}\n};\n\n// 2. Call grio REST API\nasync function runPrediction() {\n  const response = await fetch("${predictUrl}", {\n    method: "POST",\n    headers: { "Content-Type": "application/json" },\n    body: JSON.stringify(payload)\n  });\n  const result = await response.json();\n  console.log("Prediction Result:", result);\n}\n\nrunPrediction();`,

      curl: `curl -X POST "${predictUrl}" \\\n  -H "Content-Type: application/json" \\\n  -d '{\n    "inputs": ${inputsJson.replace(/\n/g, '\n    ')}\n  }'`,

      mcp: `{\n  "name": "grio_ai_predict",\n  "description": "Execute prediction on grio multimodal AI pipeline",\n  "parameters": {\n    "type": "object",\n    "properties": {\n      "inputs": {\n        "type": "object",\n        "description": "Dynamic application parameters",\n        "properties": {\n${Object.keys(inputs).map(k => `          "${k}": { "type": "${typeof inputs[k]}" }`).join(',\n')}\n        }\n      }\n    },\n    "required": ["inputs"]\n  }\n}`
    };
  }

  function initApiModal() {
    const apiBtn = document.getElementById('mg-api-btn');
    const modal = document.getElementById('mg-api-modal');
    const closeBtn = document.getElementById('mg-api-close');
    const tabs = modal ? modal.querySelectorAll('.mg-api-tab') : [];
    const codeEl = document.getElementById('mg-api-code-content');
    const langTag = document.getElementById('mg-api-lang-tag');
    const copyBtn = document.getElementById('mg-copy-snippet-btn');
    const fullUrl = document.getElementById('mg-api-full-url');

    if (!modal) return;

    let activeTab = 'python';

    const renderSnippet = () => {
      const snippets = generateApiSnippets();
      if (codeEl) codeEl.textContent = snippets[activeTab] || snippets.python;
      if (langTag) langTag.textContent = activeTab;
      if (fullUrl) fullUrl.textContent = window.location.origin + '/api/predict';
    };

    const openModal = () => {
      modal.hidden = false;
      renderSnippet();
    };

    const closeModal = () => {
      modal.hidden = true;
    };

    if (apiBtn) apiBtn.addEventListener('click', openModal);
    if (closeBtn) closeBtn.addEventListener('click', closeModal);
    modal.addEventListener('click', (e) => {
      if (e.target === modal) closeModal();
    });

    tabs.forEach((tab) => {
      tab.addEventListener('click', () => {
        tabs.forEach((t) => t.classList.remove('active'));
        tab.classList.add('active');
        activeTab = tab.dataset.tab;
        renderSnippet();
      });
    });

    if (copyBtn) {
      copyBtn.addEventListener('click', () => {
        const text = codeEl ? codeEl.textContent : '';
        if (navigator.clipboard && text) {
          navigator.clipboard.writeText(text).then(() => {
            const orig = copyBtn.innerHTML;
            copyBtn.textContent = '✓ ' + t('copied', 'Copied!');
            setTimeout(() => { copyBtn.innerHTML = orig; }, 1800);
          });
        }
      });
    }
  }

  function initPreferences() {
    const prefsBtn = document.getElementById('mg-prefs-btn');
    const modal = document.getElementById('mg-prefs-modal');
    const closeBtn = document.getElementById('mg-prefs-close');

    if (!modal) return;

    const openModal = () => {
      modal.hidden = false;
      const currentTheme = document.documentElement.getAttribute('data-theme') || 'system';
      modal.querySelectorAll('.mg-theme-btn').forEach((b) => {
        b.classList.toggle('active', b.dataset.setTheme === currentTheme);
      });
      modal.querySelectorAll('.mg-lang-btn').forEach((b) => {
        b.classList.toggle('active', b.dataset.setLang === currentLang);
      });
    };

    const closeModal = () => {
      modal.hidden = true;
    };

    if (prefsBtn) prefsBtn.addEventListener('click', openModal);
    if (closeBtn) closeBtn.addEventListener('click', closeModal);
    modal.addEventListener('click', (e) => {
      if (e.target === modal) closeModal();
    });

    modal.querySelectorAll('.mg-lang-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        setLanguage(btn.dataset.setLang);
      });
    });

    modal.querySelectorAll('.mg-theme-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        const mode = btn.dataset.setTheme;
        if (mode === 'system') {
          document.documentElement.removeAttribute('data-theme');
          localStorage.removeItem('mg-theme');
        } else {
          document.documentElement.setAttribute('data-theme', mode);
          localStorage.setItem('mg-theme', mode);
        }
        modal.querySelectorAll('.mg-theme-btn').forEach((b) => b.classList.toggle('active', b === btn));
        const toggleBtn = document.getElementById('mg-theme-toggle');
        if (toggleBtn) {
          const isDark = document.documentElement.getAttribute('data-theme') === 'dark'
            || (!document.documentElement.getAttribute('data-theme') && window.matchMedia('(prefers-color-scheme: dark)').matches);
          toggleBtn.textContent = isDark ? '☀️' : '🌙';
        }
      });
    });
  }

  function init() {
    initTheme();
    setLanguage(currentLang);
    initPreferences();
    initApiModal();
    document.querySelectorAll('[data-kind]').forEach(mount);

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && e.target && e.target.classList.contains('mg-input') && runButton) {
        e.preventDefault();
        runButton.click();
      }
      if (e.key === 'Escape') {
        const prefs = document.getElementById('mg-prefs-modal');
        const apiModal = document.getElementById('mg-api-modal');
        if (prefs && !prefs.hidden) prefs.hidden = true;
        if (apiModal && !apiModal.hidden) apiModal.hidden = true;
      }
    });
    connect();
  }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init);
  else init();

  window.MG = { register, emit, byId, markdown, t, setLanguage, stream(id) { return { send(blob) { const c = byId[id]; if (c && blob) sendStream(c, blob); } }; } };
})();