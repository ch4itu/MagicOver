/* Opt-in Devnet trial. Registers two test accounts through the shipped Smaran form.
   Only public keys, transaction signatures, poses and timing data are recorded. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {createHash,randomBytes} = require('node:crypto');
const {chromium} = require('playwright');

const out = process.env.EB_EVIDENCE;
const rpc = 'https://devnet-as.magicblock.app';
if (process.env.EB_PUBLIC_TRIAL !== '1' || !out) throw Error('Set EB_PUBLIC_TRIAL=1 and EB_EVIDENCE for a public Devnet trial');
fs.mkdirSync(out,{recursive:true});
const accounts = ['a','b'].map(letter => ({name:'ebperf-'+randomBytes(5).toString('hex')+'-'+letter,pass:randomBytes(24).toString('hex')}));
const sleep = ms => new Promise(resolve => setTimeout(resolve,ms));
const log = (kind, data) => fs.appendFileSync(path.join(out,'peer-events.jsonl'),JSON.stringify({kind,...data})+'\n');

function instrument(source) {
    function insert(needle, extra) {
        assert.equal(source.split(needle).length,2,needle);
        source=source.replace(needle,()=>needle+extra);
    }
    insert('view = new WorldView(canvas, layer, world);','window.__view=view;');
    insert('const watch = connection.stream?.receipt(id);', "trace('signed',{op,sequence,id});");
    insert('const revision = u32(40);', "trace('arrival',{revision,slot});");
    insert('const previousOwnSeq = me()?.seq;', "trace('decode',{revision,slot});");
    source=source.replace('            samplePlayers();\n            if (!connection.presentationFrame)',`            samplePlayers();
            trace('ingested',{revision,slot,peers:remote.map(p=>[p.name,p.seq,p.x,p.y])});
            if (!connection.presentationFrame)`);
    insert('        renderFrame(eyeEnabled) {', "if(eyeEnabled) trace('raf',{});");
    insert('this.renderer.render(this.scene, eyeEnabled ? this.eyeCamera : this.camera);', `if(eyeEnabled) {
        const peers=[...this.actors.values()].filter(r=>r.kind==='remote').map(r=>[r.data.name,r.data.seq,r.model.group.position.x,r.model.group.position.z]);
        trace('scene',{peers,triangles:this.renderer.info.render.triangles,calls:this.renderer.info.render.calls});
    }`);
    insert('if (element.style.top !== top) element.style.top = top;', "trace('classic',{peers:[[player.name,player.seq,pose.x,pose.y]]});");
    return source;
}
async function run(browser, file, index, last) {
    const raw=fs.readFileSync(file,'utf8'), html=instrument(raw);
    const sha=createHash('sha256').update(raw).digest('hex');
    const clients=[], writes=[], responses=[], errors=[];
    log('start',{sha,browser:browser.version(),viewport:[900,560]});
    try {
        for (const account of accounts) {
            const ctx=await browser.newContext({viewport:{width:900,height:560}});
            const page=await ctx.newPage();
            clients.push(page);
            page.on('pageerror',e=>errors.push(e.message));
            page.on('response',async res=>{
                try{
                    const req=res.request();
                    if(req.method()!=='POST'||new URL(req.url()).origin!==rpc)return;
                    const method=req.postDataJSON()?.method;
                    if(method!=='sendTransaction')return;
                    const reply=await res.json();
                    if(reply.result)responses.push(reply.result);
                }catch{}
            });
            await page.addInitScript(()=>{
                window.events=[];
                window.trace=(kind,data)=>{if(window.collect || kind==='signed')events.push({kind,t:performance.timeOrigin+performance.now(),...data});};
                const fetchReal=window.fetch.bind(window);
                window.fetch=async (url,options)=>{
                    let request;
                    try {request=JSON.parse(options?.body);}catch{}
                    const start=performance.timeOrigin+performance.now();
                    const result=await fetchReal(url,options);
                    if(request)trace('http',{method:request.method,start});
                    return result;
                };
                const Native=window.WebSocket;
                window.WebSocket=class extends Native{
                    constructor(...args){super(...args);this.addEventListener('message',e=>{
                        try{const m=JSON.parse(e.data);if(m.method==='accountNotification')trace('ws',{slot:m.params.result.context.slot});}catch{}
                    });}
                };
            });
            await page.route('http://127.0.0.1:8091/eternalbliss/',r=>r.fulfill({contentType:'text/html',body:html}));
            await page.goto('http://127.0.0.1:8091/eternalbliss/');
            await page.locator('#eb-menu-world').click();
            // Dismiss graphics load during registration with the ordinary view control.
            await page.evaluate(()=>{if(__view.active)document.querySelector('#eb-view-toggle').click()});
            if(index===0)await page.locator('#eb-smaran-mode').click();
            await page.locator('#eb-player-name').fill(account.name);
            await page.locator('#eb-passphrase').fill(account.pass);
            await page.locator('#eb-smaran-submit').click();
            try{
                await page.waitForFunction(()=>sharedWorld.active(),null,{timeout:120000,polling:100});
            }catch(e){
                throw Error('Join failed: '+await page.locator('#eb-smaran-status').textContent()+' '+await page.evaluate(()=>sharedWorld.status()));
            }
            log('join',{sha,name:account.name,owner:await page.evaluate(()=>sharedWorld.ranks().find(r=>r.name===gameState.player.name).owner)});
        }
        for(const p of clients)await p.waitForFunction(()=>sharedWorld.remote().length===1);
        for(const mode of ['2d','fp']){
            if(mode==='fp')for(const p of clients)await p.evaluate(()=>{
                document.querySelector('#eb-view-toggle').click();document.querySelector('#eb-immersive-toggle').click();
            });
            await sleep(2500);
            for(const p of clients)await p.evaluate(()=>{window.collect=true});
            for(let step=0;step<4;step++){
                const i=step%2, actor=clients[i], peer=clients[1-i];
                const dx=step<2?1:-1;
                const before=await peer.evaluate(()=>{const p=sharedWorld.remote()[0];return {x:p.x,y:p.y,seq:p.seq,name:p.name};});
                const start=await actor.evaluate(dx=>{
                    const time=performance.timeOrigin+performance.now();
                    if(!sharedWorld.move(dx,0)) throw Error('Ordinary cardinal input refused');
                    return time;
                },dx);
                const handle=await peer.waitForFunction(({before,dx,mode,start})=>{
                    const stream=mode==='fp'?'scene':'classic';
                    const hit=events.find(e=>e.kind==='ingested'&&e.t>=start&&e.peers.some(([name,seq,x,y])=>name===before.name&&seq>before.seq&&x===before.x+dx&&y===before.y));
                    if(!hit)return false;
                    const placed=events.find(e=>e.kind===stream&&e.t>=hit.t&&e.peers.some(([name,seq,x])=>name===before.name&&seq>before.seq&&Math.abs(x-(before.x+dx))<.48));
                    return placed&&{hit,placed};
                },{before,dx,mode,start},{timeout:45000,polling:100});
                const {hit,placed}=await handle.jsonValue();
                const sender=await actor.evaluate(start=>events.filter(e=>e.t>=start),start);
                const sign=sender.find(e=>e.kind==='signed'&&e.op===19);
                assert(sign);
                const http=sender.find(e=>e.kind==='http'&&e.method==='sendTransaction'&&e.start>=sign.t);
                const observer=await peer.evaluate(()=>events);
                const arrival=observer.find(e=>e.kind==='arrival'&&e.slot===hit.slot&&e.revision===hit.revision);
                writes.push(sign.id);
                log('move',{sha,mode,i,step,from:before,input:start,sign:sign.t,httpStart:http?.start,httpHeaders:http?.t,arrival:arrival?.t,ingested:hit.t,placed:placed.t,slot:hit.slot,id:sign.id});
                await sleep(400);
            }
            for(const [i,p] of clients.entries()){
                const rows=await p.evaluate(()=>{window.collect=false;return events.splice(0)});
                fs.writeFileSync(path.join(out,sha.slice(0,8)+'-'+mode+'-'+i+'.json'),JSON.stringify(rows));
            }
            await clients[0].screenshot({path:path.join(out,sha.slice(0,8)+'-'+mode+'.png')});
        }
        if(last && index>0){
            for(const p of clients)await p.evaluate(()=>document.querySelector('#eb-view-toggle').click());
            const [a,b]=clients;
            const owner=await b.evaluate(()=>sharedWorld.ranks().find(p=>p.name===gameState.player.name).owner);
            const from=await a.evaluate(()=>({x:gameState.player.x,y:gameState.player.y}));
            await b.context().setOffline(true);
            try{
                await b.evaluate(()=>sharedWorld.leave());
                assert(await b.evaluate(()=>sharedWorld.active()));
                assert(await a.evaluate(()=>sharedWorld.move(1,0)));
                await a.waitForFunction(x=>sharedWorld.ranks().find(p=>p.name===gameState.player.name).x===x,from.x+1);
            }finally{await b.context().setOffline(false);}
            await b.waitForFunction(x=>sharedWorld.remote().some(p=>p.x===x),from.x+1);
            log('reconnected',{sha});
            await b.evaluate(()=>sharedWorld.leave());
            await a.waitForFunction(()=>!sharedWorld.remote().length);
            await b.reload();
            await b.locator('#eb-menu-world').click();
            await b.evaluate(()=>{if(__view.active)document.querySelector('#eb-view-toggle').click()});
            await b.locator('#eb-player-name').fill(accounts[1].name);
            await b.locator('#eb-passphrase').fill(accounts[1].pass);
            await b.locator('#eb-smaran-submit').click();
            await b.waitForFunction(()=>sharedWorld.active(),null,{timeout:90000});
            await a.waitForFunction(key=>sharedWorld.remote().some(p=>p.owner===key),owner);
            log('reload-rejoin',{sha,owner});
            // Follow a real route to the same accessible inn with ordinary signed cardinal moves.
            for(const p of clients){
                const route=await p.evaluate(()=>{
                    const inn=buildings.find(b=>b.type==='inn'&&b.x===68&&b.y===65);
                    const start=gameState.player,q=[[start.x,start.y,[]]],seen=new Set();
                    for(let i=0;i<q.length;i++){
                        const [x,y,path]=q[i];
                        const f=buildingFootprint(inn);
                        if(Math.max(f.x0-x,0,x-f.x0-f.w+1)+Math.max(f.y0-y,0,y-f.y0-f.h+1)<=2)return path;
                        if(path.length>=35)continue;
                        for(const [dx,dy] of [[-1,0],[0,-1],[1,0],[0,1]]){
                            const X=x+dx,Y=y+dy,k=X+','+Y,tile=worldMap[Y]?.[X];
                            if(seen.has(k)||!tile||['water','door','mountain'].includes(tile)||solidStructure(X,Y)||
                                enemies.some(e=>Math.round(e.x)===X&&Math.round(e.y)===Y))continue;
                            seen.add(k);q.push([X,Y,[...path,[dx,dy,X,Y]]]);
                        }
                    }
                    throw Error('No ordinary route to inn');
                });
                for(const [dx,dy,x,y] of route){
                    await p.waitForFunction(()=>sharedWorld.readyToMove());
                    assert(await p.evaluate(([dx,dy])=>sharedWorld.move(dx,dy),[dx,dy]));
                    await p.waitForFunction(([x,y])=>sharedWorld.ranks().some(r=>r.name===gameState.player.name&&r.x===x&&r.y===y),[x,y]);
                }
                assert(await p.evaluate(()=>sharedWorld.toggleRoom(buildings.find(b=>b.type==='inn'&&b.x===68&&b.y===65))));
                if(p===a)await b.waitForFunction(()=>!sharedWorld.remote().length);
            }
            for(const p of clients)await p.waitForFunction(()=>buildingInterior && sharedWorld.remote().some(r=>r.room>0));
            for(const p of clients)assert(await p.evaluate(()=>sharedWorld.toggleRoom(buildingInterior.building)));
            for(const p of clients)await p.waitForFunction(()=>!buildingInterior&&sharedWorld.remote().some(r=>r.room===0));
            log('room-roundtrip',{sha});
        }
        assert.deepEqual(errors,[]);
    }finally{
        for(const page of clients){
            try{
                if(await page.evaluate(()=>sharedWorld.active())){
                    await page.evaluate(()=>{if(__view.active)document.querySelector('#eb-view-toggle').click()});
                    for(let n=0;n<4&&await page.evaluate(()=>sharedWorld.active());n++){
                        await page.evaluate(()=>sharedWorld.leave()); await sleep(1200);
                    }
                    assert.equal(await page.evaluate(()=>sharedWorld.active()),false);
                    log('left',{sha});
                }
            }catch(e){log('cleanup-error',{sha,error:e.message});process.exitCode=1;}
        }
        for(const p of clients)await p.context().close();
        if(writes.length){
            const result=await fetch(rpc,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({jsonrpc:'2.0',id:1,method:'getSignatureStatuses',params:[writes,{searchTransactionHistory:true}]})});
            const statuses=(await result.json()).result.value;
            log('receipts',{sha,writes,statuses,acknowledged:responses});
            assert(statuses.every(s=>s && !s.err && s.confirmationStatus==='finalized'));
        }
    }
}
(async()=>{
    const browser=await chromium.launch({executablePath:process.env.EB_CHROMIUM_PATH,args:['--no-sandbox','--enable-unsafe-swiftshader']});
    try{for(const [i,source] of process.argv.slice(2).entries())await run(browser,source,i,i===process.argv.length-3);}
    finally{await browser.close();}
})().catch(e=>{console.error(e.message);process.exitCode=1});
