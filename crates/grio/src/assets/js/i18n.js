  /* ---------- i18n, preferences, snippets & boot ---------- */

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

  function generateApiSnippets() {
    const host = window.location.origin;
    const predictUrl = host + '/api/predict';

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
    initMultiPage();
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

        document.querySelectorAll('.mg-drawer-container.mg-drawer-open').forEach((d) => {
          d.classList.remove('mg-drawer-open');
        });
        document.body.classList.remove('mg-drawer-active');

        const sidebar = document.getElementById('mg-sidebar');
        const sidebarBackdrop = document.getElementById('mg-sidebar-backdrop');
        if (sidebar && sidebar.classList.contains('mg-sidebar-open')) {
          sidebar.classList.remove('mg-sidebar-open');
          if (sidebarBackdrop) sidebarBackdrop.hidden = true;
        }
      }
    });
    connect();
  }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init);
  else init();

  window.MG = { register, emit, byId, markdown, t, setLanguage, stream(id) { return { send(blob) { const c = byId[id]; if (c && blob) sendStream(c, blob); } }; } };
