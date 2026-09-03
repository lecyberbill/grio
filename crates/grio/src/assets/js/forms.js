  /* ---------- forms & user input controls ---------- */

  function makeLabel(p, c) {
    return '<div class="mg-label"><span>' + esc(p.label || c.id) + '</span></div>';
  }

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
        choices.forEach((choice) => {
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

  register('richtext', {
    mount(c) {
      const p = c.props;
      const id = 'f_' + c.id;
      let textVal = p.value || '';
      let isPreview = false;

      const wrap = document.createElement('div');
      wrap.className = 'mg-richtext-wrap';

      if (p.label) {
        const lab = document.createElement('div');
        lab.className = 'mg-label';
        lab.innerHTML = '<span>' + esc(p.label) + '</span>';
        wrap.appendChild(lab);
      }

      const toolbar = document.createElement('div');
      toolbar.className = 'mg-richtext-toolbar';

      const toolDefs = [
        { label: 'B', tag: 'bold', title: 'Gras (Ctrl+B)', wrap: ['**', '**'] },
        { label: 'I', tag: 'italic', title: 'Italique (Ctrl+I)', wrap: ['*', '*'] },
        { label: 'H2', tag: 'h2', title: 'Titre 2', prefix: '## ' },
        { label: 'H3', tag: 'h3', title: 'Titre 3', prefix: '### ' },
        { label: '</>', tag: 'code', title: 'Code', wrap: ['`', '`'] },
        { label: '🔗', tag: 'link', title: 'Lien', wrap: ['[', '](https://)'] },
        { label: '•', tag: 'bullet', title: 'Liste à puces', prefix: '- ' },
        { label: '1.', tag: 'numbered', title: 'Liste numérotée', prefix: '1. ' },
        { label: '❝', tag: 'quote', title: 'Citation', prefix: '> ' }
      ];

      const textarea = document.createElement('textarea');
      textarea.id = id;
      textarea.className = 'mg-input mg-richtext-textarea';
      textarea.rows = p.lines || 6;
      textarea.value = textVal;
      if (p.placeholder) textarea.placeholder = p.placeholder;
      if (p.interactive === false) textarea.disabled = true;

      const previewDiv = document.createElement('div');
      previewDiv.className = 'mg-richtext-preview mg-markdown';
      previewDiv.style.display = 'none';
      if (p.lines) previewDiv.style.minHeight = (p.lines * 24) + 'px';

      const updatePreview = () => {
        previewDiv.innerHTML = markdown(textVal || '*Aucun contenu*');
      };

      const applyTool = (tool) => {
        if (p.interactive === false) return;
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const sel = textVal.substring(start, end);

        let replaced = '';
        let newCursor = start;

        if (tool.wrap) {
          const [before, after] = tool.wrap;
          replaced = before + (sel || 'texte') + after;
          textVal = textVal.substring(0, start) + replaced + textVal.substring(end);
          newCursor = start + before.length + (sel ? sel.length : 'texte'.length);
        } else if (tool.prefix) {
          const beforeCursor = textVal.substring(0, start);
          const lineStart = beforeCursor.lastIndexOf('\n') + 1;
          textVal = textVal.substring(0, lineStart) + tool.prefix + textVal.substring(lineStart);
          newCursor = start + tool.prefix.length;
        }

        textarea.value = textVal;
        textarea.focus();
        textarea.setSelectionRange(newCursor, newCursor);
        emit(c, 'change', textVal);
        updatePreview();
      };

      toolDefs.forEach((td) => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'mg-richtext-btn';
        btn.textContent = td.label;
        btn.title = td.title;
        btn.addEventListener('click', () => applyTool(td));
        toolbar.appendChild(btn);
      });

      if (p.show_preview !== false) {
        const sep = document.createElement('span');
        sep.className = 'mg-richtext-sep';
        toolbar.appendChild(sep);

        const previewBtn = document.createElement('button');
        previewBtn.type = 'button';
        previewBtn.className = 'mg-richtext-btn mg-richtext-toggle-preview';
        previewBtn.textContent = '👁 Aperçu';
        previewBtn.title = 'Basculer aperçu markdown';
        previewBtn.addEventListener('click', () => {
          isPreview = !isPreview;
          if (isPreview) {
            updatePreview();
            textarea.style.display = 'none';
            previewDiv.style.display = 'block';
            previewBtn.classList.add('active');
            previewBtn.textContent = '✏️ Éditer';
          } else {
            textarea.style.display = 'block';
            previewDiv.style.display = 'none';
            previewBtn.classList.remove('active');
            previewBtn.textContent = '👁 Aperçu';
            textarea.focus();
          }
        });
        toolbar.appendChild(previewBtn);
      }

      textarea.addEventListener('input', () => {
        textVal = textarea.value;
        emit(c, 'change', textVal);
      });

      textarea.addEventListener('keydown', (e) => {
        if (e.ctrlKey || e.metaKey) {
          if (e.key === 'b' || e.key === 'B') {
            e.preventDefault();
            applyTool(toolDefs[0]);
          } else if (e.key === 'i' || e.key === 'I') {
            e.preventDefault();
            applyTool(toolDefs[1]);
          } else if (e.key === 'k' || e.key === 'K') {
            e.preventDefault();
            applyTool(toolDefs[5]);
          }
        }
      });

      wrap.appendChild(toolbar);
      wrap.appendChild(textarea);
      wrap.appendChild(previewDiv);
      c.el.appendChild(wrap);

      c.getValue = () => textVal;
      c.apply = (patch) => {
        if (patch.value != null && String(patch.value) !== textVal) {
          textVal = String(patch.value);
          textarea.value = textVal;
          updatePreview();
        }
        if (patch.label != null && wrap.querySelector('.mg-label span')) {
          wrap.querySelector('.mg-label span').textContent = patch.label;
        }
        if (patch.visible != null) c.el.hidden = !patch.visible;
      };
    }
  });

