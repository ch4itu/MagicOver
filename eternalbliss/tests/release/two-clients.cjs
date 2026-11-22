const fs = require('node:fs');
const path = require('node:path');
const {spawn} = require('node:child_process');
const {createInterface} = require('node:readline');
const {performance} = require('node:perf_hooks');
const t = require('../browser.cjs');

const source = process.env.EB_SOURCE || path.resolve(__dirname,'../../index.html');
let html = fs.readFileSync(source,'utf8');
function intercept(needle, replacement) {
    t.assert(html.includes(needle),needle);
    html=html.replace(needle,replacement);
}
// Only the transport and session bootstrap are substituted. Moving, decoding, room transitions,
// interpolation, disconnect handling and leaving run in the real browser client.
intercept('view = new WorldView(canvas, layer, world);','view = new WorldView(canvas, layer, world); window.__view=view;');
intercept('    function refresh(connection = session) {',
    '    function refresh(connection = session) { return window.ledgerRead().then(row=>window.worldFixture.feed(row)); }\n    function liveRefresh(connection = session) {');
intercept('    async function send(op, sequence, payload = [], registry = null, expected = null, books = []) {',
    '    async function send(op, sequence, payload = []) { const row=await window.ledgerWrite({op,sequence,payload:[...payload]}); window.worldFixture.feed(row); }\n    async function liveSend(op, sequence, payload = [], registry = null, expected = null, books = []) {');
intercept('    return {\n        active:() => Boolean(session?.entered),',`    window.worldFixture = {
        idle:()=>!pending.length&&!sending,
        feed(row){
            if(!session)return;
            const bytes=new Uint8Array(40);
            new DataView(bytes.buffer).setBigInt64(32,BigInt(row.time),true);
            acceptClock({owner:'Sysvar1111111111111111111111111111111111111',data:[btoa(String.fromCharCode(...bytes)),'base64']},row.slot);
            acceptWorld(session,{owner:sharedWorldProgramId,data:[row.data,'base64']},row.slot);
        },
        join(row,index){
            const actor={publicKey:row.owners[index],name:row.names[index]};
            const solo={map:captureAdventureMap(),player:{...gameState.player},inventory:gameState.inventory,
                stats:gameState.stats,sailing:gameState.sailingMoves,room:null};
            const map=structuredClone(BLISS_MAP_DATA);
            for(const group of [map.enemies,map.items,map.buildings]) group.forEach((data,i)=>{ data.sharedId=i; });
            localStorage.setItem('fixture-identity',actor.publicKey);
            session={player:actor,solo,entered:true,book:'fixture-book',terrainRevision:-1,enemies:map.enemies.slice(),items:map.items.slice()};
            gameState.inventory={...solo.inventory};gameState.stats={...solo.stats};
            applyAdventureMap(map,{...solo.player});
            this.feed(row); setMode(true);
        }
    };
    return {
        active:() => Boolean(session?.entered),`);
intercept('    function saveDue(now,delay = saveDelay) {','    function saveDue(){return false;}\n    function liveSaveDue(now,delay = saveDelay) {');

const child=spawn('python',['-u',path.join(__dirname,'shared-ledger-fixture.py')],{stdio:['pipe','pipe','inherit']});
const waiting=[];
createInterface({input:child.stdout}).on('line',line=>{
    const response=JSON.parse(line), request=waiting.shift();
    if(response.error)request.reject(Error(response.error));else request.resolve(response);
});
child.on('exit',code=>{for(const request of waiting.splice(0))request.reject(Error('Ledger exited '+code));});
function ledger(request){
    return new Promise((resolve,reject)=>{
        waiting.push({resolve,reject});
        child.stdin.write(JSON.stringify(request)+'\n');
    });
}
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const quantile=(samples,p)=>[...samples].sort((a,b)=>a-b)[Math.ceil(samples.length*p)-1];

