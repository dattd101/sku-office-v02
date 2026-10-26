use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode { Word, Excel, PowerPoint }
impl Mode { pub fn label(self) -> &'static str { match self { Self::Word => "Word", Self::Excel => "Excel", Self::PowerPoint => "PowerPoint" } } }

#[derive(Debug, Clone, Default)]
pub struct WordDocument { pub text: String }
#[derive(Debug, Clone, Default)]
pub struct Sheet { pub name: String, pub cells: BTreeMap<(u32,u16), String> }
#[derive(Debug, Clone)]
pub struct Workbook { pub sheets: Vec<Sheet>, pub selected: usize }
impl Default for Workbook { fn default() -> Self { Self { sheets: vec![Sheet { name: "Sheet1".into(), ..Default::default() }], selected: 0 } } }
impl Workbook {
    pub fn sheet(&self) -> &Sheet { &self.sheets[self.selected] }
    pub fn sheet_mut(&mut self) -> &mut Sheet { &mut self.sheets[self.selected] }
    pub fn set(&mut self, row:u32, col:u16, value:String) {
        if value.is_empty() { self.sheet_mut().cells.remove(&(row,col)); }
        else { self.sheet_mut().cells.insert((row,col),value); }
    }
}
#[derive(Debug, Clone, Default)]
pub struct Slide { pub title:String, pub body:String }
#[derive(Debug, Clone)]
pub struct Presentation { pub slides:Vec<Slide>, pub selected:usize }
impl Default for Presentation { fn default() -> Self { Self { slides: vec![Slide { title:"New presentation".into(), body:String::new() }], selected:0 } } }
