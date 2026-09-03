  /* ---------- canvas image editor (layers, brush, shapes, crop, history) ---------- */

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
      const safeToDataUrl = (cv) => {
        try {
          return cv.toDataURL('image/png');
        } catch (e) {
          console.warn('[grio imageeditor] Canvas tainted, exporting clean fallback:', e);
          return '';
        }
      };

      const safeGetImageData = (ctx, w, h) => {
        try {
          return ctx.getImageData(0, 0, w, h);
        } catch (e) {
          console.warn('[grio imageeditor] getImageData blocked by cross-origin, using blank buffer:', e);
          return ctx.createImageData(w, h);
        }
      };

      const visibleComp = () => {
        const cv = document.createElement('canvas');
        cv.width = BGW; cv.height = BGH;
        const g = cv.getContext('2d');
        if (bg) {
          try { g.drawImage(bg, 0, 0); } catch (_) {}
        }
        layers.forEach((l) => { if (l.visible) { g.globalAlpha = l.opacity; try { g.drawImage(l.cv, 0, 0); } catch (_) {} g.globalAlpha = 1; } });
        return cv;
      };
      const mask = () => {
        const cv = document.createElement('canvas');
        cv.width = BGW; cv.height = BGH;
        const g = cv.getContext('2d');
        g.fillStyle = '#000'; g.fillRect(0, 0, BGW, BGH);
        layers.forEach((l) => {
          try {
            g.drawImage(l.cv, 0, 0);
            g.globalCompositeOperation = 'source-in';
            g.fillStyle = '#fff'; g.fillRect(0, 0, BGW, BGH);
            g.globalCompositeOperation = 'source-over';
          } catch (_) {}
        });
        return cv;
      };
      const snapshot = () => ({
        bg: bg ? safeGetImageData(bg.getContext('2d'), BGW, BGH) : null,
        layers: layers.map((l) => safeGetImageData(l.ctx, BGW, BGH)),
        w: BGW,
        h: BGH
      });
      const history = []; let hIndex = -1;
      const pushHistory = () => { history.splice(hIndex + 1); history.push(snapshot()); if (history.length > 20) history.shift(); hIndex = history.length - 1; };
      const restore = (s) => {
        BGW = s.w; BGH = s.h;
        bg = document.createElement('canvas'); bg.width = BGW; bg.height = BGH;
        if (s.bg) {
          try { bg.getContext('2d').putImageData(s.bg, 0, 0); } catch (_) {}
        }
        layers.splice(0, layers.length);
        for (let i = 0; i < s.layers.length; i++) {
          layers.push(mkLayer(BGW, BGH));
          try { layers[i].ctx.putImageData(s.layers[i], 0, 0); } catch (_) {}
        }
      };
      const redo = () => { if (hIndex + 1 >= history.length) return; restore(history[++hIndex]); draw(); commit(); };
      const undo = () => { if (hIndex < 0) return; hIndex--; restore(hIndex >= 0 ? history[hIndex] : snapshotBlank()); draw(); commit(); };
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
        if (bg) { try { g.drawImage(bg, tx, ty, BGW * zoom, BGH * zoom); } catch (_) {} }
        layers.forEach((l) => {
          if (!l.visible) return;
          g.globalAlpha = l.opacity;
          try { g.drawImage(l.cv, tx, ty, BGW * zoom, BGH * zoom); } catch (_) {}
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
          image: safeToDataUrl(img),
          layers: layers.map((l) => safeToDataUrl(l.cv)),
          mask: safeToDataUrl(mk),
        });
      };
      c.getValue = () => ({
        image: safeToDataUrl(visibleComp()),
        layers: layers.map((l) => safeToDataUrl(l.cv)),
        mask: safeToDataUrl(mask())
      });
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
        try {
          const d = g.getImageData(0, 0, BGW, BGH);
          for (let i = 0; i < d.data.length; i += 4) fn(i, d.data);
          g.putImageData(d, 0, 0);
        } catch (e) {
          console.warn('[grio imageeditor] filter pixel manipulation blocked by CORS:', e);
        }
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
        old.forEach((l) => {
          const c2 = document.createElement('canvas'); c2.width = cw; c2.height = ch;
          c2.getContext('2d').drawImage(l.cv, -x, -y);
          layers.push({ cv: c2, ctx: c2.getContext('2d'), visible: l.visible, opacity: l.opacity });
        });
        bg = nb; BGW = cw; BGH = ch;
        fit();
      };
      const rotateBoth = () => {
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

        const applyImage = (imgObj) => {
          let w = imgObj.naturalWidth || imgObj.width || 700;
          let h = imgObj.naturalHeight || imgObj.height || 400;
          const s = Math.min(1, 4096 / w, 4096 / h, Math.sqrt(1.6e7 / (w * h)));
          w = Math.max(1, Math.round(w * s)); h = Math.max(1, Math.round(h * s));
          BGW = w; BGH = h;
          bgSrc = src;
          bg = document.createElement('canvas');
          bg.width = BGW; bg.height = BGH;
          bg.getContext('2d').drawImage(imgObj, 0, 0, BGW, BGH);
          layers.splice(0, layers.length);
          for (let i = 0; i < nLayers; i++) layers.push(mkLayer(BGW, BGH));
          fin();
        };

        const im = new Image();
        im.crossOrigin = 'anonymous';
        im.onload = () => applyImage(im);
        im.onerror = () => {
          fetch(src)
            .then((r) => r.blob())
            .then((blob) => {
              const reader = new FileReader();
              reader.onload = () => {
                const im2 = new Image();
                im2.onload = () => applyImage(im2);
                im2.src = reader.result;
              };
              reader.readAsDataURL(blob);
            })
            .catch(() => {
              console.warn('[grio imageeditor] Could not load background image:', src);
              toast('Impossible de charger cette image', 'error');
            });
        };
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
        bar.hidden = true;
      }
    }
  });
