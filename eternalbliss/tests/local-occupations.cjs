const t = require('./browser.cjs');

const locations = [
    { site: 'settlement-rest-0', role: 'Farmer', kind: 'harvest', note: /gather the ripe crop/ },
    { site: 'settlement-lantern-0', role: 'Provisioner', kind: 'sortCatch', note: /grade the shell baskets from Pearl Banks/ },
];

(async () => {
    const browser = await t.launch();
    try {
        const day = (1767225600 + 4 * 480 + 200) * 1000; // 10:00, both morning shifts
        const page = await t.page(browser, undefined, day);
        const evidence = [];
        for (const {site,role,kind,note} of locations) {
            const worker = await page.evaluate(({site,role}) => {
                const place = Farlands.byId.get(site);
                applyAdventureMap(captureAdventureMap(), {...gameState.player,x:place.x,y:place.y+1});
                const npc = npcs.find(n => n.ebSite === site && n.ebRole === role);
                if (!npc) throw Error('Missing on-shift resident at ' + site);
                const approach = [[0,1],[0,-1],[1,0],[-1,0]]
                    .map(([dx,dy]) => [npc.x+dx,npc.y+dy])
                    .find(([x,y]) => canEntityMoveTo(x,y));
                if (!approach) throw Error('No walkable approach to ' + npc.name);
                applyAdventureMap(captureAdventureMap(), {...gameState.player,x:approach[0],y:approach[1]});
                return {name:npc.name,label:ebDisplayName(npc)};
            }, {site,role});
            await page.waitForFunction(({worker,kind}) => {
                const npc = npcs.find(n => n.name === worker.name);
                const actor = [...__view.actors.values()].find(record => record.data === npc);
                return ebLife.activity(npc, performance.now())?.kind === kind &&
                    Object.values(actor?.model?.ebProps ?? {}).some(prop => prop.visible);
            }, {worker,kind});
            await page.click('#eb-farlands-people');
            const row = page.locator('#modalContent > div').filter({has:page.getByText(worker.label,{exact:true})});
            await row.getByRole('button',{name:'Talk',exact:true}).click();
            const work = page.getByRole('button',{name:'About your work',exact:true});
            if (await work.isHidden()) await page.getByRole('tab',{name:/Talk$/}).click();
            await work.click();
            await page.locator('#modalContent [data-eb-pane="talk"] p').filter({hasText:note}).waitFor();
            await t.shot(page, `work-${kind}-${t.sha.slice(0,8)}`);
            if (kind === 'sortCatch') {
                await page.getByRole('button',{name:'Mark Pearl Banks on the Atlas',exact:true}).click();
                t.assert.equal(await page.evaluate(()=>ebFarlands.pinned()), 'pearl');
            } else {
                await page.getByRole('button',{name:'Goodbye',exact:true}).click();
            }
            evidence.push({worker,kind});
        }
        t.assert.deepEqual(page.errors,[]);
        t.record('Local occupations', evidence);
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
