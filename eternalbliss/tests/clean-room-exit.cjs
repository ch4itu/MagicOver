const t = require('./browser.cjs');

const profiles = [
    ['desktop-overhead', {viewport:{width:1024,height:700}}, false],
    ['desktop-first-person', {viewport:{width:1024,height:700}}, true],
    ['portrait-overhead', {viewport:{width:390,height:844},hasTouch:true,isMobile:true}, false],
    ['landscape-first-person', {viewport:{width:844,height:390},hasTouch:true,isMobile:true}, true],
];

(async () => {
    const browser = await t.launch();
    try {
        for (const [name, profile, eye] of profiles) {
            const page = await t.page(browser, profile, (1767225600 + 4*480 + 240)*1000);
            const exit = page.locator('#eb-clean-room-exit');
            const normalExit = page.locator('#eb-room-exit');
            const clean = page.locator('#eb-capture-toggle');
            const panel = page.locator('#eb-actions-collapse');
            const press = locator => profile.hasTouch ? locator.tap() : locator.click();
            if (eye) await press(page.locator('#eb-immersive-toggle'));
            const enter = () => page.evaluate(() => {
                const inn = buildings.find(b => b.realmId === 'rest-service-0');
                const approach = {x:inn.x, y:inn.y+2};
                Object.assign(gameState.player, approach);
                enterBuilding(inn);
                return approach;
            });
            t.assert(await exit.isHidden());
            await enter();
            await page.waitForFunction(() => Boolean(buildingInterior));
            t.assert(await exit.isHidden());
            const panelWasClosed = await panel.getAttribute('aria-expanded') === 'false';
            if (panelWasClosed) await press(panel);
            await normalExit.waitFor({state:'visible'});
            await press(normalExit);
            await page.waitForFunction(() => !buildingInterior);
            if (panelWasClosed) await press(panel);
            await press(clean);
            t.assert(await exit.isHidden());
            const location = page.locator('.location-info');
            const locationWasHidden = await location.isHidden();

            const approach = await enter();
            await page.waitForFunction(eye => Boolean(__view.room) && __view.eyeActive() === eye, eye);
            const frame = await page.evaluate(() => __view.renderer.info.render.frame);
            await page.waitForFunction(frame => __view.renderer.info.render.frame >= frame + 3, frame);
            await exit.waitFor({state:'visible'});
            t.assert(await normalExit.isHidden());
            const bounds = await exit.boundingBox();
            t.assert(bounds.height >= 44 && bounds.x >= 0 && bounds.y >= 0);
            for (const selector of ['.location-info','#eb-capture-toggle','#mobile-controls','#eb-joystick','#eb-eye-actions','#eb-immersive-look']) {
                const control = page.locator(selector);
                if (await control.isHidden()) continue;
                const other = await control.boundingBox();
                t.assert(bounds.x+bounds.width <= other.x || other.x+other.width <= bounds.x ||
                    bounds.y+bounds.height <= other.y || other.y+other.height <= bounds.y, selector+' overlaps exit');
            }
            await t.shot(page, 'clean-room-exit-'+name+'-'+t.sha.slice(0,8));
            await press(exit);
            await exit.waitFor({state:'hidden'});
            await page.waitForFunction(eye => !__view.room && __view.eyeActive() === eye, eye);
            t.assert.deepEqual(await page.evaluate(() => ({x:gameState.player.x,y:gameState.player.y})), approach);
            t.assert.equal(await clean.getAttribute('aria-pressed'), 'true');
            t.assert.equal(await location.isHidden(), locationWasHidden);
            t.assert.deepEqual(page.errors, []);
            t.record('Clean View building exit '+name, 'PASS full UI exit, Clean View enter and exit, control clearance, returned position and retained camera');
            console.log('PASS', name);
            await page.close();
        }
    } finally {
        await browser.close();
    }
})().catch(error => { console.error(error); process.exitCode = 1; });
