'use client';
import Office from '@/components/office';
import Manager from '@/components/manager';
export default function Page(){
  const isEditor=typeof window!=='undefined'&&new URLSearchParams(window.location.search).get('view')==='editor';
  return isEditor?<Office/>:<Manager/>;
}
