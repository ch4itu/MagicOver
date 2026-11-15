const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const {chromium}=require('playwright');
const assert=require('node:assert/strict');
const source=process.env.EB_SOURCE||path.join(__dirname,'../index.html');
const html=fs.readFileSync(source,'utf8'),sha=crypto.createHash('sha256').update(html).digest('hex');
const out=process.env.EB_EVIDENCE||'/workspace/.magic-over-setup/eternalbliss-evidence';fs.mkdirSync(out,{recursive:true});
async function launch(){return chromium.launch({executablePath:'/usr/lib/chromium/chromium',args:['--no-sandbox','--enable-unsafe-swiftshader']});}
async function page(browser,profile={viewport:{width:1280,height:800}}){
 const p=await browser.newPage(profile);p.errors=[];p.on('pageerror',e=>p.errors.push(e.message));
 await p.route('**/eternalbliss/',r=>r.fulfill({contentType:'text/html',body:html.replace('view = new WorldView(canvas, layer, world);','view = new WorldView(canvas, layer, world); window.__view=view;')}));
 await p.goto('http://127.0.0.1:8091/eternalbliss/',{waitUntil:'domcontentloaded'});await p.click('#eb-menu-solo');
 await p.waitForFunction(()=>window.__view?.active && ebFarlands.surface());return p;
}
async function shot(p,name){await p.screenshot({path:path.join(out,name+'.png')});}
function record(test,result){fs.appendFileSync(path.join(out,'ledger.jsonl'),JSON.stringify({time:new Date().toISOString(),sha,script:crypto.createHash('sha256').update(fs.readFileSync(require.main.filename)).digest('hex'),browser:'system Chromium / Playwright (emulated viewports)',test,result})+'\n');}
async function smoke(p){
 await p.click('#eb-immersive-toggle');await p.waitForFunction(()=>__view.eyeActive());
 await p.keyboard.press('KeyQ');await p.waitForTimeout(200);assert.equal(await p.evaluate(()=>Number.isFinite(__view.eyeCamera.position.y)),true);
 await shot(p,'smoke-fp-'+sha.slice(0,8));await p.click('#eb-immersive-toggle');assert.equal(await p.evaluate(()=>__view.eyeMode),false);
 await p.click('#eb-view-toggle');assert.equal(await p.evaluate(()=>document.documentElement.dataset.ebView),'2d');await p.click('#eb-view-toggle');assert.deepEqual(p.errors,[]);
}
module.exports={assert,launch,page,shot,record,smoke,sha,out};
