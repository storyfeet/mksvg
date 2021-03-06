use std::io::Write;
use std::ffi::{OsStr,OsString};
use std::fs::File;
use std::path::{Path,PathBuf};
use {Svg,SvgW,qcast};
use cdnum::CDNum;
use std::process::Command;


pub trait Card<NT:CDNum> :Clone{
    fn front<S:Svg>(&self,svg:&mut S,w:NT,h:NT);
}


pub fn page<W:Write,NT:CDNum,C:Card<NT>>(w:W,pw:NT,ph:NT,nw:usize,nh:usize,cards:&[C]){
    let mut svg = SvgW::new(w);
    svg.start(pw,ph);

    //let n_20:NT = NumCast::from(20).unwrap();
    //let n_2:NT = NumCast::from(2).unwrap();
    let mw:NT = pw/qcast(20);
    let mh:NT = ph/qcast(20);
    let max = nw * nh;
    let cw = (pw - qcast::<i32,NT>(2)*mw )/qcast(nw);
    let ch = (ph - qcast::<i32,NT>(2)*mh)/qcast(nh);


    for (i,c) in cards.iter().enumerate(){
        let x:NT = qcast(i % nw);
        let y:NT = qcast(i / nw);
        svg.g_translate(mw+ x*cw,mh+y*ch,"");
        c.front(&mut svg,cw,ch);
        svg.g_end();
        if i+1 == max {
            break;
        }
    }

    svg.end();

}


pub fn page_a4<W:Write,NT:CDNum,C:Card<NT>>(w:W,nw:usize,nh:usize,cards:&[C]){
    page(w,qcast::<i32,NT>(2480),qcast::<i32,NT>(3508),nw,nh,cards);
}

pub fn pages<NT:CDNum,C:Card<NT>,P:AsRef<Path>>(basepath:P,pw:NT,ph:NT,nw:usize,nh:usize,cards:&[C])->Vec<PathBuf>{
    let mut res = Vec::new();
    let total = nw * nh; 

    let cpath:&Path = basepath.as_ref();
    let cname = OsString::from(cpath.file_name().unwrap_or(&OsStr::new("")));


    if cards.len() == 0 {
        return res;
    }
    //print!("\n{}\n",(cards.len()-1/total) +1);
    for i in 0 .. ((cards.len()-1) /total) +1 {
     //   print!("{}",i);
        let mut path = PathBuf::from(cpath.parent().unwrap_or(Path::new("")));
        let mut fname = cname.clone();
        fname.push(&format!("{}.svg",i));
        path.push(fname);
        let w = match File::create(&path) {
            Ok(f)=>f,
            Err(_)=>{
                return res
            }
        };
        page(w,pw,ph,nw,nh,&cards[i*total..]);
        res.push(path);
    }
    res 
}


pub fn pages_a4<NT:CDNum,C:Card<NT>,P:AsRef<Path>>(basepath:P,nw:usize,nh:usize,cards:&[C])->Vec<PathBuf>{
    pages(basepath,qcast::<i32,NT>(2480),qcast::<i32,NT>(3508),nw,nh,cards)
}

pub fn page_flip<T:Clone>(v:&Vec<T>,w:usize)->Vec<T>{
    //TODO
    let mut res:Vec<T> = Vec::new();
    if v.len() == 0 {
        return res;
    }
    let blank = v[0].clone();
    let mut tmp = Vec::new();
    for elem in v {
        tmp.push(elem.clone());
        if tmp.len() == w {
            for e2 in tmp.into_iter().rev() {
                res.push(e2);
            }
            tmp = Vec::new();
        }
    }

    if tmp.len() > 0{
        for _ in 0 .. w - tmp.len(){
            res.push(blank.clone());
        }
        for elem in tmp {
            res.push(elem);
        }
    }
    res
}

pub fn interlace<T:Clone>(a:Vec<T>,b:Vec<T>)->Vec<T>{
    let mut it_a = a.iter();
    let mut it_b = b.iter();
    let mut res:Vec<T> = Vec::new();
    loop {
        let mut done = 0;
        match it_a.next(){
            Some(t)=>res.push(t.clone()),
            None=>done += 1,
        }
        match it_b.next(){
            Some(t)=>res.push(t.clone()),
            None=>done+= 1,
        }
        if done == 2 {
            return res;
        }
    }
}

pub fn unite_as_pdf<P:AsRef<Path>,Q:AsRef<Path>>(v:Vec<P>,fpath:Q)->bool{
    let mut pdv:Vec<String> = Vec::new();  
    for i in v {

        //get .pdf path
        let op= PathBuf::from(i.as_ref());
        let mut pp = op.clone();
        pp.set_extension("pdf"); 

        let pps = pp.to_str().unwrap_or("cc.pdf");
        print!("Creating : {}",pps);


        let _output = Command::new("inkscape")
            .arg(op).arg(&format!("--export-pdf={}",pps))
            .output().expect("Could not run process");

        pdv.push(pps.to_string());
    }

    pdv.push(fpath.as_ref().to_str().unwrap_or("pooyt4.pdf").to_string());
    print!("Combining");
    Command::new("pdfunite").args(pdv).output().expect("could not unite the pdfs");

    true
}


#[cfg(test)]
mod tests {
    use page::page_flip;
    #[test]
    fn test_flip() {
        let v = vec![1,2,3,4,5,6,7,8,9];
        let v2 = page_flip(&v,4);
        assert_eq!(v2,vec![4,3,2,1,8,7,6,5,1,1,1,9]);
    }
}
