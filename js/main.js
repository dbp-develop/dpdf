/**
 * dpdf — Website Client Scripts
 * Handles OS detection, dropdowns, interactive mockups, and FAQ accordion.
 */

document.addEventListener('DOMContentLoaded', () => {
  initPlatformDetection();
  initPlatformDropdown();
  initPreviewTabs();
  initFaqAccordion();
});

/**
 * Detect client OS and update the primary Hero CTA download button
 */
function initPlatformDetection() {
  const mainBtn = document.getElementById('main-download-btn');
  const osLabel = document.getElementById('btn-os-label');
  const osIcon = document.getElementById('btn-os-icon');

  if (!mainBtn || !osLabel || !osIcon) return;

  const ua = navigator.userAgent || '';
  const platform = navigator.platform || '';

  const isMac = /Macintosh|MacIntel|MacPPC|Mac68K/i.test(platform) || /Mac OS X/i.test(ua);
  const isLinux = /Linux/i.test(platform) && !/Android/i.test(ua);

  if (isMac) {
    mainBtn.href = 'https://github.com/dbp-develop/dpdf/releases/download/v1.0.1/dpdf-1.0.1-macos-universal.dmg';
    osLabel.textContent = 'macOS (Universal .dmg)';
    osIcon.innerHTML = `
      <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
        <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M15.97 6.37c.61-.75 1.04-1.8 0.92-2.85-.9.04-2 .6-2.63 1.34-.55.63-.99 1.68-.86 2.7 1 .08 1.96-.44 2.57-1.19z"/>
      </svg>
    `;
  } else if (isLinux) {
    mainBtn.href = 'https://github.com/dbp-develop/dpdf/releases/latest';
    osLabel.textContent = 'Linux Releases & Source';
    osIcon.innerHTML = `
      <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12.012 0c-3.13 0-5.672 2.542-5.672 5.672 0 1.077.307 2.083.84 2.941C6.273 9.429 5.5 10.871 5.5 12.5c0 1.258.463 2.41 1.229 3.3-.23.51-.36 1.079-.36 1.68 0 2.21 1.79 4 4 4 .26 0 .51-.03.76-.08.64.38 1.39.6 2.19.6s1.55-.22 2.19-.6c.25.05.5.08.76.08 2.21 0 4-1.79 4-4 0-.601-.13-1.17-.36-1.68.766-.89 1.229-2.042 1.229-3.3 0-1.629-.773-3.071-1.68-3.887.533-.858.84-1.864.84-2.941 0-3.13-2.542-5.672-5.672-5.672z"/>
      </svg>
    `;
  }
}

/**
 * Dropdown menu for platform selection in hero section
 */
function initPlatformDropdown() {
  const toggleBtn = document.getElementById('platform-menu-toggle');
  const dropdown = document.getElementById('platform-dropdown');

  if (!toggleBtn || !dropdown) return;

  toggleBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    const isOpen = dropdown.classList.toggle('show');
    toggleBtn.setAttribute('aria-expanded', String(isOpen));
  });

  document.addEventListener('click', (e) => {
    if (!dropdown.contains(e.target) && e.target !== toggleBtn) {
      dropdown.classList.remove('show');
      toggleBtn.setAttribute('aria-expanded', 'false');
    }
  });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      dropdown.classList.remove('show');
      toggleBtn.setAttribute('aria-expanded', 'false');
    }
  });
}

/**
 * Interactive tabs in the hero mockup window
 */
function initPreviewTabs() {
  const tabs = document.querySelectorAll('.preview-tab');
  const previewImg = document.getElementById('preview-img');

  if (!previewImg || tabs.length === 0) return;

  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const targetSrc = tab.getAttribute('data-src');
      if (!targetSrc || tab.classList.contains('active')) return;

      tabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');

      // Smooth fade transition
      previewImg.classList.add('fading');
      setTimeout(() => {
        previewImg.src = targetSrc;
        previewImg.onload = () => {
          previewImg.classList.remove('fading');
        };
        // Fallback in case image is already cached
        setTimeout(() => previewImg.classList.remove('fading'), 50);
      }, 150);
    });
  });
}

/**
 * Accessible FAQ Accordion
 */
function initFaqAccordion() {
  const items = document.querySelectorAll('.faq-item');

  items.forEach(item => {
    const question = item.querySelector('.faq-question');
    if (!question) return;

    question.addEventListener('click', () => {
      const isOpen = item.classList.contains('open');

      // Close all other items for clean single-expand behavior
      items.forEach(i => i.classList.remove('open'));

      if (!isOpen) {
        item.classList.add('open');
      }
    });
  });
}
