  /* ---------- media components (image, audio, video, gallery, etc.) ---------- */

  function dataUrl(v) { return typeof v === 'string' && v.length ? v : ''; }

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

  register('gallery', {
    mount(c) {
      const p = c.props;
      const holder = document.createElement('div');
      holder.className = 'mg-gallery';
      
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

  register('pdf', {
    mount(c) {
      const p = c.props;
      const wrap = document.createElement('div');
      wrap.className = 'mg-pdf-wrap';

      if (p.label) {
        const lab = document.createElement('div');
        lab.className = 'mg-label';
        lab.innerHTML = '<span>' + esc(p.label) + '</span>';
        wrap.appendChild(lab);
      }

      let src = p.src || '';
      let page = p.page || 1;
      let totalPages = p.total_pages || null;
      let zoom = p.zoom || 1.0;
      let highlights = Array.isArray(p.highlights) ? p.highlights : [];

      const toolbar = document.createElement('div');
      toolbar.className = 'mg-pdf-toolbar';

      const prevBtn = document.createElement('button');
      prevBtn.type = 'button';
      prevBtn.className = 'mg-btn mg-btn-secondary mg-pdf-prev';
      prevBtn.textContent = '◀';
      prevBtn.title = 'Page précédente';

      const pageInfo = document.createElement('span');
      pageInfo.className = 'mg-pdf-page-info';

      const nextBtn = document.createElement('button');
      nextBtn.type = 'button';
      nextBtn.className = 'mg-btn mg-btn-secondary mg-pdf-next';
      nextBtn.textContent = '▶';
      nextBtn.title = 'Page suivante';

      const zoomOutBtn = document.createElement('button');
      zoomOutBtn.type = 'button';
      zoomOutBtn.className = 'mg-btn mg-btn-secondary';
      zoomOutBtn.textContent = '−';
      zoomOutBtn.title = 'Zoom arrière';

      const zoomBadge = document.createElement('span');
      zoomBadge.className = 'mg-pdf-zoom-badge';
      zoomBadge.textContent = Math.round(zoom * 100) + '%';

      const zoomInBtn = document.createElement('button');
      zoomInBtn.type = 'button';
      zoomInBtn.className = 'mg-btn mg-btn-secondary';
      zoomInBtn.textContent = '+';
      zoomInBtn.title = 'Zoom avant';

      const downloadBtn = document.createElement('a');
      downloadBtn.className = 'mg-btn mg-btn-secondary';
      downloadBtn.textContent = '⬇ Télécharger';
      downloadBtn.target = '_blank';

      toolbar.appendChild(prevBtn);
      toolbar.appendChild(pageInfo);
      toolbar.appendChild(nextBtn);
      toolbar.appendChild(zoomOutBtn);
      toolbar.appendChild(zoomBadge);
      toolbar.appendChild(zoomInBtn);
      toolbar.appendChild(downloadBtn);

      const viewport = document.createElement('div');
      viewport.className = 'mg-pdf-viewport';

      const container = document.createElement('div');
      container.className = 'mg-pdf-container';

      // Affichage de document interactif avec support embed / object et rendu de page
      const docView = document.createElement('div');
      docView.className = 'mg-pdf-doc-view';

      const overlay = document.createElement('div');
      overlay.className = 'mg-pdf-overlay';

      container.appendChild(docView);
      container.appendChild(overlay);
      viewport.appendChild(container);

      if (p.show_toolbar !== false) {
        wrap.appendChild(toolbar);
      }
      wrap.appendChild(viewport);
      c.el.appendChild(wrap);

      const renderSampleDoc = (title) => {
        return `
          <div class="mg-pdf-paper">
            <div class="mg-pdf-paper-header">
              <div class="mg-pdf-paper-logo">🏢 ENTERPRISE IT SUPPORT</div>
              <div class="mg-pdf-paper-meta">DOC-REF: IT-SEC-2026-09 · REV 3.2</div>
            </div>
            <h2 class="mg-pdf-paper-title">${esc(title || 'Guide de Configuration VPN & Certificats')}</h2>
            <div class="mg-pdf-paper-section">
              <h3>1. Diagnostic & Renouvellement de Certificat</h3>
              <p>Lorsque le message d'erreur <code>Error 403: Certificate expired</code> apparaît, le poste de travail ne peut plus établir de tunnel IPsec / WireGuard avec les passerelles du siège.</p>
              <div class="mg-pdf-callout">
                <strong>Action Requise :</strong> Rendez-vous sur le portail <em>https://cert.groupe.fr</em> et générez un nouveau profil de connexion PKCS#12 valide pour 365 jours.
              </div>
            </div>
            <div class="mg-pdf-paper-section">
              <h3>2. Paramètres Réseau & Passerelles</h3>
              <p>Passerelle Principale : <code>vpn-paris.groupe.fr:443</code><br>Passerelle Secondaire : <code>vpn-backup.groupe.fr:443</code></p>
            </div>
          </div>
        `;
      };

      const updateUI = () => {
        pageInfo.textContent = `Page ${page}` + (totalPages ? ` / ${totalPages}` : '');
        zoomBadge.textContent = Math.round(zoom * 100) + '%';
        container.style.transform = `scale(${zoom})`;
        container.style.transformOrigin = 'top center';

        if (src && (src.endsWith('.pdf') || src.includes('/pdf') || src.startsWith('data:application/pdf'))) {
          downloadBtn.href = src;
          downloadBtn.style.display = 'inline-flex';
          docView.innerHTML = `
            <object data="${src}#page=${page}&zoom=${Math.round(zoom * 100)}" type="application/pdf" class="mg-pdf-frame">
              ${renderSampleDoc(p.label)}
            </object>
          `;
        } else {
          downloadBtn.style.display = 'none';
          docView.innerHTML = renderSampleDoc(p.label);
        }

        // Rendu des surlignages (highlights) pour la page active
        overlay.innerHTML = '';
        const pageHighlights = highlights.filter((h) => !h.page || h.page === page);
        pageHighlights.forEach((hl) => {
          const box = document.createElement('div');
          box.className = 'mg-pdf-highlight';
          const x = (hl.x > 1 ? hl.x : hl.x * 100) + '%';
          const y = (hl.y > 1 ? hl.y : hl.y * 100) + '%';
          const w = (hl.width > 1 ? hl.width : hl.width * 100) + '%';
          const h = (hl.height > 1 ? hl.height : hl.height * 100) + '%';
          box.style.left = x;
          box.style.top = y;
          box.style.width = w;
          box.style.height = h;
          if (hl.color) {
            box.style.borderColor = hl.color;
            box.style.backgroundColor = hl.color + '25';
          }
          if (hl.label) {
            const badge = document.createElement('span');
            badge.className = 'mg-pdf-highlight-badge';
            badge.textContent = hl.label;
            if (hl.color) badge.style.backgroundColor = hl.color;
            box.appendChild(badge);
          }
          overlay.appendChild(box);
        });
      };

      prevBtn.addEventListener('click', () => {
        if (page > 1) {
          page--;
          updateUI();
          emit(c, 'change', { page, zoom, src });
        }
      });

      nextBtn.addEventListener('click', () => {
        if (!totalPages || page < totalPages) {
          page++;
          updateUI();
          emit(c, 'change', { page, zoom, src });
        }
      });

      zoomInBtn.addEventListener('click', () => {
        zoom = Math.min(zoom + 0.25, 3.0);
        updateUI();
      });

      zoomOutBtn.addEventListener('click', () => {
        zoom = Math.max(zoom - 0.25, 0.5);
        updateUI();
      });

      updateUI();

      c.getValue = () => ({ page, zoom, src, highlights });
      c.apply = (patch) => {
        if (patch.src != null) src = String(patch.src);
        if (patch.value != null && typeof patch.value === 'string') src = patch.value;
        if (patch.page != null) page = Number(patch.page);
        if (patch.total_pages != null) totalPages = Number(patch.total_pages);
        if (patch.zoom != null) zoom = Number(patch.zoom);
        if (Array.isArray(patch.highlights)) highlights = patch.highlights;
        if (patch.visible != null) c.el.hidden = !patch.visible;
        updateUI();
      };
    }
  });

