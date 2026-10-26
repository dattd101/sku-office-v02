'use client';
import {useCallback,useEffect,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {FileText,Table2,Presentation,FolderOpen,Clock3,RefreshCw,Trash2,LayoutGrid,ShieldCheck} from 'lucide-react';
import type {DocumentSession} from '@/lib/session';
import type {Mode} from '@/lib/types';
const apps=[{mode:'word' as Mode,name:'Word',desc:'Documents & writing',icon:FileText,color:'#2563eb'},{mode:'excel' as Mode,name:'Excel',desc:'Workbooks & data',icon:Table2,color:'#16834a'},{mode:'powerpoint' as Mode,name:'PowerPoint',desc:'Slides & presentations',icon:Presentation,color:'#ea6b32'}];
export default function Manager(){
  const [sessions,setSessions]=useState<DocumentSession[]>([]);
  const [error,setError]=useState('');
  const [busy,setBusy]=useState(false);
  const refresh=useCallback(async()=>{try{setSessions(await invoke<DocumentSession[]>('list_sessions'));setError('')}catch(e){setError(String(e))}},[]);
  useEffect(()=>{void refresh();const timer=setInterval(()=>void refresh(),5000);return()=>clearInterval(timer)},[refresh]);
  const start=async(mode:Mode)=>{setBusy(true);try{const s=await invoke<DocumentSession>('create_session',{mode});await invoke('open_editor_window',{id:s.id});await refresh()}catch(e){setError(String(e))}finally{setBusy(false)}};
  const open=async()=>{setBusy(true);try{const s=await invoke<DocumentSession|null>('import_session');if(s){await invoke('open_editor_window',{id:s.id});await refresh()}}catch(e){setError(String(e))}finally{setBusy(false)}};
  const resume=async(id:string)=>{try{await invoke('open_editor_window',{id})}catch(e){setError(String(e))}};
  const discard=async(s:DocumentSession)=>{if(!confirm(`Remove local recovery entry for ${s.path?.split(/[\\/]/).pop()??'Untitled'}? Do not do this while its editor is open.`))return;try{await invoke('discard_session',{id:s.id});await refresh()}catch(e){setError(String(e))}};
  return <div className="manager-shell"><header className="manager-header"><div className="manager-brand"><LayoutGrid size={22}/> Rust<span>Office</span><small>WORKSPACE</small></div><div className="manager-secure"><ShieldCheck size={17}/> Local-first desktop</div></header>
    <div className="manager-main"><section className="manager-welcome"><div className="manager-eyebrow">YOUR WORKSPACE</div><h1>What would you like to create?</h1><p>Start a document, workbook or presentation. Each editor opens in its own window and keeps its own recovery state.</p></section>
    <section className="manager-cards">{apps.map(a=><button disabled={busy} key={a.mode} className="manager-card" onClick={()=>void start(a.mode)}><span className="manager-app-icon" style={{background:a.color+'18',color:a.color}}><a.icon size={29}/></span><strong>{a.name}</strong><span>{a.desc}</span><span className="manager-card-cta">Create new →</span></button>)}</section>
    <section className="manager-files"><div className="manager-section-head"><div><h2>Documents & recovery</h2><p>Local snapshots are retained after closing an editor or unexpected shutdown.</p></div><div className="manager-buttons"><button onClick={()=>void refresh()}><RefreshCw size={16}/> Refresh</button><button className="manager-open" disabled={busy} onClick={()=>void open()}><FolderOpen size={16}/> Open file</button></div></div>
    {error&&<div className="manager-error">{error}</div>}
    {sessions.length===0?<div className="manager-empty"><Clock3 size={26}/><strong>No recent documents</strong><span>Create a document or open an existing Office file to get started.</span></div>:<div className="manager-list">{sessions.map(s=>{const a=apps.find(a=>a.mode===s.document.mode)??apps[0];const Icon=a.icon;return <div className="manager-file" key={s.id}><span className="manager-file-icon" style={{color:a.color,background:a.color+'14'}}><Icon size={22}/></span><button className="manager-file-title" onClick={()=>void resume(s.id)}><strong>{s.path?.split(/[\\/]/).pop()??`Untitled.${a.mode==='word'?'docx':a.mode==='excel'?'xlsx':'pptx'}`}</strong><small>{a.name} · {new Date(s.updated_at*1000).toLocaleString()} · {s.dirty?'Unsaved recovery available':'Saved snapshot'}</small></button><button className="manager-resume" onClick={()=>void resume(s.id)}>Open</button><button title="Remove recovery entry" className="manager-remove" onClick={()=>void discard(s)}><Trash2 size={17}/></button></div>})}</div>}
    </section></div><footer className="manager-footer">Rust Office v0.5 · Next.js 15 · Tauri 2 · Rust · Recovery snapshots are not a substitute for Save</footer></div>
}
