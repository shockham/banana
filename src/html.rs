pub fn elem(elem_type:&str, class:&str, content:String) -> String{
    format!("<{e_type} class=\"{class}\">{value}</{e_type}>",e_type=elem_type,value=content,class=class)
}

pub fn html(title:&str, content:String) -> String{
    format!("<!DOCTYPE html><head><title>{}</title></head><body>{}</body></html>", title, content)
}

