export type Mode='word'|'excel'|'powerpoint';
export type Slide={title:string;body:string};
export type Sheet={name:string;cells:Record<string,string>};
export type OfficeDocument={mode:Mode;word:string;sheets:Sheet[];selected_sheet:number;slides:Slide[];selected_slide:number};
export const fresh=(mode:Mode='word'):OfficeDocument=>({mode,word:'',sheets:[{name:'Sheet1',cells:{'0:0':'Product','0:1':'Quantity','0:2':'Price','1:0':'Example','1:1':'2','1:2':'150'}}],selected_sheet:0,slides:[{title:'Your presentation',body:'Click here to add your ideas'}],selected_slide:0});
