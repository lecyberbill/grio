  /* ---------- specialized components (output, markdown, progress, chatbot, map, etc.) ---------- */

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

        const cx = Math.cos(rotX), sx = Math.sin(rotX);
        const cy = Math.cos(rotY), sy = Math.sin(rotY);
        const model = [
          cy, sx * sy, -cx * sy, 0,
          0, cx, sx, 0,
          sy, -sx * cy, cx * cy, 0,
          0, 0, -zoom, 1
        ];

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

      const setContent = (rawHtml) => {
        cleanups.forEach((fn) => { try { fn(); } catch (_) {} });
        cleanups = [];

        container.innerHTML = rawHtml;

        const scripts = container.querySelectorAll('script');
        scripts.forEach((oldScript) => {
          const newScript = document.createElement('script');
          Array.from(oldScript.attributes).forEach((attr) => {
            newScript.setAttribute(attr.name, attr.value);
          });
          if (oldScript.src) {
            newScript.src = oldScript.src;
          } else {
            newScript.textContent = `(function(element, grio) {\n${oldScript.textContent}\n})(document.querySelector('[data-id="${c.id}"] .mg-html-container'), window.grio);`;
          }
          oldScript.parentNode.replaceChild(newScript, oldScript);
        });
      };

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

  register('map', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-map-wrap';

      const lab = document.createElement('div');
      lab.className = 'mg-label';
      lab.innerHTML = '<span>' + esc(p.label || c.id) + '</span>';
      wrap.appendChild(lab);

      const frame = document.createElement('div');
      frame.className = 'mg-map-frame';
      const mapHeight = p.height || 420;
      frame.style.height = `${mapHeight}px`;

      const tilesContainer = document.createElement('div');
      tilesContainer.className = 'mg-map-tiles';
      frame.appendChild(tilesContainer);

      const svgLayer = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
      svgLayer.setAttribute('class', 'mg-map-svg-layer');
      frame.appendChild(svgLayer);

      const markersContainer = document.createElement('div');
      markersContainer.className = 'mg-map-markers';
      frame.appendChild(markersContainer);

      const controls = document.createElement('div');
      controls.className = 'mg-map-controls';
      const zoomInBtn = document.createElement('button');
      zoomInBtn.type = 'button'; zoomInBtn.className = 'mg-map-ctrl-btn'; zoomInBtn.textContent = '+'; zoomInBtn.title = 'Zoom in';
      const zoomOutBtn = document.createElement('button');
      zoomOutBtn.type = 'button'; zoomOutBtn.className = 'mg-map-ctrl-btn'; zoomOutBtn.textContent = '−'; zoomOutBtn.title = 'Zoom out';
      controls.appendChild(zoomInBtn);
      controls.appendChild(zoomOutBtn);
      frame.appendChild(controls);

      const attribution = document.createElement('div');
      attribution.className = 'mg-map-attribution';
      attribution.innerHTML = '© <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors';
      frame.appendChild(attribution);

      wrap.appendChild(frame);
      c.el.appendChild(wrap);

      let centerLat = Array.isArray(p.center) && p.center.length > 0 ? Number(p.center[0]) : 48.8566;
      let centerLon = Array.isArray(p.center) && p.center.length > 1 ? Number(p.center[1]) : 2.3522;
      let zoom = typeof p.zoom === 'number' ? Math.max(1, Math.min(19, Math.round(p.zoom))) : 12;
      let markers = Array.isArray(p.markers) ? p.markers : [];
      let circles = Array.isArray(p.circles) ? p.circles : [];
      let selectedCoord = { lat: centerLat, lon: centerLon };

      const lon2x = (lon, z) => ((lon + 180) / 360) * Math.pow(2, z) * 256;
      const lat2y = (lat, z) => {
        const rad = (lat * Math.PI) / 180;
        return ((1 - Math.log(Math.tan(rad) + 1 / Math.cos(rad)) / Math.PI) / 2) * Math.pow(2, z) * 256;
      };
      const x2lon = (x, z) => (x / (Math.pow(2, z) * 256)) * 360 - 180;
      const y2lat = (y, z) => {
        const n = Math.PI - (2 * Math.PI * y) / (Math.pow(2, z) * 256);
        return (180 / Math.PI) * Math.atan(0.5 * (Math.exp(n) - Math.exp(-n)));
      };

      const render = () => {
        const width = frame.clientWidth || 600;
        const height = frame.clientHeight || mapHeight;
        if (width <= 0 || height <= 0) return;

        const centerX = lon2x(centerLon, zoom);
        const centerY = lat2y(centerLat, zoom);

        const leftX = centerX - width / 2;
        const topY = centerY - height / 2;
        const rightX = centerX + width / 2;
        const bottomY = centerY + height / 2;

        const startTileX = Math.floor(leftX / 256);
        const endTileX = Math.floor(rightX / 256);
        const startTileY = Math.floor(topY / 256);
        const endTileY = Math.floor(bottomY / 256);

        const maxTiles = Math.pow(2, zoom);

        tilesContainer.innerHTML = '';
        for (let tx = startTileX; tx <= endTileX; tx++) {
          for (let ty = startTileY; ty <= endTileY; ty++) {
            if (ty < 0 || ty >= maxTiles) continue;
            const normalizedX = ((tx % maxTiles) + maxTiles) % maxTiles;
            const tileImg = document.createElement('img');
            tileImg.className = 'mg-map-tile';
            tileImg.src = `https://tile.openstreetmap.org/${zoom}/${normalizedX}/${ty}.png`;
            tileImg.loading = 'lazy';
            tileImg.style.left = `${tx * 256 - leftX}px`;
            tileImg.style.top = `${ty * 256 - topY}px`;
            tilesContainer.appendChild(tileImg);
          }
        }

        svgLayer.innerHTML = '';
        svgLayer.setAttribute('viewBox', `0 0 ${width} ${height}`);
        circles.forEach((circ) => {
          if (typeof circ.lat !== 'number' || typeof circ.lon !== 'number') return;
          const cx = lon2x(circ.lon, zoom) - leftX;
          const cy = lat2y(circ.lat, zoom) - topY;

          const metersPerPixel = (156543.03392 * Math.cos((circ.lat * Math.PI) / 180)) / Math.pow(2, zoom);
          const rPx = Math.max(2, (circ.radius || 1000) / metersPerPixel);

          const circleElem = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
          circleElem.setAttribute('cx', cx);
          circleElem.setAttribute('cy', cy);
          circleElem.setAttribute('r', rPx);
          const col = circ.color || '#6366f1';
          circleElem.setAttribute('stroke', col);
          circleElem.setAttribute('stroke-width', '2');
          circleElem.setAttribute('fill', col + '26');
          svgLayer.appendChild(circleElem);
        });

        markersContainer.innerHTML = '';
        markers.forEach((m, idx) => {
          if (typeof m.lat !== 'number' || typeof m.lon !== 'number') return;
          const mx = lon2x(m.lon, zoom) - leftX;
          const my = lat2y(m.lat, zoom) - topY;

          const pin = document.createElement('div');
          pin.className = 'mg-map-marker';
          pin.style.left = `${mx}px`;
          pin.style.top = `${my}px`;
          const col = m.color || '#6366f1';

          pin.innerHTML = `
            <svg class="mg-map-pin" viewBox="0 0 24 36" width="28" height="36">
              <path d="M12 0C5.37 0 0 5.37 0 12c0 9 12 24 12 24s12-15 12-24c0-6.63-5.37-12-12-12z" fill="${col}"/>
              <circle cx="12" cy="12" r="5" fill="#ffffff"/>
            </svg>
          `;

          if (m.label) {
            const popup = document.createElement('div');
            popup.className = 'mg-map-popup';
            popup.textContent = m.label;
            pin.appendChild(popup);
          }

          pin.addEventListener('click', (ev) => {
            ev.stopPropagation();
            selectedCoord = { lat: m.lat, lon: m.lon, marker_id: m.id || `m_${idx}`, label: m.label || '' };
            emit(c, 'change', selectedCoord);
            emit(c, 'click', selectedCoord);
          });

          markersContainer.appendChild(pin);
        });
      };

      if (p.interactive !== false) {
        let isDragging = false;
        let startX = 0, startY = 0;
        let startCenterLat = centerLat, startCenterLon = centerLon;

        frame.addEventListener('mousedown', (e) => {
          if (e.target.closest('.mg-map-controls') || e.target.closest('.mg-map-attribution')) return;
          isDragging = true;
          startX = e.clientX;
          startY = e.clientY;
          startCenterLat = centerLat;
          startCenterLon = centerLon;
          frame.style.cursor = 'grabbing';
        });

        window.addEventListener('mousemove', (e) => {
          if (!isDragging) return;
          const dx = e.clientX - startX;
          const dy = e.clientY - startY;
          const origCenterX = lon2x(startCenterLon, zoom);
          const origCenterY = lat2y(startCenterLat, zoom);
          const newCenterX = origCenterX - dx;
          const newCenterY = origCenterY - dy;
          centerLon = x2lon(newCenterX, zoom);
          centerLat = y2lat(newCenterY, zoom);
          render();
        });

        window.addEventListener('mouseup', () => {
          if (isDragging) {
            isDragging = false;
            frame.style.cursor = 'grab';
          }
        });

        frame.addEventListener('wheel', (e) => {
          e.preventDefault();
          const rect = frame.getBoundingClientRect();
          const mouseX = e.clientX - rect.left;
          const mouseY = e.clientY - rect.top;

          const oldCenterX = lon2x(centerLon, zoom);
          const oldCenterY = lat2y(centerLat, zoom);
          const mouseWorldX = oldCenterX - frame.clientWidth / 2 + mouseX;
          const mouseWorldY = oldCenterY - frame.clientHeight / 2 + mouseY;

          const mouseLon = x2lon(mouseWorldX, zoom);
          const mouseLat = y2lat(mouseWorldY, zoom);

          if (e.deltaY < 0 && zoom < 19) {
            zoom++;
          } else if (e.deltaY > 0 && zoom > 1) {
            zoom--;
          } else {
            return;
          }

          const newMouseWorldX = lon2x(mouseLon, zoom);
          const newMouseWorldY = lat2y(mouseLat, zoom);
          const newCenterX = newMouseWorldX + frame.clientWidth / 2 - mouseX;
          const newCenterY = newMouseWorldY + frame.clientHeight / 2 - mouseY;

          centerLon = x2lon(newCenterX, zoom);
          centerLat = y2lat(newCenterY, zoom);
          render();
        }, { passive: false });

        frame.addEventListener('click', (e) => {
          if (e.target.closest('.mg-map-controls') || e.target.closest('.mg-map-attribution') || e.target.closest('.mg-map-marker')) return;
          const rect = frame.getBoundingClientRect();
          const clickX = e.clientX - rect.left;
          const clickY = e.clientY - rect.top;
          const currentCenterX = lon2x(centerLon, zoom);
          const currentCenterY = lat2y(centerLat, zoom);
          const worldX = currentCenterX - frame.clientWidth / 2 + clickX;
          const worldY = currentCenterY - frame.clientHeight / 2 + clickY;
          const lat = y2lat(worldY, zoom);
          const lon = x2lon(worldX, zoom);
          selectedCoord = { lat: Math.round(lat * 100000) / 100000, lon: Math.round(lon * 100000) / 100000 };
          emit(c, 'change', selectedCoord);
          emit(c, 'click', selectedCoord);
        });

        zoomInBtn.addEventListener('click', () => { if (zoom < 19) { zoom++; render(); } });
        zoomOutBtn.addEventListener('click', () => { if (zoom > 1) { zoom--; render(); } });
      }

      requestAnimationFrame(render);

      c.apply = (patch) => {
        if (patch.label != null) lab.innerHTML = '<span>' + esc(String(patch.label)) + '</span>';
        if (patch.visible != null) c.el.hidden = !patch.visible;
        if (Array.isArray(patch.center)) {
          if (patch.center.length > 0) centerLat = Number(patch.center[0]);
          if (patch.center.length > 1) centerLon = Number(patch.center[1]);
        }
        if (typeof patch.zoom === 'number') zoom = Math.max(1, Math.min(19, Math.round(patch.zoom)));
        if (Array.isArray(patch.markers)) markers = patch.markers;
        if (Array.isArray(patch.circles)) circles = patch.circles;
        render();
      };
    }
  });

  register('nodegraph', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-nodegraph-wrap';

      if (p.label) {
        const lab = document.createElement('div');
        lab.className = 'mg-label';
        lab.innerHTML = '<span>' + esc(p.label) + '</span>';
        wrap.appendChild(lab);
      }

      const toolbar = document.createElement('div');
      toolbar.className = 'mg-nodegraph-toolbar';

      const addBtn = document.createElement('button');
      addBtn.type = 'button';
      addBtn.className = 'mg-btn mg-btn-secondary';
      addBtn.textContent = '+ Nœud LLM';

      const resetViewBtn = document.createElement('button');
      resetViewBtn.type = 'button';
      resetViewBtn.className = 'mg-btn mg-btn-secondary';
      resetViewBtn.textContent = '🎯 Recentrer';

      toolbar.appendChild(addBtn);
      toolbar.appendChild(resetViewBtn);
      if (p.interactive !== false) {
        wrap.appendChild(toolbar);
      }

      const canvasArea = document.createElement('div');
      canvasArea.className = 'mg-nodegraph-canvas';
      canvasArea.style.height = (p.height || 480) + 'px';

      const svgLayer = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
      svgLayer.setAttribute('class', 'mg-nodegraph-svg');

      const nodesLayer = document.createElement('div');
      nodesLayer.className = 'mg-nodegraph-nodes';

      canvasArea.appendChild(svgLayer);
      canvasArea.appendChild(nodesLayer);
      wrap.appendChild(canvasArea);
      c.el.appendChild(wrap);

      let nodes = Array.isArray(p.nodes) ? p.nodes.map((n) => Object.assign({}, n)) : [];
      let edges = Array.isArray(p.edges) ? p.edges.map((e) => Object.assign({}, e)) : [];
      let connectingFrom = null; // { nodeId, socketId, isOutput, x, y }

      const commit = () => {
        if (p.interactive === false) return;
        emit(c, 'change', { nodes, edges });
      };

      const getSocketPos = (nodeId, socketId, isOutput) => {
        const nodeEl = nodesLayer.querySelector(`[data-node-id="${nodeId}"]`);
        if (!nodeEl) return null;
        const selector = isOutput ? `[data-output-id="${socketId}"]` : `[data-input-id="${socketId}"]`;
        const sockEl = nodeEl.querySelector(selector);
        if (!sockEl) return null;
        const sockRect = sockEl.getBoundingClientRect();
        const canvasRect = canvasArea.getBoundingClientRect();
        return {
          x: sockRect.left + sockRect.width / 2 - canvasRect.left,
          y: sockRect.top + sockRect.height / 2 - canvasRect.top,
        };
      };

      const drawBezier = (x1, y1, x2, y2, color = 'var(--mg-accent)') => {
        const dx = Math.abs(x2 - x1) * 0.5 + 20;
        const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        path.setAttribute('d', `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`);
        path.setAttribute('fill', 'none');
        path.setAttribute('stroke', color);
        path.setAttribute('stroke-width', '3');
        path.setAttribute('stroke-linecap', 'round');
        return path;
      };

      const updateConnections = () => {
        svgLayer.innerHTML = '';
        edges.forEach((edge, eIdx) => {
          const p1 = getSocketPos(edge.from_node, edge.from_socket, true);
          const p2 = getSocketPos(edge.to_node, edge.to_socket, false);
          if (p1 && p2) {
            const curve = drawBezier(p1.x, p1.y, p2.x, p2.y, '#6366f1');
            curve.style.cursor = 'pointer';
            curve.addEventListener('click', () => {
              if (p.interactive !== false) {
                edges.splice(eIdx, 1);
                updateConnections();
                commit();
              }
            });
            svgLayer.appendChild(curve);
          }
        });

        if (connectingFrom) {
          const curve = drawBezier(connectingFrom.x, connectingFrom.y, connectingFrom.curX, connectingFrom.curY, '#e0e7ff');
          curve.setAttribute('stroke-dasharray', '5,5');
          svgLayer.appendChild(curve);
        }
      };

      const renderNodes = () => {
        nodesLayer.innerHTML = '';
        nodes.forEach((node) => {
          const nodeEl = document.createElement('div');
          nodeEl.className = `mg-nodegraph-node mg-node-${node.category || 'default'}`;
          nodeEl.dataset.nodeId = node.id;
          nodeEl.style.left = (node.x || 40) + 'px';
          nodeEl.style.top = (node.y || 40) + 'px';

          const header = document.createElement('div');
          header.className = 'mg-node-header';

          const titleSpan = document.createElement('span');
          titleSpan.className = 'mg-node-title';
          titleSpan.textContent = node.title || node.id;

          const statusBadge = document.createElement('span');
          statusBadge.className = `mg-node-status status-${node.status || 'idle'}`;
          statusBadge.textContent = node.status || 'idle';

          header.appendChild(titleSpan);
          header.appendChild(statusBadge);
          nodeEl.appendChild(header);

          const body = document.createElement('div');
          body.className = 'mg-node-body';

          const inputsCol = document.createElement('div');
          inputsCol.className = 'mg-node-inputs';
          (node.inputs || []).forEach((inp) => {
            const row = document.createElement('div');
            row.className = 'mg-node-socket-row mg-node-input-row';
            const dot = document.createElement('div');
            dot.className = 'mg-node-socket mg-socket-input';
            dot.dataset.inputId = inp.id;
            dot.title = `${inp.label} (${inp.data_type || 'any'})`;

            dot.addEventListener('mouseup', () => {
              if (connectingFrom && connectingFrom.isOutput && connectingFrom.nodeId !== node.id) {
                edges.push({
                  from_node: connectingFrom.nodeId,
                  from_socket: connectingFrom.socketId,
                  to_node: node.id,
                  to_socket: inp.id,
                });
                connectingFrom = null;
                updateConnections();
                commit();
              }
            });

            const lbl = document.createElement('span');
            lbl.textContent = inp.label || inp.id;
            row.appendChild(dot);
            row.appendChild(lbl);
            inputsCol.appendChild(row);
          });

          const outputsCol = document.createElement('div');
          outputsCol.className = 'mg-node-outputs';
          (node.outputs || []).forEach((out) => {
            const row = document.createElement('div');
            row.className = 'mg-node-socket-row mg-node-output-row';
            const lbl = document.createElement('span');
            lbl.textContent = out.label || out.id;
            const dot = document.createElement('div');
            dot.className = 'mg-node-socket mg-socket-output';
            dot.dataset.outputId = out.id;
            dot.title = `${out.label} (${out.data_type || 'any'})`;

            dot.addEventListener('mousedown', (e) => {
              e.stopPropagation();
              const pos = getSocketPos(node.id, out.id, true);
              if (pos) {
                connectingFrom = {
                  nodeId: node.id,
                  socketId: out.id,
                  isOutput: true,
                  x: pos.x,
                  y: pos.y,
                  curX: pos.x,
                  curY: pos.y,
                };
              }
            });

            row.appendChild(lbl);
            row.appendChild(dot);
            outputsCol.appendChild(row);
          });

          body.appendChild(inputsCol);
          body.appendChild(outputsCol);
          nodeEl.appendChild(body);

          // Drag du nœud
          let isDragging = false;
          let dragStartX = 0;
          let dragStartY = 0;
          let initialNodeX = node.x || 40;
          let initialNodeY = node.y || 40;

          header.addEventListener('mousedown', (e) => {
            if (p.interactive === false) return;
            isDragging = true;
            dragStartX = e.clientX;
            dragStartY = e.clientY;
            initialNodeX = node.x || 40;
            initialNodeY = node.y || 40;
            nodeEl.style.zIndex = '100';
          });

          window.addEventListener('mousemove', (e) => {
            if (isDragging) {
              const dx = e.clientX - dragStartX;
              const dy = e.clientY - dragStartY;
              node.x = Math.max(0, initialNodeX + dx);
              node.y = Math.max(0, initialNodeY + dy);
              nodeEl.style.left = node.x + 'px';
              nodeEl.style.top = node.y + 'px';
              updateConnections();
            } else if (connectingFrom) {
              const canvasRect = canvasArea.getBoundingClientRect();
              connectingFrom.curX = e.clientX - canvasRect.left;
              connectingFrom.curY = e.clientY - canvasRect.top;
              updateConnections();
            }
          });

          window.addEventListener('mouseup', () => {
            if (isDragging) {
              isDragging = false;
              nodeEl.style.zIndex = '10';
              commit();
            }
            if (connectingFrom) {
              connectingFrom = null;
              updateConnections();
            }
          });

          nodesLayer.appendChild(nodeEl);
        });

        setTimeout(updateConnections, 50);
      };

      addBtn.addEventListener('click', () => {
        const id = 'node_' + (nodes.length + 1);
        nodes.push({
          id,
          title: 'LLM Agent #' + (nodes.length + 1),
          category: 'llm',
          x: 100 + nodes.length * 30,
          y: 80 + nodes.length * 20,
          inputs: [{ id: 'prompt', label: 'Prompt', data_type: 'text' }],
          outputs: [{ id: 'response', label: 'Réponse', data_type: 'text' }],
          status: 'idle',
        });
        renderNodes();
        commit();
      });

      resetViewBtn.addEventListener('click', () => {
        nodes.forEach((n, i) => {
          n.x = 40 + (i % 3) * 240;
          n.y = 40 + Math.floor(i / 3) * 160;
        });
        renderNodes();
        commit();
      });

      renderNodes();

      c.getValue = () => ({ nodes, edges });
      c.apply = (patch) => {
        if (Array.isArray(patch.nodes)) nodes = patch.nodes;
        if (Array.isArray(patch.edges)) edges = patch.edges;
        if (patch.value && typeof patch.value === 'object') {
          if (Array.isArray(patch.value.nodes)) nodes = patch.value.nodes;
          if (Array.isArray(patch.value.edges)) edges = patch.value.edges;
        }
        if (patch.visible != null) c.el.hidden = !patch.visible;
        renderNodes();
      };
    }
  });

