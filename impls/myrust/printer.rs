use crate::types::*;

pub fn pr_str(maltype: &MalType, print_readably: bool) -> String {
    if print_readably && maltype.string().is_some() {
//        eprintln!("String:{}", maltype);
        format!("\"{}\"", maltype.to_string().escape_default().to_string())
    } else {
        format!("{}", maltype)
    }
}
