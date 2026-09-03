/* grio — moteur frontend (vanilla JS, zéro dépendance) */
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
          if (c) {
            if (u.p && u.p.visible !== undefined) {
              c.el.hidden = !u.p.visible;
            }
            if (c.apply) c.apply(u.p || {});
            flash(c.el);
          }
          // Notification des abonnés client (window.grio.on)
          if (clientSubscribers[u.id]) {
            clientSubscribers[u.id].forEach((cb) => {
              try { cb(u.p || {}); } catch (err) { console.error('[grio.on error]', err); }
            });
          }
        });
      } else if (m.t === 'slot') {
        const container = document.querySelector(`[data-id="${m.container}"]`) || byId[m.container]?.el;
        if (container) {
          if (m.mode === 'clear') {
            container.innerHTML = '';
          } else if (m.mode === 'replace') {
            container.innerHTML = m.html || '';
            container.querySelectorAll('[data-kind]').forEach(mount);
          } else if (m.mode === 'append') {
            const temp = document.createElement('div');
            temp.innerHTML = m.html || '';
            const children = Array.from(temp.children);
            children.forEach((child) => {
              container.appendChild(child);
              if (child.dataset && child.dataset.kind) mount(child);
              child.querySelectorAll('[data-kind]').forEach(mount);
            });
          }
        }
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
    if (kind === 'row' || kind === 'column' || kind === 'grid' || kind === 'panel' || kind === 'accordion' || kind === 'dynamic_container') {
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
    if (typeof impl === 'function') {
      const res = impl(c, props);
      if (res) {
        if (res.apply) c.apply = res.apply;
        if (res.getValue) c.getValue = res.getValue;
      }
    } else if (impl.mount) {
      impl.mount(c);
    }
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
