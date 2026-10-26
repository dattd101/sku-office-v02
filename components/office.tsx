'use client';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { DocumentSession } from '@/lib/session';
import { FileText, Table2, Presentation, FolderOpen, Save, FilePlus2, Undo2, Redo2, Printer, Bold, Italic, Underline, AlignLeft, AlignCenter, AlignRight, List, Search, ChevronDown, Plus, Trash2, Download, PanelLeft, Maximize2, Type, LayoutTemplate, Sparkles, Grid3X3, Check, X, Menu, Copy } from 'lucide-react';
import { fresh, type OfficeDocument, type Mode } from '@/lib/types';

type FileResult={path:string;document:OfficeDocument};
const modes:{key:Mode;label:string;icon:typeof FileText}[]=[{key:'word',label:'Word',icon:FileText},{key:'excel',label:'Excel',icon:Table2},{key:'powerpoint',label:'PowerPoint',icon:Presentation}];
const ribbonTabs=['File','Home','Insert','Layout','Review','View'];
const colName=(i:number)=>{let s='';for(let n=i+1;n>0;n=Math.floor((n-1)/26))s=String.fromCharCode(65+(n-1)%26)+s;return s};
function RibbonButton({icon:Icon,label,onClick,disabled=false}:{icon:typeof FileText;label:string;onClick?:()=>void;disabled?:boolean}){return <button title={label} disabled={disabled} onClick={onClick} className="ribbon-action"><Icon size={19} strokeWidth={1.8}/><span>{label}</span></button>}
export default function Office(){
  const sessionId = typeof window !== "undefined" ? new URLSearchParams(window.location.search).get("session") : null;
  const [ready,setReady]=useState(false);
  const [loadError,setLoadError]=useState('');
  const latest=useRef<{document:OfficeDocument;path:string|null;dirty:boolean}|null>(null);
  const savedRevision=useRef(0);
  const revision=useRef(0);
  const [doc,setDoc]=useState<OfficeDocument>(()=>fresh());
  const [tab,setTab]=useState('Home');
  const [path,setPath]=useState<string|null>(null);
  const [dirty,setDirty]=useState(false);
  const [message,setMessage]=useState('Ready');
  const [dark,setDark]=useState(false);
  const [selectedCell,setSelectedCell]=useState('1:0');
  const [zoom,setZoom]=useState(100);
  const [history,setHistory]=useState<OfficeDocument[]>([]);
  const [future,setFuture]=useState<OfficeDocument[]>([]);
  const [search,setSearch]=useState('');
  const mode=doc.mode;
  latest.current={document:doc,path,dirty};
  useEffect(()=>{
    if(!sessionId){setLoadError('Missing session ID in editor URL.');return;}
    let active=true;
    invoke<DocumentSession>('get_session',{id:sessionId}).then(s=>{
      if(!active)return;
      setDoc(s.document);setPath(s.path);setDirty(s.dirty);setReady(true);
      setMessage(s.dirty?'Recovered unsaved changes from local snapshot':'Document ready');
    }).catch(e=>{if(active)setLoadError(`Cannot load document session: ${String(e)}`)});
    return()=>{active=false};
  },[sessionId]);
  useEffect(()=>{
    if(!ready||!sessionId)return;
    const n=++revision.current;
    const timer=setTimeout(()=>{
      if(n!==revision.current||!latest.current)return;
      const {document,path,dirty}=latest.current;
      invoke('snapshot_session',{id:sessionId,document,path,dirty}).then(()=>{savedRevision.current=n}).catch(e=>setMessage(`Recovery snapshot failed: ${String(e)}`));
    },850);
    return()=>clearTimeout(timer);
  },[doc,path,dirty,ready,sessionId]);
  useEffect(()=>{
    if(!sessionId)return;
    let disposed=false;
    let unlisten:(()=>void)|undefined;
    getCurrentWindow().onCloseRequested(async event=>{
      if(!latest.current)return;
      if(latest.current.dirty){
        event.preventDefault();
        const choice=window.confirm('Unsaved edits are kept in Recovery. Close this editor?');
        if(!choice)return;
      }
      const {document,path,dirty}=latest.current;
      try{await invoke('snapshot_session',{id:sessionId,document,path,dirty})}catch(e){console.error(e)}
      await getCurrentWindow().destroy();
    }).then(fn=>{if(disposed)fn();else unlisten=fn});
    return()=>{disposed=true;unlisten?.()};
  },[sessionId]);
  const update=useCallback((updater:(d:OfficeDocument)=>OfficeDocument)=>{
    setDoc(prev=>{const next=updater(prev);if(next===prev)return prev;setHistory(h=>[...h.slice(-29),prev]);setFuture([]);return next});setDirty(true);
  },[]);
  const newFile=async()=>{try{const s=await invoke<DocumentSession>('create_session',{mode});await invoke('open_editor_window',{id:s.id})}catch(e){setMessage(String(e))}};

  const open=async()=>{try{const result=await invoke<DocumentSession|null>('import_session');if(result){await invoke('open_editor_window',{id:result.id});setMessage('Opened in a separate window')}}catch(e){setMessage(`Open failed: ${String(e)}`)}};
  const save=async(as=false)=>{try{if(!sessionId)return;const result=await invoke<string|null>('save_session',{id:sessionId,document:doc,path,saveAs:as});if(result){setPath(result);setDirty(false);setMessage(`Saved ${result}`)}}catch(e){setMessage(`Save failed: ${String(e)}`)}};
  const undo=()=>{if(!history.length)return;const prev=history[history.length-1];setHistory(h=>h.slice(0,-1));setFuture(f=>[...f,doc]);setDoc(prev);setDirty(true)};
  const redo=()=>{if(!future.length)return;const next=future[future.length-1];setFuture(f=>f.slice(0,-1));setHistory(h=>[...h,doc]);setDoc(next);setDirty(true)};
  useEffect(()=>{const handler=(e:KeyboardEvent)=>{if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='s'){e.preventDefault();void save(e.shiftKey)}if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='o'){e.preventDefault();void open()}if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='n'){e.preventDefault();newFile()}if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='z'&&!(e.target instanceof HTMLTextAreaElement)&&!(e.target instanceof HTMLInputElement)){e.preventDefault();undo()}};window.addEventListener('keydown',handler);return()=>window.removeEventListener('keydown',handler)});
  const sheet=doc.sheets[doc.selected_sheet]??doc.sheets[0];
  const slide=doc.slides[doc.selected_slide]??doc.slides[0];
  const setCell=(key:string,value:string)=>update(d=>({...d,sheets:d.sheets.map((s,i)=>i===d.selected_sheet?{...s,cells:{...s.cells,[key]:value}}:s)}));
  const setSlide=(field:'title'|'body',value:string)=>update(d=>({...d,slides:d.slides.map((s,i)=>i===d.selected_slide?{...s,[field]:value}:s)}));
  const formula=sheet?.cells[selectedCell]??'';
  const words=doc.word.trim()?doc.word.trim().split(/\s+/).length:0;
  const title=path?.split(/[\\/]/).pop()??`Untitled.${mode==='word'?'docx':mode==='excel'?'xlsx':'pptx'}`;
  if(loadError)return <main style={{padding:32,fontFamily:'sans-serif'}}><h1>Unable to open editor</h1><p>{loadError}</p><button onClick={()=>void invoke('focus_manager')}>Return to Manager</button></main>;
  if(!ready)return <main style={{padding:32,fontFamily:'sans-serif'}}><h1>Opening document…</h1><p>Loading local recovery session.</p></main>;
  return <div className={`app-shell mode-${mode} ${dark?'dark-app':''}`}>
    <header className="titlebar"><div className="brand"><span className="brand-mark"><Grid3X3 size={18}/></span><span>Rust<span className="brand-light">Office</span></span><span className="brand-pill">MODERN</span></div><div className="document-name">{title}{dirty?' •':''}</div><div className="title-actions"><span className="autosave"><Check size={14}/> Desktop workspace</span><button className="icon-button" title="Toggle dark mode" onClick={()=>setDark(!dark)}><Sparkles size={17}/></button></div></header>
    <div className="topbar"><div className="app-switcher"><button className="app-tab" onClick={()=>void invoke('focus_manager')}><Grid3X3 size={17}/> Manager</button><span className="app-tab active">{mode==='word'?'Word':mode==='excel'?'Excel':'PowerPoint'}</span></div><div className="quick-actions"><button onClick={newFile}><FilePlus2 size={16}/> New</button><button onClick={()=>void open()}><FolderOpen size={16}/> Open</button><button onClick={()=>void save()}><Save size={16}/> Save</button><button onClick={()=>void save(true)}><Download size={16}/> Save as</button><span className="vertical-rule"/><button className="square" title="Undo" disabled={!history.length} onClick={undo}><Undo2 size={17}/></button><button className="square" title="Redo" disabled={!future.length} onClick={redo}><Redo2 size={17}/></button></div></div>
    <div className="ribbon-tabs">{ribbonTabs.map(t=><button key={t} className={tab===t?'selected':''} onClick={()=>setTab(t)}>{t}</button>)}<div className="ribbon-search"><Search size={15}/><input placeholder="Search commands" value={search} onChange={e=>setSearch(e.target.value)}/></div></div>
    <div className="ribbon-body">{tab==='File'?<><div className="ribbon-group"><RibbonButton icon={FilePlus2} label="New" onClick={newFile}/><RibbonButton icon={FolderOpen} label="Open" onClick={()=>void open()}/><RibbonButton icon={Save} label="Save" onClick={()=>void save()}/><RibbonButton icon={Download} label="Save As" onClick={()=>void save(true)}/><span className="group-label">Document</span></div></>:<>
      <div className="ribbon-group"><RibbonButton icon={Copy} label="Copy" onClick={()=>navigator.clipboard?.writeText(mode==='word'?doc.word:mode==='excel'?formula:slide?.title??'')}/><RibbonButton icon={Undo2} label="Undo" onClick={undo} disabled={!history.length}/><RibbonButton icon={Redo2} label="Redo" onClick={redo} disabled={!future.length}/><span className="group-label">Clipboard</span></div>
      <div className="ribbon-group font-group"><div className="font-row"><select aria-label="Font" defaultValue="Aptos"><option>Aptos</option><option>Calibri</option><option>Arial</option><option>Times New Roman</option></select><select aria-label="Font size" defaultValue="12"><option>10</option><option>11</option><option>12</option><option>14</option><option>18</option><option>24</option></select></div><div className="format-row"><button title="Bold" onClick={()=>setMessage('Rich text formatting is planned for a later release')}><Bold size={16}/></button><button title="Italic" onClick={()=>setMessage('Rich text formatting is planned for a later release')}><Italic size={16}/></button><button title="Underline" onClick={()=>setMessage('Rich text formatting is planned for a later release')}><Underline size={16}/></button><span className="vertical-rule"/><button title="Align left"><AlignLeft size={16}/></button><button title="Align center"><AlignCenter size={16}/></button><button title="Align right"><AlignRight size={16}/></button></div><span className="group-label">Font & paragraph</span></div>
      <div className="ribbon-group"><RibbonButton icon={LayoutTemplate} label="Layout" onClick={()=>setTab('Layout')}/><RibbonButton icon={PanelLeft} label="Panels" onClick={()=>setMessage('Panel layout is currently fixed')}/><RibbonButton icon={Maximize2} label="Zoom" onClick={()=>setZoom(z=>z===100?125:100)}/><span className="group-label">View</span></div>
      <div className="ribbon-group"><RibbonButton icon={Type} label="Insert text" onClick={()=>setMessage('Select a text area to type')}/><RibbonButton icon={Plus} label={mode==='powerpoint'?'New slide':mode==='excel'?'New sheet':'New file'} onClick={()=>{if(mode==='powerpoint')update(d=>({...d,slides:[...d.slides,{title:'New slide',body:''}],selected_slide:d.slides.length}));else if(mode==='excel')update(d=>({...d,sheets:[...d.sheets,{name:`Sheet${d.sheets.length+1}`,cells:{}}],selected_sheet:d.sheets.length}));else newFile()}}/><span className="group-label">Insert</span></div>
    </>}</div>
    {mode==='word'&&<main className="word-workspace"><aside className="word-sidebar"><div className="sidebar-heading"><Menu size={17}/> Navigation</div><div className="sidebar-muted">DOCUMENT OUTLINE</div><div className="nav-entry active"><FileText size={15}/> Page 1</div><p className="sidebar-help">Write your document on the page. DOCX import/export supports basic paragraph text.</p></aside><div className="word-scroll"><div className="page-ruler"><span>1</span><span>2</span><span>3</span><span>4</span><span>5</span><span>6</span></div><section className="paper"><textarea aria-label="Word document" placeholder="Start writing your document…" spellCheck value={doc.word} onChange={e=>update(d=>({...d,word:e.target.value}))}/></section></div></main>}
    {mode==='excel'&&<main className="excel-workspace"><div className="formula-bar"><div className="name-box">{colName(Number(selectedCell.split(':')[1]))}{Number(selectedCell.split(':')[0])+1}<ChevronDown size={14}/></div><span className="fx">ƒx</span><input aria-label="Formula bar" value={formula} onChange={e=>setCell(selectedCell,e.target.value)} placeholder="Enter a value or formula"/></div><div className="sheet-scroll"><table className="sheet-table"><thead><tr><th className="corner"></th>{Array.from({length:22},(_,c)=><th key={c}>{colName(c)}</th>)}</tr></thead><tbody>{Array.from({length:70},(_,r)=><tr key={r}><th>{r+1}</th>{Array.from({length:22},(_,c)=>{const key=`${r}:${c}`;return <td key={key} className={selectedCell===key?'cell-selected':''}><input aria-label={`Cell ${colName(c)}${r+1}`} value={sheet?.cells[key]??''} onFocus={()=>setSelectedCell(key)} onChange={e=>setCell(key,e.target.value)}/></td>})}</tr>)}</tbody></table></div><div className="sheet-tabs">{doc.sheets.map((s,i)=><button key={i} className={i===doc.selected_sheet?'active':''} onClick={()=>setDoc(d=>({...d,selected_sheet:i}))}><Table2 size={14}/>{s.name}</button>)}<button title="Add sheet" onClick={()=>update(d=>({...d,sheets:[...d.sheets,{name:`Sheet${d.sheets.length+1}`,cells:{}}],selected_sheet:d.sheets.length}))}><Plus size={17}/></button></div></main>}
    {mode==='powerpoint'&&<main className="powerpoint-workspace"><aside className="slides-sidebar"><div className="sidebar-heading">Slides <button title="Add slide" onClick={()=>update(d=>({...d,slides:[...d.slides,{title:'New slide',body:''}],selected_slide:d.slides.length}))}><Plus size={17}/></button></div>{doc.slides.map((s,i)=><button key={i} className={`slide-thumb ${i===doc.selected_slide?'active':''}`} onClick={()=>setDoc(d=>({...d,selected_slide:i}))}><span className="slide-index">{i+1}</span><span className="mini-slide"><strong>{s.title||'Untitled'}</strong><small>{s.body}</small></span></button>)}</aside><div className="slide-stage"><div className="slide-canvas"><input aria-label="Slide title" className="slide-title" value={slide?.title??''} placeholder="Click to add title" onChange={e=>setSlide('title',e.target.value)}/><textarea aria-label="Slide content" className="slide-body" value={slide?.body??''} placeholder="Click to add subtitle" onChange={e=>setSlide('body',e.target.value)}/></div><div className="slide-hint">Slide {doc.selected_slide+1} of {doc.slides.length} · 16:9 widescreen</div></div><aside className="properties-sidebar"><div className="sidebar-heading">Design</div><div className="sidebar-muted">SLIDE OPTIONS</div><div className="property-row"><LayoutTemplate size={16}/> Widescreen 16:9</div><div className="property-row"><Sparkles size={16}/> Clean light theme</div><button className="delete-slide" disabled={doc.slides.length<=1} onClick={()=>update(d=>({...d,slides:d.slides.filter((_,i)=>i!==d.selected_slide),selected_slide:Math.max(0,d.selected_slide-1)}))}><Trash2 size={16}/> Delete slide</button></aside></main>}
    <footer className="statusbar"><div className="status-left"><span className="status-dot"/>{message.length>110?message.slice(0,110)+'…':message}<span className="status-sep"/>{mode==='word'?`${words} words`:mode==='excel'?`${doc.sheets.length} sheet(s)`: `${doc.slides.length} slide(s)`}</div><div className="status-right"><span>{dirty?'Recovery snapshot enabled · Not saved to file':'Saved to file'}</span><span className="status-sep"/><button onClick={()=>setZoom(z=>Math.max(60,z-10))}>−</button><input aria-label="Zoom" type="range" min="60" max="150" value={zoom} onChange={e=>setZoom(Number(e.target.value))}/><button onClick={()=>setZoom(z=>Math.min(150,z+10))}>+</button><span>{zoom}%</span></div></footer>
  </div>
}
