const dialog = document.querySelector('#preview-dialog');

if (dialog) {
  const tabs = [...document.querySelectorAll('[data-appearance]')];
  const frame = document.querySelector('#app-window');
  const stage = document.querySelector('#preview-stage');
  const canvas = document.querySelector('#preview-canvas');
  const image = document.querySelector('#preview-image');
  const error = document.querySelector('#preview-error');
  const loading = document.querySelector('#preview-loading');
  const zoomIn = document.querySelector('#zoom-in');
  const zoomOut = document.querySelector('#zoom-out');
  const fit = document.querySelector('#zoom-fit');
  const label = document.querySelector('#zoom-label');
  let appearance = 'light';
  let opener;
  // null means fit. Other scales use the capture's 1280px logical width.
  let scale = null;
  let fitScale = 1;
  let failed = false;
  let drag;

  function selectAppearance(tab) {
    appearance = tab.dataset.appearance;
    frame.dataset.theme = appearance;
    for (const option of tabs) {
      const selected = option === tab;
      option.setAttribute('aria-selected', String(selected));
      option.tabIndex = selected ? 0 : -1;
      document.getElementById(option.getAttribute('aria-controls')).hidden =
        !selected;
    }
  }

  for (const [index, tab] of tabs.entries()) {
    tab.addEventListener('click', () => selectAppearance(tab));
    tab.addEventListener('keydown', (event) => {
      let next;
      if (event.key === 'ArrowRight') next = (index + 1) % tabs.length;
      if (event.key === 'ArrowLeft')
        next = (index + tabs.length - 1) % tabs.length;
      if (event.key === 'Home') next = 0;
      if (event.key === 'End') next = tabs.length - 1;
      if (next === undefined) return;
      event.preventDefault();
      // Manual activation avoids switching to an image that may still be loading.
      for (const option of tabs) option.tabIndex = -1;
      tabs[next].tabIndex = 0;
      tabs[next].focus();
    });
  }

  function layout() {
    if (!dialog.open) return;
    fitScale = Math.min(
      1,
      (stage.clientWidth - 32) / 1280,
      (stage.clientHeight - 32) / 900,
    );
    if (scale !== null && scale <= fitScale) scale = null;
    const current = scale ?? fitScale;
    canvas.style.width = `${1280 * current}px`;
    canvas.style.height = `${900 * current}px`;
    label.textContent =
      scale === null ? 'Fit' : `${Math.round(current * 100)}%`;
    fit.setAttribute(
      'aria-label',
      `Fit screenshot to window. Current zoom: ${Math.round(current * 100)} percent`,
    );
    const unavailable = failed || !image.complete || image.naturalWidth === 0;
    zoomOut.disabled = unavailable || scale === null;
    zoomIn.disabled = unavailable || current >= 2;
    fit.disabled = unavailable;
    if (
      !unavailable &&
      ((zoomOut.disabled && document.activeElement === zoomOut) ||
        (zoomIn.disabled && document.activeElement === zoomIn))
    )
      fit.focus({ preventScroll: true });
    stage.dataset.zoomed = String(scale !== null);
  }

  function zoom(direction) {
    const current = scale ?? fitScale;
    const levels = [fitScale, 1, 1.5, 2].filter(
      (value, index, all) => value >= fitScale && all.indexOf(value) === index,
    );
    const next =
      direction > 0
        ? levels.find((value) => value > current + 0.01)
        : levels.findLast((value) => value < current - 0.01);
    if (next === undefined) return;
    const centerX =
      (stage.scrollLeft + stage.clientWidth / 2) / stage.scrollWidth;
    const centerY =
      (stage.scrollTop + stage.clientHeight / 2) / stage.scrollHeight;
    scale = next === fitScale ? null : next;
    layout();
    stage.scrollLeft = centerX * stage.scrollWidth - stage.clientWidth / 2;
    stage.scrollTop = centerY * stage.scrollHeight - stage.clientHeight / 2;
  }

  function fitImage() {
    scale = null;
    layout();
    stage.scrollTo(0, 0);
  }

  function loadImage() {
    failed = false;
    error.hidden = true;
    const source = document.querySelector(`#preview-${appearance} img`);
    image.alt = source.alt;
    image.src = source.src;
    loading.hidden = image.complete && image.naturalWidth > 0;
    canvas.hidden = !loading.hidden;
    layout();
  }

  for (const button of document.querySelectorAll('[data-open-preview]')) {
    button.addEventListener('click', () => {
      opener = button;
      scale = null;
      document.querySelector('#preview-title').textContent =
        `Local Chat · ${appearance === 'dark' ? 'Dark' : 'Light'}`;
      loadImage();
      dialog.showModal();
      document.body.classList.add('preview-open');
      layout();
      stage.scrollTo(0, 0);
    });
  }

  image.addEventListener('load', () => {
    loading.hidden = true;
    canvas.hidden = false;
    layout();
  });
  image.addEventListener('error', () => {
    failed = true;
    loading.hidden = true;
    canvas.hidden = true;
    error.hidden = false;
    layout();
  });
  document.querySelector('#preview-retry').addEventListener('click', loadImage);
  zoomIn.addEventListener('click', () => zoom(1));
  zoomOut.addEventListener('click', () => zoom(-1));
  fit.addEventListener('click', fitImage);
  document
    .querySelector('#preview-close')
    .addEventListener('click', () => dialog.close());
  let backdropStart = false;
  function outside(event) {
    const rect = dialog.getBoundingClientRect();
    return (
      event.clientX < rect.left ||
      event.clientX > rect.right ||
      event.clientY < rect.top ||
      event.clientY > rect.bottom
    );
  }
  dialog.addEventListener('pointerdown', (event) => {
    backdropStart = outside(event);
  });
  dialog.addEventListener('click', (event) => {
    if (backdropStart && outside(event)) dialog.close();
    backdropStart = false;
  });
  dialog.addEventListener('close', () => {
    document.body.classList.remove('preview-open');
    drag = undefined;
    stage.classList.remove('is-dragging');
    opener?.focus({ preventScroll: true });
  });
  dialog.addEventListener('keydown', (event) => {
    if (event.metaKey || event.ctrlKey || event.altKey || failed) return;
    if (event.key === '+' || event.key === '=') {
      event.preventDefault();
      zoom(1);
    }
    if (event.key === '-') {
      event.preventDefault();
      zoom(-1);
    }
    if (event.key === '0') {
      event.preventDefault();
      fitImage();
    }
  });
  stage.addEventListener('dblclick', () => {
    if (scale === null) zoom(1);
    else fitImage();
  });
  stage.addEventListener('pointerdown', (event) => {
    if (event.pointerType !== 'mouse' || event.button !== 0 || scale === null)
      return;
    drag = {
      x: event.clientX,
      y: event.clientY,
      left: stage.scrollLeft,
      top: stage.scrollTop,
    };
    stage.setPointerCapture(event.pointerId);
    stage.classList.add('is-dragging');
    event.preventDefault();
    stage.focus({ preventScroll: true });
  });
  stage.addEventListener('pointermove', (event) => {
    if (!drag) return;
    stage.scrollLeft = drag.left - event.clientX + drag.x;
    stage.scrollTop = drag.top - event.clientY + drag.y;
  });
  function endDrag() {
    drag = undefined;
    stage.classList.remove('is-dragging');
  }
  stage.addEventListener('pointerup', endDrag);
  stage.addEventListener('lostpointercapture', endDrag);
  window.addEventListener('resize', layout);
}
