  /* ---------- router & drawers ---------- */

  register('drawer', (c) => {
    const el = c.el;
    const backdrop = el.querySelector('.mg-drawer-backdrop');
    const closeBtn = el.querySelector('.mg-drawer-close');

    const setOpen = (open) => {
      if (open) {
        el.classList.add('mg-drawer-open');
        document.body.classList.add('mg-drawer-active');
      } else {
        el.classList.remove('mg-drawer-open');
        if (!document.querySelector('.mg-drawer-container.mg-drawer-open')) {
          document.body.classList.remove('mg-drawer-active');
        }
      }
    };

    if (closeBtn) {
      closeBtn.addEventListener('click', () => {
        setOpen(false);
        emit(c, 'change', { open: false });
      });
    }

    if (backdrop) {
      backdrop.addEventListener('click', () => {
        setOpen(false);
        emit(c, 'change', { open: false });
      });
    }

    c.getValue = () => el.classList.contains('mg-drawer-open');
    c.apply = (patch) => {
      if (patch.value !== undefined) setOpen(Boolean(patch.value));
      if (patch.open !== undefined) setOpen(Boolean(patch.open));
      if (patch.visible !== undefined) {
        if (!patch.visible) setOpen(false);
        el.hidden = !patch.visible;
      }
    };
  });

  function initMultiPage() {
    const navItems = document.querySelectorAll('.mg-nav-item');
    const pages = document.querySelectorAll('.mg-page-view');
    const sidebar = document.getElementById('mg-sidebar');
    const sidebarToggle = document.getElementById('mg-sidebar-toggle');
    const sidebarClose = document.getElementById('mg-sidebar-close');
    const sidebarBackdrop = document.getElementById('mg-sidebar-backdrop');

    if (!navItems.length || !pages.length) return;

    const setSidebarOpen = (open) => {
      if (!sidebar) return;
      sidebar.classList.toggle('mg-sidebar-open', open);
      if (sidebarBackdrop) sidebarBackdrop.hidden = !open;
    };

    if (sidebarToggle) sidebarToggle.addEventListener('click', () => setSidebarOpen(true));
    if (sidebarClose) sidebarClose.addEventListener('click', () => setSidebarOpen(false));
    if (sidebarBackdrop) sidebarBackdrop.addEventListener('click', () => setSidebarOpen(false));

    const navigateTo = (route, push = true) => {
      let matched = false;
      pages.forEach((p) => {
        const pRoute = p.dataset.route;
        const isMatch = pRoute === route || (route === '/' && pRoute === '') || (pRoute === '/' && route === '');
        p.classList.toggle('active', isMatch);
        if (isMatch) matched = true;
      });

      if (!matched && pages.length > 0) {
        pages[0].classList.add('active');
        route = pages[0].dataset.route;
      }

      navItems.forEach((item) => {
        const itemRoute = item.dataset.grioRoute;
        const isMatch = itemRoute === route || (route === '/' && itemRoute === '') || (itemRoute === '/' && route === '');
        item.classList.toggle('active', isMatch);
      });

      if (push && location.pathname !== route) {
        try {
          history.pushState({ route }, '', route);
        } catch (e) { /* fallback pour environnements stricts */ }
      }
      setSidebarOpen(false);
    };

    navItems.forEach((item) => {
      item.addEventListener('click', (e) => {
        e.preventDefault();
        const route = item.dataset.grioRoute;
        navigateTo(route, true);
      });
    });

    window.addEventListener('popstate', (e) => {
      const route = (e.state && e.state.route) ? e.state.route : location.pathname;
      navigateTo(route, false);
    });

    // Initial route sync
    navigateTo(location.pathname, false);
  }

  /* ---------- authentication & user state ---------- */
  function initAuth() {
    const authHeader = document.getElementById('mg-auth-header');
    if (!authHeader) return;

    fetch('/auth/user')
      .then((r) => r.json())
      .then((data) => {
        const loginBtn = document.getElementById('mg-login-btn');
        const userPill = document.getElementById('mg-user-pill');
        const userAvatar = document.getElementById('mg-user-avatar');
        const userName = document.getElementById('mg-user-name');
        const userRole = document.getElementById('mg-user-role');
        const logoutBtn = document.getElementById('mg-logout-btn');

        if (data.authenticated && data.user) {
          if (loginBtn) loginBtn.hidden = true;
          if (userPill) userPill.hidden = false;
          if (userName) userName.textContent = data.user.username || 'User';
          if (userAvatar) {
            if (data.user.avatar_url) userAvatar.src = data.user.avatar_url;
            else userAvatar.src = `https://api.dicebear.com/7.x/identicon/svg?seed=${data.user.username}`;
          }
          if (userRole && data.user.roles && data.user.roles.length) {
            userRole.textContent = data.user.roles[0];
          }

          // RBAC: Check required roles on nav items and pages
          document.querySelectorAll('[data-required-role]').forEach((el) => {
            const req = el.dataset.requiredRole;
            const hasRole = data.user.roles && data.user.roles.includes(req);
            if (!hasRole) {
              el.classList.add('mg-rbac-denied');
              if (el.tagName === 'A' || el.tagName === 'BUTTON') {
                el.style.display = 'none';
              }
            }
          });
        } else {
          if (loginBtn) loginBtn.hidden = false;
          if (userPill) userPill.hidden = true;
        }

        if (logoutBtn) {
          logoutBtn.addEventListener('click', () => {
            fetch('/auth/logout', { method: 'POST' }).then(() => {
              location.reload();
            });
          });
        }
      })
      .catch(() => {});
  }
