use crate::model::{Presentation, Slide, WordDocument, Workbook, Sheet};
use anyhow::{anyhow, Context, Result};
use calamine::{open_workbook_auto, Reader};
use quick_xml::{events::Event, Reader as XmlReader};
use std::{fs::File, io::{Read, Write}, path::Path};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

fn escape(s:&str)->String { s.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;").replace('"',"&quot;").replace('\'',"&apos;") }
fn read_part(path:&Path, name:&str)->Result<String> {
    let mut archive=ZipArchive::new(File::open(path)?)?;
    let mut part=archive.by_name(name).with_context(||format!("Missing OOXML part: {name}"))?;
    let mut text=String::new(); part.read_to_string(&mut text)?; Ok(text)
}
fn xml_texts(xml:&str, tags:&[&[u8]])->Result<Vec<String>> {
    let mut r=XmlReader::from_str(xml);
    let mut output=Vec::new(); let mut inside=false;
    loop { match r.read_event()? {
        Event::Start(e) => { inside=tags.iter().any(|tag| e.local_name().as_ref()==*tag); },
        Event::Text(e) if inside => { output.push(e.unescape()?.into_owned()); },
        Event::End(e) if tags.iter().any(|tag| e.local_name().as_ref()==*tag) => inside=false,
        Event::Eof=>break, _=>{}
    }} Ok(output)
}
fn write_zip(path:&Path, parts:&[(&str,String)])->Result<()> {
    let mut zip=ZipWriter::new(File::create(path)?);
    for (name,content) in parts { zip.start_file(*name,SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated))?; zip.write_all(content.as_bytes())?; }
    zip.finish()?; Ok(())
}
pub fn read_word(path:&Path)->Result<WordDocument> {
    let xml=read_part(path,"word/document.xml")?;
    // Paragraph-aware extraction; formatting is not preserved in this starter.
    let mut reader=XmlReader::from_str(&xml); let mut text=String::new(); let mut in_text=false;
    loop { match reader.read_event()? {
        Event::Start(e) if e.local_name().as_ref()==b"t" => in_text=true,
        Event::Text(e) if in_text => text.push_str(&e.unescape()?),
        Event::End(e) if e.local_name().as_ref()==b"t" => in_text=false,
        Event::End(e) if e.local_name().as_ref()==b"p" => text.push('\n'),
        Event::Eof=>break, _=>{}
    }} Ok(WordDocument { text: text.trim_end_matches('\n').to_owned() })
}
pub fn write_word(path:&Path,doc:&WordDocument)->Result<()> {
    let mut body=String::new();
    for line in doc.text.split('\n') { body.push_str(&format!("<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",escape(line))); }
    let document=format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr/></w:body></w:document>"#);
    write_zip(path,&[
        ("[Content_Types].xml",r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#.into()),
        ("_rels/.rels",r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.into()),
        ("word/document.xml",document)
    ])
}
pub fn read_excel(path:&Path)->Result<Workbook> {
    let mut book=open_workbook_auto(path)?; let names=book.sheet_names().to_vec();
    let mut sheets=Vec::new();
    for name in names {
        let range=book.worksheet_range(&name)?;
        let mut sheet=Sheet { name, ..Default::default() };
        let (start_row,start_col)=range.start().unwrap_or((0,0));
        for (r,row) in range.rows().enumerate() { for (c,cell) in row.iter().enumerate() {
            let value=cell.to_string(); if !value.is_empty() { sheet.cells.insert((start_row+r as u32,(start_col+c as u32).try_into().context("Column index exceeds XLSX limits")?),value); }
        }} sheets.push(sheet);
    }
    if sheets.is_empty() { return Ok(Workbook::default()); }
    Ok(Workbook { sheets,selected:0 })
}
pub fn write_excel(path:&Path,book:&Workbook)->Result<()> {
    let mut x=rust_xlsxwriter::Workbook::new();
    for sheet in &book.sheets {
        let ws=x.add_worksheet(); ws.set_name(&sheet.name)?;
        for (&(row,col),value) in &sheet.cells {
            if value.starts_with('=') { ws.write_formula(row,col,value.as_str())?; }
            else if let Ok(n)=value.parse::<f64>() { ws.write_number(row,col,n)?; }
            else { ws.write_string(row,col,value)?; }
        }
    } x.save(path)?; Ok(())
}
pub fn read_pptx(path:&Path)->Result<Presentation> {
    let file=File::open(path)?; let mut zip=ZipArchive::new(file)?;
    let mut slide_parts=Vec::new();
    for i in 0..zip.len() { let name=zip.by_index(i)?.name().to_owned();
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") && !name.contains("_rels") { slide_parts.push(name); }
    }
    slide_parts.sort_by_key(|name| name.trim_start_matches("ppt/slides/slide").trim_end_matches(".xml").parse::<usize>().unwrap_or(usize::MAX));
    let mut slides=Vec::new();
    for part in slide_parts { let mut xml=String::new(); zip.by_name(&part)?.read_to_string(&mut xml)?;
        let texts=xml_texts(&xml,&[b"t"])?;
        slides.push(Slide { title:texts.first().cloned().unwrap_or_default(), body:texts.into_iter().skip(1).collect::<Vec<_>>().join("\n") });
    }
    if slides.is_empty() { return Err(anyhow!("No slides found")); }
    Ok(Presentation { slides,selected:0 })
}
pub fn write_pptx(path:&Path,pres:&Presentation)->Result<()> {
    let mut parts:Vec<(String,String)>=Vec::new();
    let mut overrides=String::new(); let mut ids=String::new(); let mut rels=String::new();
    for (i,slide) in pres.slides.iter().enumerate() {
        let n=i+1; overrides.push_str(&format!("<Override PartName=\"/ppt/slides/slide{n}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slide+xml\"/>"));
        ids.push_str(&format!("<p:sldId id=\"{}\" r:id=\"rId{n}\"/>",256+i));
        rels.push_str(&format!("<Relationship Id=\"rId{n}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide\" Target=\"slides/slide{n}.xml\"/>"));
        let shape=|id:usize,name:&str,text:&str,y:usize|format!(r#"<p:sp><p:nvSpPr><p:cNvPr id="{id}" name="{name}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="914400" y="{y}"/><a:ext cx="7315200" cy="1371600"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>{}</a:t></a:r><a:endParaRPr lang="en-US"/></a:p></p:txBody></p:sp>"#,escape(text));
        let xml=format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>{}{}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>"#,shape(2,"Title",&slide.title,700000),shape(3,"Content",&slide.body,2600000));
        parts.push((format!("ppt/slides/slide{n}.xml"),xml));
    }
    let types=format!(r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>{overrides}</Types>"#);
    let presxml=format!(r#"<?xml version="1.0" encoding="UTF-8"?><p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:sldIdLst>{ids}</p:sldIdLst><p:sldSz cx="9144000" cy="5143500" type="screen16x9"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#);
    parts.push(("[Content_Types].xml".into(),types));
    parts.push(("_rels/.rels".into(),r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/></Relationships>"#.into()));
    parts.push(("ppt/presentation.xml".into(),presxml));
    parts.push(("ppt/_rels/presentation.xml.rels".into(),format!(r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{rels}</Relationships>"#)));
    let borrowed:Vec<(&str,String)>=parts.iter().map(|(n,c)|(n.as_str(),c.clone())).collect(); write_zip(path,&borrowed)
}
