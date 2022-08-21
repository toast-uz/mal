use itertools::Itertools;
use crate::types::*;

// ListVecしか対応していないため、HashMapにバグがある
pub fn pr_str(maltype: &MalType, print_readably: bool) -> String {
    if print_readably && maltype.string().is_some() {
//        eprintln!("String:{}", maltype);
        format!("\"{}\"", maltype.to_string().escape_default().to_string())
    } else if let MalType::ListVec(mlv) = maltype {
        if mlv.0 {
            format!("({})", mlv.1.iter().map(|x| pr_str(x, print_readably)).join(" "))
        } else {
            format!("[{}]", mlv.1.iter().map(|x| pr_str(x, print_readably)).join(" "))
        }
    } else {
        format!("{}", maltype)
    }
}
