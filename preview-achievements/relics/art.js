// Original relic artwork, drawn from the relic's kind, rarity and id. Pure function, no external assets.
const RARITY = ["#8a8f98", "#3f9b6e", "#3d6fd1", "#c9962b"];
const byte = (hex, i) => parseInt(hex.slice(i * 2, i * 2 + 2), 16) || 0;

export function relicSvg({ id, kind, rarity }, size = 120) {
  if (!/^[0-9a-f]{64}$/.test(id)) throw new Error("bad relic id");
  const k = Number.isInteger(kind) ? kind : 0;
  const colour = RARITY[Math.min(Math.max(rarity | 0, 0), 3)];
  const dots = [];
  for (let i = 0; i < 12; i++) {
    const a = (Math.PI * 2 * i) / 12;
    const r = 3 + (byte(id, i) % 5);
    dots.push(`<circle cx="${(60 + 48 * Math.cos(a)).toFixed(1)}" cy="${(60 + 48 * Math.sin(a)).toFixed(1)}" r="${r}" fill="${colour}" opacity="0.55"/>`);
  }
  const hue = (byte(id, 20) * 360) / 256;
  const core = `hsl(${hue.toFixed(0)} 55% 62%)`;
  const shapes = {
    1: `<path d="M60 26c16 18 22 30 22 42a22 22 0 0 1-44 0c0-12 6-24 22-42z" fill="${core}"/>`,
    2: `<circle cx="60" cy="60" r="26" fill="none" stroke="${core}" stroke-width="6"/><path d="M60 34l8 26-8 26-8-26z" fill="${core}"/>`,
    3: `<path d="M34 78c0-26 8-40 26-40s26 14 26 40z" fill="${core}"/><circle cx="60" cy="84" r="6" fill="${core}"/>`,
    4: `<path d="M60 30l8 20 22 2-17 14 6 22-19-12-19 12 6-22-17-14 22-2z" fill="${core}"/>`,
  };
  const body = shapes[k] || `<rect x="38" y="38" width="44" height="44" rx="8" fill="${core}"/>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120" width="${size}" height="${size}" role="img" aria-label="relic"><rect width="120" height="120" rx="22" fill="#14161c"/><circle cx="60" cy="60" r="54" fill="none" stroke="${colour}" stroke-width="3"/>${dots.join("")}${body}</svg>`;
}
