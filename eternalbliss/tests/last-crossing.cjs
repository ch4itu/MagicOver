const t = require('./browser.cjs');

const phone={viewport:{width:390,height:844},isMobile:true,hasTouch:true};
const fullHarvest=['met','key','gate','guardian','relic','reward'].map(s=>'quest-vault-'+s);
const clock=new Date((1767225600+4*480+240)*1000);
const title=page=>page.locator('#modalTitle');
const content=page=>page.locator('#modalContent');
const flag=(page,s)=>page.evaluate(s=>ebFarlands.storyFlag('quest-vault-crossing-'+s),s);
const close=page=>page.keyboard.press('Escape');
const gold=page=>page.evaluate(()=>gameState.inventory.gold);

async function target(page,x,y,yaw){
    await page.evaluate(({x,y,yaw})=>{
        Object.assign(gameState.player,{x,y,targetX:x,targetY:y});
        const view=__view;
        view.hero.group.position.set(x,.055,y);
        view.hero.lastX=x;
        view.hero.lastZ=y;
        view.cameraPosition.set(x,y);
        if(yaw!==undefined)view.eyeYaw=view.eyeRenderYaw=yaw;
        renderWorld();
        document.activeElement?.blur();
    },{x,y,yaw});
}
async function go(page,id){
    const [x,y]=await page.evaluate(id=>{const s=Farlands.byId.get(id);return [s.x,s.y+2];},id);
    await target(page,x,y);
    await page.evaluate(()=>{ebFarlands.refreshScene(true);renderWorld();});
}
async function mira(page){
    await page.evaluate(()=>{
        const n=npcs.find(n=>n.name==='Mira, the Cartographer');
        if(!n)throw Error('Mira missing');
        Object.assign(gameState.player,{x:n.x,y:n.y});
        useWorldActor('npc',n);
    });
    await title(page).filter({hasText:'Mira'}).waitFor();
}
async function questTab(page){
    const tab=page.getByRole('tab',{name:/Quests/});
    if(await tab.isVisible()&&await tab.getAttribute('aria-selected')!=='true')await tab.click();
}
async function enter(page){
    await page.evaluate(()=>{
        const b=buildings.find(b=>b.realmId==='toll-structure');
        if(!b)throw Error('Tollhouse missing');
        const site=Farlands.byId.get('toll');
        Object.assign(gameState.player,{x:site.x+2,y:site.y});
        enterBuilding(b);
    });
    await page.waitForFunction(()=>__view.room?.questDungeon);
}
async function fight(page,press){
    await page.waitForFunction(()=>gameState.currentEnemy?.name==='Siltbound Keeper');
    for(let turn=0;turn<12;turn++){
        const state=await page.evaluate(()=>({battle:gameState.inBattle,foe:gameState.currentEnemy?.hp,
            mp:gameState.player.mp,hp:gameState.player.hp,potions:gameState.inventory.healthPotions}));
        if(!state.battle||state.foe<=0)break;
        const action=state.hp<30&&state.potions?'heal':state.mp>=12?'magic':'attack';
        await press(page.locator(`[data-eb-combat="${action}"]:enabled`));
        await page.waitForFunction(()=>!gameState.inBattle||gameState.currentEnemy?.hp<=0||!ebCombatHUD.waiting());
    }
    await page.waitForFunction(()=>!gameState.inBattle);
}
async function walkThroughGate(page,mobile){
    if(!mobile){await page.keyboard.press('ArrowUp');return;}
    const box=await page.locator('#eb-joystick').boundingBox();
    const cdp=await page.context().newCDPSession(page);
    const x=box.x+box.width/2,y=box.y+box.height/2;
    await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x,y}]});
    try{
        await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x,y:y-38}]});
        await page.waitForFunction(()=>gameState.player.y>=9);
    }finally{
        await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});
        await cdp.detach();
    }
}
async function run(browser,name,profile){
    const page=await t.page(browser,profile,clock);
    const mobile=Boolean(profile?.hasTouch);
    const press=locator=>mobile?locator.tap():locator.click();
    const use=()=>press(page.locator('#eb-eye-use'));
    try{
        await mira(page);
        t.assert.equal(await page.getByRole('button',{name:'Ask about the flood chart'}).count(),0);
        await close(page);
        await page.evaluate(spent=>{
            const {x,y}=Farlands.start;
            ebFarlands.leave();
            ebFarlands.restore({version:1,active:true,place:{x,y:y+1,sailing:0},spent});
            // A post-Harvest character fixture. All new progression and battle inputs are real.
            Object.assign(gameState.player,{hp:110,maxHp:110,mp:70,maxMp:70,attack:14,magic:18,defense:8});
            ebFarlands.saveStoryNow();
        },fullHarvest);
        const before=await gold(page);
        await mira(page);
        await questTab(page);
        await press(page.getByRole('button',{name:'Ask about the flood chart'}));
        t.assert(await flag(page,'started'));
        t.assert.equal(await page.evaluate(()=>ebFarlands.pinned()),'mill');
        await go(page,'toll');
        await enter(page);
        await press(page.locator('#eb-immersive-toggle'));
        await page.waitForFunction(()=>__view.eyeActive());
        await target(page,12,8,Math.PI);
        await use();
        await title(page).filter({hasText:'The flood door'}).waitFor();
        t.assert((await content(page).textContent()).includes('Millbend'));
        t.assert.equal(await page.getByRole('button',{name:'Boat',exact:true}).count(),0);
        await close(page);
        await press(page.locator('#eb-room-exit'));
        await go(page,'mill');
        const marker=await page.evaluate(()=>[...Farlands.authoredProps.values()].find(p=>p.name==='Millbend flood marker'));
        await target(page,marker.x,marker.y,0);
        await page.keyboard.press('Space');
        await title(page).filter({hasText:'Millbend flood marker'}).waitFor();
        t.assert(await flag(page,'clue'));
        t.assert.equal(await page.evaluate(()=>ebFarlands.pinned()),'toll');
        await close(page);

        await go(page,'toll');
        await enter(page);
        await target(page,12,8,Math.PI);
        await use();
        await title(page).filter({hasText:'The flood door'}).waitFor();
        const sign=label=>press(content(page).getByRole('button',{name:label,exact:true}));
        await sign('Bell');
        t.assert((await content(page).textContent()).includes('Start again'));
        t.assert(!await flag(page,'lock'));
        t.assert(await page.evaluate(()=>!canEntityMoveTo(12,9)));
        for(const label of ['Boat','Grain','Bell'])await sign(label);
        t.assert(await flag(page,'lock'));
        await page.waitForFunction(()=>canEntityMoveTo(12,9));
        await t.shot(page,'crossing-open-'+name+'-'+t.sha.slice(0,8));
        await walkThroughGate(page,mobile);
        await page.waitForFunction(()=>gameState.player.y>=9);

        await target(page,13,10,Math.PI);
        const combatGold=await gold(page);
        await use();
        await fight(page,press);
        t.assert(await flag(page,'sentinel'));
        t.assert.equal(await gold(page),combatGold+18);
        await page.reload({waitUntil:'domcontentloaded'});
        await press(page.locator('#eb-menu-solo'));
        await page.waitForFunction(()=>buildingInterior?.questDungeon);
        t.assert(await flag(page,'sentinel'));
        t.assert.equal(await gold(page),combatGold+18);
        if(!await page.evaluate(()=>__view.eyeActive()))await press(page.locator('#eb-immersive-toggle'));
        await page.waitForFunction(()=>__view.eyeActive());
        await target(page,10,11,Math.PI);
        await use();
        await title(page).filter({hasText:'A private tally'}).waitFor();
        await close(page);
        t.assert(await flag(page,'margin'));
        await target(page,13,11,Math.PI);
        await use();
        await title(page).filter({hasText:'The rescued names'}).waitFor();
        t.assert(await flag(page,'chart'));
        await t.shot(page,'crossing-discovery-'+name+'-'+t.sha.slice(0,8));
        await close(page);
        await page.reload({waitUntil:'domcontentloaded'});
        await press(page.locator('#eb-menu-solo'));
        await page.waitForFunction(()=>buildingInterior?.questDungeon);
        t.assert(await page.evaluate(()=>canEntityMoveTo(12,9) && enemies.every(e=>e.name!=='Siltbound Keeper')));
        t.assert.equal(await gold(page),combatGold+18);
        await press(page.locator('#eb-room-exit'));

        await go(page,'rest');
        await mira(page);
        await questTab(page);
        await press(page.getByRole('button',{name:'Record the rescued names'}));
        await title(page).filter({hasText:'The river remembers'}).waitFor();
        t.assert(await flag(page,'done'));
        t.assert.equal(await gold(page),before+42);
        await close(page);
        await mira(page);
        t.assert((await content(page).textContent()).includes('You brought her home twice'));
        t.assert.equal(await page.getByRole('button',{name:'Record the rescued names'}).count(),0);
        t.assert.equal(await gold(page),before+42);
        await close(page);
        await page.reload({waitUntil:'domcontentloaded'});
        await press(page.locator('#eb-menu-solo'));
        await page.waitForFunction(()=>ebFarlands.surface());
        t.assert(await page.evaluate(()=>['started','clue','lock','sentinel','margin','chart','done'].every(s=>
            ebFarlands.storyFlag('quest-vault-crossing-'+s))));
        t.assert.equal(await gold(page),before+42);
        t.assert.deepEqual(page.errors,[]);
        t.record('Tollkeeper crossing '+name,'PASS post-Harvest NPC offer, inaccessible pre-clue gate, marker and Atlas, wrong and correct sign order, gate movement, actual battle, secret and chart, interior reload, once-only conclusion and reload');
        console.log('PASS',name);
    }catch(e){
        console.error(await page.evaluate(()=>({title:document.getElementById('modalTitle').textContent,
            position:[gameState.player.x,gameState.player.y],interior:!!buildingInterior,battle:gameState.currentEnemy?.name,
            quest:ebStory.objective()})),page.errors);
        throw e;
    }finally{await page.close();}
}
(async()=>{
    const b=await t.launch();
    try{
        const cases=[['desktop',undefined],['portrait',phone]];
        for(const [name,profile] of cases){
            if(process.env.EB_CROSSING_CASE&&process.env.EB_CROSSING_CASE!==name)continue;
            await run(b,name,profile);
        }
    }finally{await b.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
