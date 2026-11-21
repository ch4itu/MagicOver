const t = require('./browser.cjs');

(async () => {
    const browser = await t.launch();
    try {
        const page = await t.page(browser);
        await t.smoke(page);
        await page.setViewportSize({width: 900, height: 540});
        await page.click('#eb-immersive-toggle');
        await page.waitForFunction(() => __view.eyeActive());
        const obscured = await page.evaluate(() =>
            ['eb-eye-use', 'eb-look-left', 'eb-look-right'].filter(id => {
                const node = document.getElementById(id);
                const bounds = node.getBoundingClientRect();
                const hit = document.elementFromPoint(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
                return !node.contains(hit);
            }));
        t.assert.deepEqual(obscured, [], 'first-person controls remain usable beside the minimap');
        await page.click('#eb-look-right');
        await t.shot(page, 'smoke-short-first-person-' + t.sha.slice(0,8));
        t.assert.deepEqual(page.errors, []);
        t.record('3D/FP/2D startup and short desktop first-person controls', 'PASS');
        console.log('PASS startup smoke and 900 × 540 first-person controls', t.sha);
    } finally {
        await browser.close();
    }
})().catch(error => { console.error(error); process.exitCode = 1; });
