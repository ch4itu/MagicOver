const fs=require('node:fs');
const path=require('node:path');
const crypto=require('node:crypto');
const {spawnSync}=require('node:child_process');

const tests=path.resolve(__dirname,'..');
const html=path.resolve(tests,'../index.html');
const out=process.env.EB_EVIDENCE;
const source=process.env.EB_RC_CANDIDATE;
const baseline=process.env.EB_QUEST_BASELINE;
const rollback=process.env.EB_ROLLBACK_SOURCE;
const scope=process.env.EB_SCOPE_BASELINE;
if(!out||!source||!baseline||!rollback||!scope)throw Error('Set EB_EVIDENCE, EB_RC_CANDIDATE, EB_QUEST_BASELINE, EB_SCOPE_BASELINE and EB_ROLLBACK_SOURCE');
const hash=data=>crypto.createHash('sha256').update(data).digest('hex');
const checkSource=()=>{
    if(hash(fs.readFileSync(html))!==source)throw Error('Checkout changed from pinned release HTML');
};
checkSource();
fs.mkdirSync(out,{recursive:true});
const artifact=path.join(out,'pr30-matrix.json');
const jobs=[
    ['geography','coastal-geography.cjs'],
    ['quest-contract','automation-quests.cjs',{},[baseline]],
    ['shared-source-boundary','release/pr30-scope.cjs',{},[scope]],
    ['architecture-batches','architecture-batches.cjs',{},[scope]],
    ['local-occupations','local-occupations.cjs'],
    ['natural-harvest','natural-harvest.cjs'],
    ['mobile-touch','mobile-touch.cjs'],
    ['generated-mooring-desktop','release/vehicles.cjs',{EB_VEHICLE_CASE:'desktop'}],
    ['generated-mooring-portrait','release/vehicles.cjs',{EB_VEHICLE_CASE:'phone'}],
    ['generated-mooring-landscape','release/vehicles.cjs',{EB_VEHICLE_CASE:'landscape'}],
    ['fresh-first-visit','fresh-millwright.cjs'],
    ['save-and-rollback','automation-save-version.cjs',{EB_ROLLBACK_SOURCE:rollback}],
    ['authored-coastal-transport','coastal-boarding.cjs'],
    ['shellwick-quest','coastal-quest.cjs'],
    ['portrait-combat','combat.cjs',{EB_COMBAT_CASE:'phone:false'}],
    ['landscape-combat','combat.cjs',{EB_COMBAT_CASE:'landscape:false'}],
    ['startup-modes','smoke.cjs'],
    ['shared-active-save-guard','release/world-saves.cjs',{EB_WORLD_CASE:'desktop'}],
];
const state=fs.existsSync(artifact)?JSON.parse(fs.readFileSync(artifact,'utf8')):{source,jobs:{}};
if(state.source!==source)throw Error('Use a separate evidence directory for each candidate');
const write=()=>fs.writeFileSync(artifact,JSON.stringify(state,null,2)+'\n');
for(const [name,script,env={},args=[]] of jobs){
    if(process.env.EB_RC_ONLY&&!process.env.EB_RC_ONLY.split(',').includes(name))continue;
    checkSource();
    const scriptPath=path.join(tests,script);
    const inputs=[...args,...(env.EB_ROLLBACK_SOURCE?[env.EB_ROLLBACK_SOURCE]:[])];
    const harvestResume=name==='natural-harvest' && process.env.EB_ROUTE_RESUME==='1';
    if(harvestResume)inputs.push(path.join(out,'natural-harvest-storage.json'));
    const fingerprint=hash(JSON.stringify({source,script:hash(fs.readFileSync(scriptPath)),env,
        helper:hash(fs.readFileSync(path.join(tests,'browser.cjs'))),
        inputs:inputs.map(file=>hash(fs.readFileSync(file)))}));
    if(state.jobs[name]?.status==='PASS'&&state.jobs[name].fingerprint===fingerprint){
        console.log('KEEP',name);continue;
    }
    const log=name+'-'+Date.now()+'.log';
    const fd=fs.openSync(path.join(out,log),'w');
    const previous=state.jobs[name];
    const row=state.jobs[name]={status:'RUNNING',fingerprint,log,started:new Date().toISOString(),previous};
    write();
    console.log('START',name);
    const result=spawnSync(process.execPath,[scriptPath,...args],{
        env:{...process.env,EB_ROUTE_RESUME:harvestResume?'1':'',EB_SOURCE:html,...env},stdio:['ignore',fd,fd],timeout:name==='natural-harvest'?600000:240000
    });
    fs.closeSync(fd);
    Object.assign(row,{status:result.status===0?'PASS':'FAIL',code:result.status,signal:result.signal,
        error:result.error?.message,finished:new Date().toISOString()});
    write();
    console.log(row.status,name,log);
}
checkSource();
const totals=Object.values(state.jobs).reduce((all,row)=>(all[row.status]=(all[row.status]||0)+1,all),{});
console.log('MATRIX',JSON.stringify(totals));
if(totals.FAIL)process.exitCode=1;
