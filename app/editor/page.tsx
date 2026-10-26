'use client';
import Office from '@/components/office';
import { useEffect, useState } from 'react';
export default function EditorPage(){
 const [error,setError]=useState('');
 useEffect(()=>{const report=(event:ErrorEvent)=>setError(event.message||'Unknown JavaScript error');const rejection=(event:PromiseRejectionEvent)=>setError(String(event.reason));window.addEventListener('error',report);window.addEventListener('unhandledrejection',rejection);return()=>{window.removeEventListener('error',report);window.removeEventListener('unhandledrejection',rejection)}},[]);
 if(error)return <main style={{padding:32,fontFamily:'sans-serif'}}><h1>Editor could not start</h1><pre style={{whiteSpace:'pre-wrap'}}>{error}</pre><p>Copy this message when reporting a problem.</p></main>;
 return <Office/>;
}
