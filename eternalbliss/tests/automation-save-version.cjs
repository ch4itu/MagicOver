const t = require('./browser.cjs');
(async () => {
    const browser = await t.launch();
    try {
        const page = await t.page(browser);
        const slot = 'eternalbliss/adventure/v1/guest';
        const {valid, unknown} = await page.evaluate(key => {
            saveAdventure(key);
            const save = JSON.parse(localStorage.getItem(key));
            const site = Farlands.byId.get('settlement-lantern-0');
            const cell = Farlands.sceneCell(Math.floor(site.x / Farlands.cellSize), Math.floor(site.y / Farlands.cellSize));
            const room = cell.buildings.find(building => building.name === site.services[0]);
            const layout = roomLayout(room);
            const y = layout.findIndex(row => row.includes('E'));
            let water;
            for (let dy = -12; dy <= 12 && !water; dy++) {
                for (let dx = -12; dx <= 12; dx++) {
                    const x = Farlands.start.x + dx;
                    const y = Farlands.start.y + dy;
                    if (Farlands.sample(x, y) === 'water') {
                        water = {x, y};
                        break;
                    }
                }
            }
            Object.assign(save.farlands, {
                station: 'settlement-lantern-0',
                pin: 'pearl',
                place: {x: site.x, y: site.y + 1, sailing: 0, room: {id: room.realmId, x: layout[y].indexOf('E'), y}},
                horseOwned: true,
                horsePlace: {x: Farlands.start.x + 1, y: Farlands.start.y + 1},
                boat: {taken: water, parked: {...water, heading: 0.25}},
                edits: [[Farlands.start.y * Farlands.size + Farlands.start.x, 'road']],
                spent: ['rq-taken-b-settlement-lantern-0', 'rq-done-a-rest'],
            });
            save.inventory.gold = 450;
            const valid = JSON.stringify(save);
            save.farlands.version = 999;
            const unknown = JSON.stringify(save);
            localStorage.setItem(key, unknown);
            return {valid, unknown};
        }, slot);
        await page.reload({waitUntil: 'domcontentloaded'});
        await page.click('#eb-menu-solo');
        await page.waitForFunction(() => ebFarlands.surface());
        t.assert.match(await page.locator('#eb-save-status').textContent(), /different save version/);
        t.assert.notEqual(await page.evaluate(() => gameState.inventory.gold), 450);
        await page.evaluate(() => document.activeElement?.blur());
        const moves = await page.evaluate(() => gameState.player.moveCount);
        await page.keyboard.press('ArrowDown');
        await page.waitForFunction(count => gameState.player.moveCount > count, moves);
        await page.evaluate(() => soloSaves.flush());
        t.assert.equal(await page.evaluate(key => localStorage.getItem(key), slot), unknown);
        t.assert.deepEqual(page.errors, []);
        t.record('Unknown nested Farlands save version', 'PASS incompatible guest record retained after accepted input and attempted flush');
        console.log('PASS unknown Farlands version protection');
        await page.evaluate(({key, record}) => localStorage.setItem(key, record), {key: slot, record: valid});
        await page.reload({waitUntil: 'domcontentloaded'});
        await page.click('#eb-menu-solo');
        await page.waitForFunction(() => ebFarlands.inRealm() && buildingInterior);
        const restored = await page.evaluate(key => {
            saveAdventure(key);
            return JSON.parse(localStorage.getItem(key));
        }, slot);
        t.assert.equal(restored.inventory.gold, 450);
        const original = JSON.parse(valid).farlands;
        for (const field of ['version', 'station', 'pin', 'place', 'horseOwned', 'horsePlace', 'edits', 'spent', 'boat']) {
            t.assert.deepEqual(restored.farlands[field], original[field], field);
        }
        t.assert.deepEqual(page.errors, []);
        t.record('Farlands v1 round trip', 'PASS v1 checkpoint, hero and room coordinates, horse, parked and taken boat, terrain edit, pin, quest flags and gold');
        console.log('PASS Farlands v1 round trip');
    } finally {
        await browser.close();
    }
})().catch(error => {
    console.error(error);
    process.exitCode = 1;
});