(async()=>{
    const browser=await t.launch();
    try {
        let time=1000;
        const before=await ledger({op:'join',index:0,time});
        const joined=await ledger({op:'join',index:1,time});
        const clients=[];
        const blocked=[false,false];
        const samples=[];
        async function boot(index,context){
            const page=await context.newPage();
            page.setDefaultTimeout(20000);
            const errors=[];
            page.on('pageerror',error=>errors.push(error.message));
            await page.route('**/*',route=>route.request().url().includes('/eternalbliss/')?
                route.fulfill({contentType:'text/html',body:html}):route.abort());
            await page.routeWebSocket('**/*',ws=>ws.close());
            await page.exposeFunction('ledgerRead',()=>{
                if(blocked[index])throw Error('Fixture connection unavailable');
                return ledger({op:'read',time});
            });
            await page.exposeFunction('ledgerWrite',async args=>{
                if(blocked[index])throw Error('Fixture connection unavailable');
                const started=performance.now();
                let row;
                try { row=await ledger({...args,index,time}); }
                catch(error){ console.error('Ledger rejected',index,args,error.message);throw error; }
                if([19,24,26].includes(args.op))for(let i=0;i<clients.length;i++){
                    if(i!==index&&!blocked[i]){
                        await clients[i].evaluate(row=>worldFixture.feed(row),row);
                        if(args.op===19)samples.push(performance.now()-started);
                    }
                }
                return row;
            });
            await page.goto('http://127.0.0.1:8091/eternalbliss/',{waitUntil:'load'});
            await page.locator('#eb-menu-solo').click();
            await page.waitForFunction(()=>__view?.active&&ebFarlands.surface());
            page.fixtureErrors=errors;
            return page;
        }
        const ca=await browser.newContext({viewport:{width:900,height:560}});
        const cb=await browser.newContext({viewport:{width:900,height:560}});
        const a=await boot(0,ca);
        const b=await boot(1,cb);
        clients.push(a,b);
        await a.evaluate(row=>worldFixture.join(row,0),before);
        t.assert.equal(await a.evaluate(()=>sharedWorld.remote().length),0);
        await b.evaluate(row=>worldFixture.join(row,1),joined);
        await a.evaluate(row=>worldFixture.feed(row),joined);
        const identity=page=>page.evaluate(()=>localStorage.getItem('fixture-identity'));
        t.assert.notEqual(await identity(a),await identity(b));
        const remote=page=>page.evaluate(()=>sharedWorld.remote().map(({x,y,room})=>({x,y,room})));
        const pos=page=>page.evaluate(()=>({x:gameState.player.x,y:gameState.player.y}));
        t.assert.equal((await remote(a)).length,1);
        t.assert.equal((await remote(b)).length,1);
        console.log('Both independent clients see each other');
        const started=performance.now();
        for(let turn=0;turn<5;turn++){
            for(const [page,other] of [[a,b],[b,a]]){
                time++;
                const initial=await pos(page);
                const step=await page.evaluate(turn=>{
                    const directions=[[1,0],[0,1],[-1,0],[0,-1]];
                    return directions.find((_,i)=>{
                        const [dx,dy]=directions[(i+turn)%4];
                        return sharedWorld.move(dx,dy);
                    }) && [gameState.player.x,gameState.player.y];
                },turn);
                t.assert(step,'at least one cardinal step must be available');
                console.log('Step predicted',turn,page===a?'A':'B',initial,step);
                await other.waitForFunction(({x,y})=>sharedWorld.remote().some(p=>p.x===x&&p.y===y),
                    {x:step[0],y:step[1]});
                t.assert.equal(Math.abs(initial.x-step[0])+Math.abs(initial.y-step[1]),1);
                const expected=await pos(page);
                t.assert.equal((await remote(other))[0].x,expected.x);
                t.assert.equal((await remote(other))[0].y,expected.y);
                await delay(40);
            }
        }
        console.log('Movement confirmed on both clients');
        const throughput=samples.length*1000/(performance.now()-started);
        t.assert.equal(await a.evaluate(()=>sharedWorld.move(1,1)),false);
        const eye = a.locator('#eb-immersive-toggle');
        await eye.click();
        await a.waitForFunction(()=>__view.eyeActive());
        const yaw = await a.evaluate(()=>__view.eyeYaw);
        for (let turn=0; turn<6; turn++) await a.keyboard.press('KeyQ');
        t.assert(Math.abs(await a.evaluate(()=>__view.eyeYaw)-yaw-1.44)<.001);
        await a.waitForFunction(()=>worldFixture.idle());
        const eyeStart = await pos(a);
        time++;
        await a.keyboard.press('KeyW');
        await b.waitForFunction(({x,y})=>sharedWorld.remote().some(p=>p.x===x-1&&p.y===y),eyeStart);
        await a.waitForFunction(()=>worldFixture.idle());
        await t.shot(a,'shared-first-person-'+t.sha.slice(0,8));
        await eye.click();
        t.assert.equal(await a.evaluate(()=>__view.eyeMode),false);
        const diagonalStart = await pos(a);
        time++;
        await a.keyboard.down('KeyD');
        await a.keyboard.down('KeyS');
        try {
            await b.waitForFunction(({x,y})=>sharedWorld.remote().some(p=>p.x>x&&p.y>y),diagonalStart);
        } finally {
            await a.keyboard.up('KeyD');
            await a.keyboard.up('KeyS');
        }
        await a.waitForFunction(()=>worldFixture.idle());
        await b.waitForFunction(()=>{
            const [peer]=sharedWorld.remote();
            const record=[...__view.actors.values()].find(({kind})=>kind==='remote');
            return peer && record && Math.abs(record.model.group.position.x-peer.viewX)<.01 &&
                Math.abs(record.model.group.position.z-peer.viewY)<.01;
        });
        console.log('Shared camera-relative keyboard movement and diagonal steering confirmed');
        t.assert(await a.locator('#eb-farlands-horse').isHidden());

        // Put the same two signed records at a valid inn approach, then publish the placement
        // with an ordinary signed heartbeat to advance the world revision.
        const [inn,x,y]=await a.evaluate(()=>{
            const inn=buildings.find(b=>b.type==='inn'&&Number.isInteger(b.sharedId));
            const f=buildingFootprint(inn);
            return [inn.sharedId,f.x0-1,f.y0];
        });
        for(let index=0;index<2;index++)await ledger({op:'place',index,x,y,time:++time});
        const row=await ledger({op:'read',time});
        const payload=Buffer.from(row.data,'base64');
        const seq=payload.readUInt32LE(64+84);
        const placed=await ledger({op:25,index:0,sequence:seq,time:++time});
        for(const page of clients)await page.evaluate(row=>worldFixture.feed(row),placed);
        // Reconciliation only runs when a player's sequence advances, so confirm both records.
        const moved=await ledger({op:25,index:1,sequence:payload.readUInt32LE(64+112+84),time:++time});
        for(const page of clients)await page.evaluate(row=>worldFixture.feed(row),moved);
        t.assert(await a.evaluate(id=>sharedWorld.toggleRoom(buildings.find(b=>b.sharedId===id)),inn));
        t.assert.equal((await remote(b)).length,0);
        t.assert(await b.evaluate(id=>sharedWorld.toggleRoom(buildings.find(b=>b.sharedId===id)),inn));
        t.assert.equal((await remote(a)).length,1);
        t.assert.equal((await remote(b)).length,1);
        for(const page of clients)t.assert(await page.evaluate(()=>sharedWorld.toggleRoom(buildingInterior.building)));

        console.log('Same-room and separate-room visibility confirmed');
        blocked[0]=true;
        const stale=await remote(a);
        time++;
        const peerStep=await b.evaluate(()=>{
            const step=[[1,0],[0,1],[-1,0],[0,-1]].some(([x,y])=>sharedWorld.move(x,y));
            return step && {x:gameState.player.x,y:gameState.player.y};
        });
        t.assert(peerStep);
        await b.waitForFunction(({x,y})=>sharedWorld.ranks().some(p=>p.name==='ash-hero'&&p.x===x&&p.y===y),peerStep);
        t.assert.deepEqual(await remote(a),stale);
        await a.evaluate(()=>sharedWorld.leave());
        const retained=await a.evaluate(()=>({shared:sharedWorld.active(),status:sharedWorld.status()}));
        t.assert(retained.shared,'a failed leave must retain the shared session');
        t.assert.match(retained.status,/Could not confirm leaving/);
        t.assert.equal((await remote(b)).length,1);
        blocked[0]=false;
        await a.evaluate(async()=>worldFixture.feed(await ledgerRead()));
        t.assert.deepEqual(await remote(a),[{...peerStep,room:0}]);
        await a.evaluate(()=>sharedWorld.leave());
        t.assert.equal((await remote(b)).length,0);
        t.assert.equal(await a.evaluate(()=>sharedWorld.active()),false);

        const owner=await identity(a);
        await a.reload({waitUntil:'load'});
        t.assert.equal(await identity(a),owner);
        await a.locator('#eb-menu-solo').click();
        const rejoined=await ledger({op:'join',index:0,time:++time});
        await a.evaluate(row=>worldFixture.join(row,0),rejoined);
        await b.evaluate(row=>worldFixture.feed(row),rejoined);
        t.assert.equal((await remote(b)).length,1);
        t.assert.equal(await identity(a),owner);
        async function frames(page) {
            return page.evaluate(async()=>{
                const frames=[];
                let last=await new Promise(requestAnimationFrame);
                for(let i=0;i<36;i++){
                    const now=await new Promise(requestAnimationFrame);
                    if(i>=4)frames.push(now-last);
                    last=now;
                }
                return frames;
            });
        }
        const frameTimes=await frames(a);
        // Same account state and browser processes, with both WebGL scenes disabled.
        for(const page of clients)await page.locator('#eb-view-toggle').click();
        const mapTimes=await frames(a);
        const result={cameraRelativeKeyboard:true,diagonalSteering:true,transport:'local Node bridge, signed LiteSVM ledger; browser session and signer provided by fixture',
            accountBytes:Buffer.from(joined.data,'base64').length,base64Bytes:joined.data.length,
            samples:samples.length,deliveryMs:{p50:quantile(samples,.5),p95:quantile(samples,.95)},updatesPerSecond:throughput,
            frameMs:{p50:quantile(frameTimes,.5),p95:quantile(frameTimes,.95)},mapFrameMs:{p50:quantile(mapTimes,.5),p95:quantile(mapTimes,.95)},failedLeave:retained};
        console.log(JSON.stringify(result));
        t.assert.deepEqual(a.fixtureErrors,[]);
        t.assert.deepEqual(b.fixtureErrors,[]);
        t.record('Two isolated Chromium clients against signed LiteSVM',result);
    }finally{await browser.close();child.stdin.end();}
})().catch(error=>{console.error(error);process.exitCode=1;});
