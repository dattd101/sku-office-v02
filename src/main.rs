mod formats;
mod model;
use eframe::egui;
use model::{Mode, WordDocument, Workbook, Presentation, Slide};
use std::path::PathBuf;

struct OfficeApp {
    mode: Mode, tab: usize, word: WordDocument, excel: Workbook, powerpoint: Presentation,
    path: Option<PathBuf>, status: String, row_offset:u32, col_offset:u16,
}
impl Default for OfficeApp { fn default()->Self { Self { mode:Mode::Word,tab:0,word:Default::default(),excel:Default::default(),powerpoint:Default::default(),path:None,status:"Ready".into(),row_offset:0,col_offset:0 } } }
impl OfficeApp {
    fn new_document(&mut self) { match self.mode { Mode::Word=>self.word=Default::default(),Mode::Excel=>self.excel=Default::default(),Mode::PowerPoint=>self.powerpoint=Default::default() }; self.path=None; self.status="New document".into(); }
    fn open(&mut self) {
        if let Some(path)=rfd::FileDialog::new().add_filter("Office files", &["docx","xlsx","xls","xlsb","ods","pptx"]).pick_file() {
            let ext=path.extension().and_then(|e|e.to_str()).unwrap_or("").to_ascii_lowercase();
            let result: anyhow::Result<()> = (|| { match ext.as_str() {
                "docx"=>{ self.word=formats::read_word(&path)?; self.mode=Mode::Word; },
                "xlsx"|"xls"|"xlsb"|"ods"=>{ self.excel=formats::read_excel(&path)?; self.mode=Mode::Excel; },
                "pptx"=>{ self.powerpoint=formats::read_pptx(&path)?; self.mode=Mode::PowerPoint; },
                _=>anyhow::bail!("Unsupported extension"),
            } Ok(()) })();
            match result { Ok(())=>{self.path=Some(path.clone());self.status=format!("Opened {}",path.display());},Err(e)=>self.status=format!("Open error: {e:#}") }
        }
    }
    fn save_as(&mut self) {
        let ext=match self.mode { Mode::Word=>"docx",Mode::Excel=>"xlsx",Mode::PowerPoint=>"pptx" };
        if let Some(path)=rfd::FileDialog::new().add_filter("Office document", &[ext]).set_file_name(&format!("untitled.{ext}")).save_file() { self.write_to(path); }
    }
    fn save(&mut self) {
        match self.path.clone() {
            Some(p) if p.extension().and_then(|e|e.to_str()).unwrap_or("").eq_ignore_ascii_case(match self.mode {Mode::Word=>"docx",Mode::Excel=>"xlsx",Mode::PowerPoint=>"pptx"}) => self.write_to(p),
            _=>self.save_as(),
        }
    }
    fn write_to(&mut self,path:PathBuf) {
        let result=match self.mode { Mode::Word=>formats::write_word(&path,&self.word),Mode::Excel=>formats::write_excel(&path,&self.excel),Mode::PowerPoint=>formats::write_pptx(&path,&self.powerpoint) };
        match result {Ok(())=>{self.status=format!("Saved {}",path.display());self.path=Some(path);},Err(e)=>self.status=format!("Save error: {e:#}")}
    }
    fn ribbon(&mut self,ui:&mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Rust Office"); ui.separator();
            for mode in [Mode::Word,Mode::Excel,Mode::PowerPoint] {
                if ui.selectable_label(self.mode==mode,mode.label()).clicked() {self.mode=mode;self.path=None;}
            }
            ui.separator(); if ui.button("New").clicked(){self.new_document();}
            if ui.button("Open…").clicked(){self.open();}
            if ui.button("Save").clicked(){self.save();}
            if ui.button("Save As…").clicked(){self.save_as();}
        });
        ui.separator();
        ui.horizontal(|ui| {for (i,name) in ["File","Home","Insert","Layout","Review","View"].iter().enumerate() {ui.selectable_value(&mut self.tab,i,*name);} });
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            match (self.mode,self.tab) {
                (Mode::PowerPoint,_) => {if ui.button("+ New slide").clicked(){self.powerpoint.slides.push(Slide::default());self.powerpoint.selected=self.powerpoint.slides.len()-1;}
                    if ui.button("Delete slide").clicked() && self.powerpoint.slides.len()>1 {self.powerpoint.slides.remove(self.powerpoint.selected);self.powerpoint.selected=self.powerpoint.selected.min(self.powerpoint.slides.len()-1);}
                },
                (Mode::Excel,_) => {if ui.button("+ Sheet").clicked(){let n=self.excel.sheets.len()+1;self.excel.sheets.push(model::Sheet{name:format!("Sheet{n}"),..Default::default()});self.excel.selected=n-1;}
                    if ui.button("← Columns").clicked(){self.col_offset=self.col_offset.saturating_sub(5);}
                    if ui.button("Columns →").clicked(){self.col_offset=self.col_offset.saturating_add(5).min(16370);}
                    if ui.button("↑ Rows").clicked(){self.row_offset=self.row_offset.saturating_sub(20);}
                    if ui.button("Rows ↓").clicked(){self.row_offset=self.row_offset.saturating_add(20);}
                },
                (Mode::Word,_)=>{ui.label("Plain-text editor · DOCX import/export");}
            }
        });
    }
    fn word_ui(&mut self,ui:&mut egui::Ui) {
        ui.heading("Document"); ui.separator();
        egui::ScrollArea::vertical().show(ui,|ui| {ui.add_sized([ui.available_width(),ui.available_height().max(500.0)],egui::TextEdit::multiline(&mut self.word.text).font(egui::TextStyle::Body).desired_width(f32::INFINITY));});
    }
    fn excel_ui(&mut self,ui:&mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {for i in 0..self.excel.sheets.len() {let label=self.excel.sheets[i].name.clone();if ui.selectable_label(self.excel.selected==i,label).clicked(){self.excel.selected=i;}}});
        ui.separator();
        egui::ScrollArea::both().auto_shrink([false,false]).show(ui,|ui| {
            egui::Grid::new("cells").striped(true).spacing([3.0,3.0]).show(ui,|ui| {
                ui.label("#");for c in 0..10 {ui.label(col_name(self.col_offset+c));}ui.end_row();
                for r in self.row_offset..self.row_offset+35 {
                    ui.label(format!("{}",r+1));
                    for c in self.col_offset..self.col_offset+10 {
                        let mut value=self.excel.sheet().cells.get(&(r,c)).cloned().unwrap_or_default();
                        if ui.add_sized([95.0,23.0],egui::TextEdit::singleline(&mut value)).changed() {self.excel.set(r,c,value);}
                    }ui.end_row();
                }
            });
        });
    }
    fn powerpoint_ui(&mut self,ui:&mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(160.0);ui.heading("Slides");
                for (i,s) in self.powerpoint.slides.iter().enumerate() {
                    if ui.selectable_label(self.powerpoint.selected==i,format!("{}  {}",i+1,s.title)).clicked(){self.powerpoint.selected=i;}
                }
            });ui.separator();
            ui.vertical(|ui| {
                ui.heading(format!("Slide {}",self.powerpoint.selected+1));
                let slide=&mut self.powerpoint.slides[self.powerpoint.selected];
                ui.label("Title");ui.add_sized([ui.available_width(),38.0],egui::TextEdit::singleline(&mut slide.title));
                ui.separator();ui.label("Content");ui.add_sized([ui.available_width(),350.0],egui::TextEdit::multiline(&mut slide.body));
            });
        });
    }
}
fn col_name(mut col:u16)->String {let mut s=String::new();loop {s.insert(0,(b'A'+(col%26) as u8) as char);if col<26{break;}col=col/26-1;}s}
impl eframe::App for OfficeApp {
    fn update(&mut self,ctx:&egui::Context,_frame:&mut eframe::Frame) {
        egui::TopBottomPanel::top("ribbon").show(ctx,|ui|self.ribbon(ui));
        egui::TopBottomPanel::bottom("status").show(ctx,|ui|{ui.horizontal(|ui|{ui.label(&self.status);ui.separator();ui.label(format!("Mode: {}",self.mode.label()));});});
        egui::CentralPanel::default().show(ctx,|ui|match self.mode {Mode::Word=>self.word_ui(ui),Mode::Excel=>self.excel_ui(ui),Mode::PowerPoint=>self.powerpoint_ui(ui)});
    }
}
fn main()->eframe::Result<()> {
    let options=eframe::NativeOptions {viewport:egui::ViewportBuilder::default().with_title("Rust Office Desktop").with_inner_size([1200.0,800.0]),..Default::default()};
    eframe::run_native("Rust Office Desktop",options,Box::new(|_|Ok(Box::<OfficeApp>::default())))
}
#[cfg(test)] mod tests {use super::*;#[test] fn columns(){assert_eq!(col_name(0),"A");assert_eq!(col_name(25),"Z");assert_eq!(col_name(26),"AA");}}
