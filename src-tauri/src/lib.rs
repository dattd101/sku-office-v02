mod formats;
mod model;
mod sessions;

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use model::{Presentation, Sheet as NativeSheet, Slide as NativeSlide, WordDocument, Workbook};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UiSheet { name: String, cells: BTreeMap<String,String> }
#[derive(Clone, Debug, Serialize, Deserialize)]
struct UiSlide { title: String, body: String }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OfficeDocument {
    pub(crate) mode: String,
    pub(crate) word: String,
    pub(crate) sheets: Vec<UiSheet>,
    pub(crate) selected_sheet: usize,
    pub(crate) slides: Vec<UiSlide>,
    pub(crate) selected_slide: usize,
}
impl OfficeDocument {
    pub(crate) fn empty(mode: &str) -> Self { Self {
        mode:mode.into(),word:String::new(),sheets:vec![UiSheet{name:"Sheet1".into(),cells:BTreeMap::new()}],
        selected_sheet:0,slides:vec![UiSlide{title:"New presentation".into(),body:String::new()}],selected_slide:0,
    }}
    fn as_book(&self)->Workbook {
        let sheets=self.sheets.iter().map(|s| NativeSheet {
            name:s.name.clone(),cells:s.cells.iter().filter_map(|(key,value)|{
                let (r,c)=key.split_once(':')?;
                Some(((r.parse::<u32>().ok()?,c.parse::<u16>().ok()?),value.clone()))
            }).collect(),
        }).collect();
        Workbook{sheets,selected:self.selected_sheet}
    }
    fn from_book(book:Workbook)->Self {
        let mut doc=Self::empty("excel");
        doc.selected_sheet=book.selected;
        doc.sheets=book.sheets.into_iter().map(|s|UiSheet{name:s.name,cells:s.cells.into_iter().map(|((r,c),v)|(format!("{r}:{c}"),v)).collect()}).collect();
        doc
    }
}

#[derive(Serialize)]
pub(crate) struct FileResult { pub(crate) path:String, pub(crate) document:OfficeDocument }

pub(crate) fn open_document()->Result<Option<FileResult>,String> {
    let Some(path)=rfd::FileDialog::new().add_filter("Office documents", &["docx","xlsx","xls","pptx","json"]).pick_file() else {return Ok(None)};
    let ext=path.extension().and_then(|e|e.to_str()).unwrap_or("").to_lowercase();
    let document=match ext.as_str() {
        "docx"=>{let word=formats::read_word(&path).map_err(|e|e.to_string())?;let mut d=OfficeDocument::empty("word");d.word=word.text;d},
        "xlsx"|"xls"=>OfficeDocument::from_book(formats::read_excel(&path).map_err(|e|e.to_string())?),
        "pptx"=>{let pres=formats::read_pptx(&path).map_err(|e|e.to_string())?;let mut d=OfficeDocument::empty("powerpoint");d.selected_slide=pres.selected;d.slides=pres.slides.into_iter().map(|s|UiSlide{title:s.title,body:s.body}).collect();d},
        "json"=>serde_json::from_slice(&std::fs::read(&path).map_err(|e|e.to_string())?).map_err(|e:serde_json::Error|e.to_string())?,
        _=>return Err("Unsupported file extension".into()),
    };
    Ok(Some(FileResult{path:path.display().to_string(),document}))
}

pub(crate) fn write_document(path:&Path,doc:&OfficeDocument)->Result<(),String> {
    let ext=path.extension().and_then(|e|e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "docx"=>formats::write_word(path,&WordDocument{text:doc.word.clone()}).map_err(|e|e.to_string()),
        "xlsx"=>formats::write_excel(path,&doc.as_book()).map_err(|e|e.to_string()),
        "pptx"=>formats::write_pptx(path,&Presentation{slides:doc.slides.iter().map(|s|NativeSlide{title:s.title.clone(),body:s.body.clone()}).collect(),selected:doc.selected_slide}).map_err(|e|e.to_string()),
        "json"=>std::fs::write(path,serde_json::to_vec_pretty(doc).map_err(|e|e.to_string())?).map_err(|e|e.to_string()),
        _=>Err("Choose a .docx, .xlsx, .pptx or .json filename".into()),
    }
}

#[tauri::command]
fn save_document(document:OfficeDocument,path:Option<String>,save_as:bool)->Result<Option<String>,String> {
    let target=if !save_as {path.map(std::path::PathBuf::from)} else {None};
    let path=match target {Some(p)=>p,None=>{
        let (ext,filter)=match document.mode.as_str(){"word"=>("docx","Word"),"excel"=>("xlsx","Excel"),_=>("pptx","PowerPoint")};
        let Some(p)=rfd::FileDialog::new().add_filter(filter,&[ext]).add_filter("Rust Office project", &["json"]).set_file_name(&format!("Untitled.{ext}")).save_file() else{return Ok(None)};
        p
    }};
    write_document(&path,&document)?;
    Ok(Some(path.display().to_string()))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![save_document, sessions::create_session, sessions::import_session, sessions::get_session, sessions::list_sessions, sessions::snapshot_session, sessions::save_session, sessions::discard_session, sessions::open_editor_window, sessions::focus_manager])
        .run(tauri::generate_context!())
        .expect("failed to run Rust Office Modern");
}
