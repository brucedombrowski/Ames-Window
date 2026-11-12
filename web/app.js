// The page's own script: starts the app, measures the frame rate, and fills
// in the Report feedback link. Kept in a file, not inline, so the page's
// Content-Security-Policy can refuse inline script altogether.
import init from "./ames-window.js";
init().catch((error) => {
  // Most likely a browser too old for WebAssembly SIMD. Say so rather
  // than leave a blank page.
  const note = document.createElement("p");
  note.textContent = "This browser could not start the app. It needs a current version of Edge, Chrome, Firefox or Safari.";
  note.style.cssText = "color:#e8d9b5;font:16px system-ui,sans-serif;padding:2rem;margin:0";
  document.body.prepend(note);
  console.error(error);
});

// The canvas claims every key press, Tab included, which would trap
// keyboard users on it. Let Tab through so the link below can be reached.
window.addEventListener("keydown", (event) => {
  if (event.key === "Tab") event.stopPropagation();
}, true);

// The app draws one frame per animation frame and rasterises on this
// thread, so the spacing of animation frames is its real frame rate.
const WINDOW_MS = 3000;
const stamps = [];
const rate = document.getElementById("rate");
let shown = 0;
function measure() {
  const span = stamps[stamps.length - 1] - stamps[0];
  if (stamps.length < 2 || span <= 0) return null;
  let slowest = 0;
  for (let i = 1; i < stamps.length; i++) slowest = Math.max(slowest, stamps[i] - stamps[i - 1]);
  return { fps: (stamps.length - 1) * 1000 / span, slowest };
}
function frame(now) {
  stamps.push(now);
  while (now - stamps[0] > WINDOW_MS) stamps.shift();
  if (now - shown > 500) {
    shown = now;
    const m = measure();
    if (m) {
      rate.textContent = `${m.fps.toFixed(0)} fps`;
      document.documentElement.dataset.fps = m.fps.toFixed(1);
      document.documentElement.dataset.slowest = m.slowest.toFixed(0);
    }
  }
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);

// Report feedback. The app itself has no network code and the page makes
// no request of its own. This is a link: following it opens GitHub's
// new-issue form, and the measurements below travel to GitHub in the
// link's address at that moment. Nothing is sent unless the viewer
// follows the link, and nothing is published unless they submit the form.
const link = document.getElementById("report");
const blank = link.href;
function report() {
  const m = measure();
  const canvas = document.querySelector("canvas");
  const lines = [
    "**What you saw:**",
    "",
    "",
    "| Measured in the browser | |",
    "|---|---|",
    `| Frame rate | ${m ? m.fps.toFixed(1) + " fps" : "not measured"} |`,
    `| Slowest frame | ${m ? m.slowest.toFixed(0) + " ms" : "not measured"} |`,
    `| Rasterised size | ${canvas ? canvas.width + " x " + canvas.height : "no canvas"} |`,
    // What the app itself spent on each frame, which it leaves on its canvas.
    // Painting is the app computing pixels; presenting is handing them to
    // the browser. Together with the frame rate they say where time goes.
    `| App time per frame | ${canvas?.dataset.paintMs ? canvas.dataset.paintMs + " ms painting, " + canvas.dataset.presentMs + " ms presenting" : "not measured yet"} |`,
    `| Page size | ${innerWidth} x ${innerHeight} at ${devicePixelRatio}x |`,
    `| Processor threads | ${navigator.hardwareConcurrency ?? "unknown"} |`,
    `| Build | ${document.querySelector('meta[name="build"]').content} |`,
    `| Browser | ${navigator.userAgent} |`,
  ];
  const title = "Web feedback" + (m ? ` (${m.fps.toFixed(0)} fps)` : "");
  return `${blank}?title=${encodeURIComponent(title)}&body=${encodeURIComponent(lines.join("\n"))}`;
}
// Fill the link in at the moment it is used, so the numbers are current
// however it is opened: click, middle click, keyboard or context menu.
for (const type of ["pointerdown", "keydown", "contextmenu"]) {
  link.addEventListener(type, () => { link.href = report(); });
}
// Afterwards give the keyboard back to the canvas, so Space, X and R work
// again without a click.
link.addEventListener("click", () => {
  setTimeout(() => document.querySelector("canvas")?.focus(), 0);
});
