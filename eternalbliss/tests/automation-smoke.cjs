const t = require('./browser.cjs');
const baseline = process.env.EB_AUTOMATION_BASELINE === '1';
(async () => {
    const browser = await t.launch();
    try {
        for (const [name, profile] of [
            ['desktop', {viewport: {width: 1280, height: 800}}],
            ['portrait', {viewport: {width: 390, height: 844}, hasTouch: true, isMobile: true}],
            ['landscape', {viewport: {width: 844, height: 390}, hasTouch: true, isMobile: true}],
        ]) {
            if (process.env.EB_AUTOMATION_PROFILE && process.env.EB_AUTOMATION_PROFILE !== name) continue;
            const page = await t.page(browser, profile, (1767225600 + 4 * 480 + 240) * 1000);
            page.setDefaultTimeout(20000);
            // The view toggle is user-facing. Use 2D for the Atlas, whose layout
            // is independent of the 3D renderer.
            await page.locator('#eb-view-toggle').click();
            const panel = page.locator('#eb-actions-collapse');
            if (await panel.getAttribute('aria-expanded') === 'false') await panel.click();
            if (await page.locator('#eb-farlands-atlas-open').isHidden()) {
                await page.locator('#eb-actions-more').click();
            }
            await page.locator('#eb-farlands-atlas-open').click();
            await page.selectOption('#eb-farlands-destination', 'alder');
            const button = page.locator('#eb-farlands-travel');
            t.assert.equal(await button.textContent(), baseline ? '⛵ Pilot boat' : '⛵ Pilot passage boat');
            t.assert.equal(await button.isEnabled(), true);
            if (!baseline) {
                t.assert.match(await page.locator('#eb-farlands-travel-note').textContent(), /separate sailing scene/);
            }
            await t.shot(page, 'atlas-' + name + '-' + t.sha.slice(0, 8));
            await button.click();
            t.assert.equal(await page.locator('#eb-voyage-canvas').isVisible(), true);
            await page.locator('#eb-voyage-exit').click();
            t.assert.deepEqual(page.errors, []);
            t.record('Atlas and passage entry ' + name, 'PASS');
            console.log('PASS Atlas and passage entry', name);
            await page.close();
        }
    } finally {
        await browser.close();
    }
})().catch(error => {
    console.error(error);
    process.exitCode = 1;
});
